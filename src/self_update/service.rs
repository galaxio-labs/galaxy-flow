use std::path::PathBuf;

use chrono::Utc;

use orion_error::conversion::ToStructError;
use orion_error::conversion::{ErrorWith, SourceErr};
use orion_error::reason::UnifiedReason as UvsReason;

use crate::err::{RunReason, RunResult};

use super::model::{CheckResult, ReleaseChannel, SelfUpdateState, StatusResult, UpdateResult};
use super::rollback;
use super::storage::SelfUpdateStorage;

const MANIFEST_BASE_URL: &str =
    "https://raw.githubusercontent.com/galaxio-labs/get/main/updates/gx";
const PRODUCT_NAME: &str = "gx";

#[derive(Clone, Debug, Default)]
pub struct CheckRequest {
    pub channel: ReleaseChannel,
}

#[derive(Clone, Debug, Default)]
pub struct UpdateRequest {
    pub channel: ReleaseChannel,
    pub to_version: Option<String>,
    pub yes: bool,
    pub dry_run: bool,
    pub force: bool,
}

#[derive(Clone, Debug)]
pub struct SelfUpdateService {
    storage: SelfUpdateStorage,
}

impl SelfUpdateService {
    pub fn new() -> RunResult<Self> {
        Ok(Self {
            storage: SelfUpdateStorage::new()?,
        })
    }

    pub fn status(&self) -> RunResult<StatusResult> {
        let install_dir = resolve_install_dir()?;
        let mut state = self.storage.load_state()?;
        state.current_version = Some(env!("CARGO_PKG_VERSION").to_string());
        Ok(StatusResult {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            install_dir,
            state,
        })
    }

    pub async fn check(&self, req: CheckRequest) -> RunResult<CheckResult> {
        let _lock = self.storage.acquire_lock()?;
        let mut state = self.storage.load_state()?;
        let channel = req.channel;

        let wp_channel = wp_self_update::UpdateChannel::from(channel);
        let wp_source = wp_self_update::SourceConfig {
            channel: wp_channel,
            kind: wp_self_update::SourceKind::Manifest {
                updates_base_url: MANIFEST_BASE_URL.to_string(),
                updates_root: None,
            },
        };

        let wp_req = wp_self_update::CheckRequest {
            product: PRODUCT_NAME.to_string(),
            source: wp_source,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            branch: channel.as_str().to_string(),
        };

        let report = wp_self_update::check(wp_req).await.map_err(|e| {
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                None,
                "check_failed",
                &e.to_string(),
            );
            convert_wp_error(e)
        })?;

        state.last_checked_at = Some(now_text());
        state.last_channel = Some(channel);
        state.last_remote_version = Some(report.latest_version.clone());
        state.last_result = Some(if report.update_available {
            "update_available".to_string()
        } else {
            "up_to_date".to_string()
        });
        state.last_error = None;
        self.storage.save_state(&state)?;

        Ok(CheckResult {
            channel,
            current_version: report.current_version,
            remote_version: report.latest_version,
            has_update: report.update_available,
        })
    }

    pub async fn update(&self, req: UpdateRequest) -> RunResult<UpdateResult> {
        let _lock = self.storage.acquire_lock()?;
        let mut state = self.storage.load_state()?;
        let channel = req.channel;
        let current = env!("CARGO_PKG_VERSION").to_string();

        let wp_channel = wp_self_update::UpdateChannel::from(channel);
        let wp_source = wp_self_update::SourceConfig {
            channel: wp_channel,
            kind: wp_self_update::SourceKind::Manifest {
                updates_base_url: MANIFEST_BASE_URL.to_string(),
                updates_root: None,
            },
        };

        // First check to get remote version
        let check_req = wp_self_update::CheckRequest {
            product: PRODUCT_NAME.to_string(),
            source: wp_source.clone(),
            current_version: current.clone(),
            branch: channel.as_str().to_string(),
        };

        let check_report = wp_self_update::check(check_req).await.map_err(|e| {
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                None,
                "update_failed",
                &e.to_string(),
            );
            convert_wp_error(e)
        })?;

        let remote = check_report.latest_version.clone();

        // Validate to_version if specified
        if let Some(expect) = &req.to_version
            && expect.trim() != remote
        {
            let err = RunReason::Args
                .to_err()
                .doing("validate target update version")
                .with_context(("expect", expect.as_str()))
                .with_context(("manifest", remote.as_str()))
                .with_detail(format!(
                    "target version mismatch: expect={expect}, manifest={remote}"
                ));
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(remote.clone()),
                "update_failed",
                &err.to_string(),
            );
            return Err(err);
        }

        // Check if update is needed
        if !req.force && !check_report.update_available {
            state.last_checked_at = Some(now_text());
            state.last_channel = Some(channel);
            state.last_remote_version = Some(remote.clone());
            state.last_result = Some("up_to_date".to_string());
            state.last_error = None;
            self.storage.save_state(&state)?;
            return Ok(UpdateResult {
                channel,
                from_version: current,
                to_version: remote,
                backup_id: None,
                updated: false,
            });
        }

        if req.dry_run {
            state.last_checked_at = Some(now_text());
            state.last_channel = Some(channel);
            state.last_remote_version = Some(remote.clone());
            state.last_result = Some("dry_run".to_string());
            state.last_error = None;
            self.storage.save_state(&state)?;
            return Ok(UpdateResult {
                channel,
                from_version: current,
                to_version: remote,
                backup_id: None,
                updated: false,
            });
        }

        // Perform update using wp-self-update
        let install_dir = resolve_install_dir()?;
        let wp_update_req = wp_self_update::UpdateRequest {
            product: PRODUCT_NAME.to_string(),
            target: wp_self_update::UpdateTarget::Bins(vec!["gx".to_string()]),
            source: wp_source,
            current_version: current.clone(),
            install_dir: Some(install_dir.clone()),
            yes: req.yes,
            dry_run: false,
            force: req.force,
        };

        let update_report = wp_self_update::update(wp_update_req).await.map_err(|e| {
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(remote.clone()),
                "update_failed",
                &e.to_string(),
            );
            convert_wp_error(e)
        })?;

        let backup_id = now_id();
        state.last_checked_at = Some(now_text());
        state.last_channel = Some(channel);
        state.last_remote_version = Some(remote.clone());
        state.last_result = Some("updated".to_string());
        state.last_error = None;
        state.current_version = Some(remote.clone());
        state.installed_at = Some(now_text());
        state.last_backup_id = Some(backup_id.clone());
        self.storage.save_state(&state)?;

        Ok(UpdateResult {
            channel,
            from_version: current,
            to_version: remote,
            backup_id: Some(backup_id),
            updated: update_report.updated,
        })
    }

    pub fn rollback(&self, id: Option<&str>) -> RunResult<UpdateResult> {
        let _lock = self.storage.acquire_lock()?;
        let backups = self.storage.list_backups_desc()?;
        let backup_id = select_backup_id(&backups, id)?;
        let mut state = self.storage.load_state()?;
        let channel = state.last_channel.unwrap_or_default();
        let from_version = state
            .current_version
            .clone()
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
        let backup_dir = self.storage.backups_dir().join(&backup_id);
        let install_dir = resolve_install_dir().inspect_err(|e| {
            let _ = record_rollback_failure_state(&self.storage, &mut state, &e.to_string());
        })?;
        rollback::rollback(&install_dir, &backup_dir).inspect_err(|e| {
            let _ = record_rollback_failure_state(&self.storage, &mut state, &e.to_string());
        })?;
        rollback::health_check(&install_dir).inspect_err(|e| {
            let _ = record_rollback_failure_state(&self.storage, &mut state, &e.to_string());
        })?;
        let restored_version = read_installed_version(&install_dir);
        state.installed_at = Some(now_text());
        match restored_version {
            Ok(v) => {
                state.last_result = Some("rollback".to_string());
                state.last_error = None;
                state.current_version = Some(v.clone());
                self.storage.save_state(&state)?;
                Ok(UpdateResult {
                    channel,
                    from_version,
                    to_version: v,
                    backup_id: Some(backup_id),
                    updated: true,
                })
            }
            Err(e) => {
                state.last_result = Some("rollback_version_unknown".to_string());
                state.last_error = Some(e.to_string());
                self.storage.save_state(&state)?;
                Ok(UpdateResult {
                    channel,
                    from_version,
                    to_version: "unknown".to_string(),
                    backup_id: Some(backup_id),
                    updated: true,
                })
            }
        }
    }
}

fn resolve_install_dir() -> RunResult<PathBuf> {
    let exe = std::env::current_exe()
        .source_err(UvsReason::system_error().into(), "source error")
        .doing("resolve current executable path")?;
    exe.parent()
        .map(PathBuf::from)
        .ok_or_else(|| {
            RunReason::Exec
                .to_err()
                .with_detail("cannot resolve install dir")
        })
        .doing("resolve install dir from current executable")
        .with_context(("exe", exe.as_path()))
}

fn record_failure_state(
    storage: &SelfUpdateStorage,
    state: &mut SelfUpdateState,
    channel: ReleaseChannel,
    remote_version: Option<String>,
    result: &str,
    err: &str,
) -> RunResult<()> {
    state.last_checked_at = Some(now_text());
    state.last_channel = Some(channel);
    state.last_remote_version = remote_version;
    state.last_result = Some(result.to_string());
    state.last_error = Some(err.to_string());
    storage.save_state(state)
}

fn record_rollback_failure_state(
    storage: &SelfUpdateStorage,
    state: &mut SelfUpdateState,
    err: &str,
) -> RunResult<()> {
    state.last_result = Some("rollback_failed".to_string());
    state.last_error = Some(err.to_string());
    storage.save_state(state)
}

fn select_backup_id(backups: &[String], id: Option<&str>) -> RunResult<String> {
    match id {
        Some(raw) => {
            if !is_valid_backup_id(raw) {
                return Err(RunReason::Args
                    .to_err()
                    .doing("validate backup id")
                    .with_context(("backup_id", raw))
                    .with_detail(format!(
                        "invalid backup id: backup_id={raw}, expected=14 digits"
                    )));
            }
            if backups.iter().any(|v| v == raw) {
                Ok(raw.to_string())
            } else {
                Err(RunReason::Args
                    .to_err()
                    .doing("select rollback backup id")
                    .with_context(("backup_id", raw))
                    .with_detail(format!("backup id not found: backup_id={raw}")))
            }
        }
        None => backups
            .first()
            .cloned()
            .ok_or_else(|| RunReason::Args.to_err().with_detail("no backup found"))
            .doing("select latest rollback backup"),
    }
}

fn is_valid_backup_id(input: &str) -> bool {
    input.len() == 14 && input.bytes().all(|b| b.is_ascii_digit())
}

fn now_id() -> String {
    Utc::now().format("%Y%m%d%H%M%S").to_string()
}

fn now_text() -> String {
    Utc::now().to_rfc3339()
}

fn read_installed_version(install_dir: &std::path::Path) -> RunResult<String> {
    let bin = if cfg!(windows) {
        install_dir.join("gx.exe")
    } else {
        install_dir.join("gx")
    };
    let out = std::process::Command::new(&bin)
        .arg("--version")
        .output()
        .source_err(UvsReason::resource_error().into(), "source error")
        .doing("run installed binary version command")
        .with_context(("bin", &bin))?;
    if !out.status.success() {
        return Err(RunReason::Exec
            .to_err()
            .doing("read installed binary version")
            .with_context(("bin", &bin))
            .with_detail(format!(
                "version command failed: {} --version exit={}",
                bin.display(),
                out.status
            )));
    }
    let text = format!(
        "{} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    parse_version_from_text(&text).ok_or_else(|| {
        RunReason::Exec
            .to_err()
            .doing("parse installed binary version output")
            .with_context(("bin", &bin))
            .with_detail(format!(
                "cannot parse version output: bin={}",
                bin.display()
            ))
    })
}

fn parse_version_from_text(text: &str) -> Option<String> {
    for tok in text.split_whitespace() {
        let normalized = tok
            .trim_matches(|c: char| ['"', ',', ':', ';', '(', ')'].contains(&c))
            .trim();
        if normalized.is_empty() {
            continue;
        }
        if let Ok(v) = semver::Version::parse(normalized.trim_start_matches('v')) {
            return Some(v.to_string());
        }
    }
    None
}

fn convert_wp_error(e: impl std::fmt::Display) -> crate::err::RunError {
    crate::err::RunReason::Exec
        .to_err()
        .with_detail(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{is_valid_backup_id, select_backup_id};

    #[test]
    fn backup_id_requires_membership() {
        let backups = vec!["20260305010101".to_string()];
        assert!(select_backup_id(&backups, Some("20260305010101")).is_ok());
        assert!(select_backup_id(&backups, Some("20260305010102")).is_err());
    }

    #[test]
    fn backup_id_rejects_path_like_input() {
        let backups = vec!["20260305010101".to_string()];
        assert!(select_backup_id(&backups, Some("../20260305010101")).is_err());
        assert!(select_backup_id(&backups, Some("/tmp/x")).is_err());
    }

    #[test]
    fn validate_backup_id() {
        assert!(is_valid_backup_id("20260321123456"));
        assert!(!is_valid_backup_id("2026032112345")); // too short
        assert!(!is_valid_backup_id("20260321123456a")); // has letter
    }
}
