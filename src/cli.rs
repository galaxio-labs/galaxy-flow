use std::ffi::OsStr;
use std::path::Path;

use clap::Parser;
use orion_accessor::addr::GitRepository;
use orion_error::conversion::{ConvErr, SourceErr, SourceRawErr, ToStructError};

use crate::GxLoader;
use crate::cmd::gx_cmd::{AdmCmd, DocArgs, GxCmd, InitCmd, ModCmd, RunCmd, SelfCmd};
use crate::cmd::gxl_cmd::GFlowCmd;
use crate::conf::load_gxl_config;
use crate::const_val::gxl_const::CMD_ARG;
use crate::err::{RunReason, RunResult};
use crate::execution::VarSpace;
use crate::galaxy::Galaxy;
use crate::help;
use crate::infra::configure_run_logging;
use crate::parser::abilities::addr::gal_extern_mod;
use crate::parser::externs::{DslStatus, ExternParser};
use crate::runner::GxlRunner;
use crate::self_update::{
    CheckRequest, CheckResult, ReleaseChannel, SelfUpdateService, UpdateRequest,
};
use crate::traits::Setter;
use crate::util::diagnose::ai_diagnose;
use crate::util::redirect::stop_redirect;
use wp_self_update::{VersionRelation, compare_versions_str, relation_message};

const DEFAULT_WORK_CONF: &str = "./_gal/work.gxl";
const DEFAULT_ADM_CONF: &str = "./_gal/adm.gxl";

pub async fn run_from_env() -> RunResult<()> {
    let args = normalized_argv(std::env::args());
    let cmd = GxCmd::parse_from(args);
    dispatch(cmd).await
}

pub fn normalized_argv<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args: Vec<String> = args.into_iter().map(Into::into).collect();
    if args.is_empty() {
        args.push("gx".to_string());
    }

    let bin = Path::new(&args[0])
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("gx");

    match bin {
        "grun" => args.insert(1, "run".to_string()),
        "gadm" => args.insert(1, "adm".to_string()),
        _ => {}
    }

    args
}

pub async fn dispatch(cmd: GxCmd) -> RunResult<()> {
    if let GxCmd::Doc(args) = cmd.clone() {
        return do_doc_cmd(args);
    }

    if matches!(output_mode(&cmd), OutputMode::Human) {
        eprintln!("galaxy-flow : {}", env!("CARGO_PKG_VERSION"));
    }

    let mut gx = GxLoader::new();
    match cmd {
        GxCmd::Run(cmd) => do_run_cmd(cmd.cmd).await?,
        GxCmd::Adm(cmd) => do_adm_cmd(cmd.cmd).await?,
        GxCmd::Init(prj_cmd) => do_prj_cmd(&mut gx, prj_cmd).await?,
        GxCmd::Mod(mod_cmd) => do_mod_cmd(&mut gx, mod_cmd).await?,
        GxCmd::Doc(_args) => unreachable!("doc is handled before command dispatch"),
        GxCmd::Check => do_check_cmd()?,
        GxCmd::SelfUpdate(cmd) => do_self_cmd(cmd).await?,
    }
    Ok(())
}

async fn do_run_cmd(mut cmd: GFlowCmd) -> RunResult<()> {
    use std::process;

    let mut var_space = VarSpace::sys_init().conv_err()?;

    configure_cli_runtime(cmd.log.clone(), cmd.debug);

    let redirect = crate::model::task_report::task_rc_config::init_redirect_and_parent_task(
        cmd.flows.join(","),
        cmd.ai,
    )
    .await
    .conv_err()?;

    if cmd.conf.is_none() {
        cmd.conf = Some(DEFAULT_WORK_CONF.to_string());
    }
    var_space.global_mut().set(CMD_ARG, cmd.cmd_args.join(" "));

    if cmd.list_cmd().is_empty() {
        if !cmd.quiet {
            GxlRunner::info(cmd.conf.clone(), var_space).await?;
        }
        let _ = stop_redirect(redirect);
        return Ok(());
    } else {
        for cmd in cmd.list_cmd() {
            match GxlRunner::run(cmd.clone(), var_space.clone(), None).await {
                Err(e) => {
                    crate::err::report_gxl_error(e);
                    if cmd.ai
                        && let Err(e) = ai_diagnose(&var_space).await
                    {
                        crate::err::report_gxl_error(e);
                    }
                }
                Ok(_) => {
                    let _ = stop_redirect(redirect);
                    return Ok(());
                }
            }
        }
    }
    let _ = stop_redirect(redirect);
    process::exit(-1);
}

async fn do_adm_cmd(mut cmd: GFlowCmd) -> RunResult<()> {
    use std::process;

    configure_cli_runtime(cmd.log.clone(), cmd.debug);
    let mut var_space = VarSpace::sys_init().conv_err()?;
    var_space.global_mut().set(CMD_ARG, cmd.cmd_args.join(" "));

    if cmd.conf.is_none() {
        cmd.conf = Some(DEFAULT_ADM_CONF.to_string());
    }
    if cmd.list_cmd().is_empty() {
        if !cmd.quiet {
            GxlRunner::info(cmd.conf.clone(), var_space).await?;
        }
        return Ok(());
    } else {
        for cmd in cmd.list_cmd() {
            match GxlRunner::run(cmd.clone(), var_space.clone(), None).await {
                Err(e) => {
                    crate::err::report_gxl_error(e);
                    if cmd.ai
                        && let Err(e) = ai_diagnose(&var_space).await
                    {
                        crate::err::report_gxl_error(e);
                    }
                }

                Ok(_) => {
                    return Ok(());
                }
            }
        }
    }
    process::exit(-1);
}

fn do_doc_cmd(args: DocArgs) -> RunResult<()> {
    help::print(args.topic.as_deref(), args.markdown)
}

fn do_check_cmd() -> RunResult<()> {
    let info = os_info::get();
    println!("galaxy flow running env info");
    println!("OS : {info}");
    println!("Type: {}", info.os_type());
    println!("Version: {}", info.version());
    println!("Bitness: {}", info.bitness());
    if let Some(arch) = info.architecture() {
        println!("Architecture: {arch}");
    }
    println!("evn path:{}", env!("PATH"));
    Ok(())
}

async fn do_prj_cmd(load: &mut GxLoader, cmd: InitCmd) -> RunResult<()> {
    const DEFAULT_REPO: &str = "https://github.com/galaxio-labs/prj-tpl.git";

    match cmd {
        InitCmd::Env => Galaxy::env_init()?,
        InitCmd::Project(args) => {
            configure_cli_runtime(args.log.clone(), args.debug);

            // Validate: --branch/--tag require --repo or --path
            if (args.branch.is_some() || args.tag.is_some())
                && args.repo.is_none()
                && args.path.is_none()
            {
                return Err(
                    RunReason::Args
                        .to_err()
                        .with_detail("--branch/--tag require --repo or --path; use: gx init project --path rust --branch main"),
                );
            }

            if args.repo.is_some() || args.path.is_some() {
                // Remote init: use specified repo or default
                let _stdout_guard = StdoutToStderrGuard::new()?;
                let repo = args.repo.as_deref().unwrap_or(DEFAULT_REPO);
                let mut addr = GitRepository::from(repo);
                if let Some(path) = args.path() {
                    addr = addr.with_path(path);
                }
                if let Some(tag) = args.tag() {
                    addr = addr.with_tag(tag);
                } else if let Some(branch) = args.branch() {
                    addr = addr.with_branch(branch);
                }
                load.init_from_git(addr).await?;
            } else {
                // Local init: no network required
                Galaxy::project_init()?;
            }
        }
    }
    Ok(())
}

async fn do_mod_cmd(load: &mut GxLoader, mod_cmd: ModCmd) -> RunResult<()> {
    match mod_cmd {
        ModCmd::Update(args) => {
            configure_cli_runtime(args.log.clone(), args.debug);

            let vars = VarSpace::sys_init().conv_err()?;
            let confs = collect_mod_update_inputs()?;
            let mut updated_mods = Vec::new();

            for conf in confs {
                let conf_mods = collect_git_extern_mod_names(conf)?;
                if conf_mods.is_empty() {
                    eprintln!("no git extern modules to update in {conf}");
                } else {
                    eprintln!(
                        "updating git extern modules from {conf}: {}",
                        conf_mods.join(", ")
                    );
                    updated_mods.extend(conf_mods);
                }
                load.parse_file(conf, true, &vars).await?;
            }
            updated_mods.sort();
            updated_mods.dedup();
            if updated_mods.is_empty() {
                eprintln!("project modules updated: no git extern modules found");
            } else {
                eprintln!("project modules updated: {}", updated_mods.join(", "));
            }
        }
    }
    Ok(())
}

fn configure_cli_runtime(log: Option<String>, debug: usize) {
    configure_run_logging(log, debug);
    load_gxl_config();
}

struct StdoutToStderrGuard {
    #[cfg(unix)]
    saved_stdout_fd: i32,
}

impl StdoutToStderrGuard {
    fn new() -> RunResult<Self> {
        #[cfg(unix)]
        {
            let saved_stdout_fd = unsafe { libc::dup(libc::STDOUT_FILENO) };
            if saved_stdout_fd < 0 {
                return Err(RunReason::Exec.to_err().with_detail("dup stdout failed"));
            }

            if unsafe { libc::dup2(libc::STDERR_FILENO, libc::STDOUT_FILENO) } < 0 {
                unsafe {
                    libc::close(saved_stdout_fd);
                }
                return Err(RunReason::Exec
                    .to_err()
                    .with_detail("redirect stdout to stderr failed"));
            }

            Ok(Self { saved_stdout_fd })
        }

        #[cfg(not(unix))]
        {
            Ok(Self {})
        }
    }
}

impl Drop for StdoutToStderrGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::dup2(self.saved_stdout_fd, libc::STDOUT_FILENO);
            libc::close(self.saved_stdout_fd);
        }
    }
}

fn collect_mod_update_inputs() -> RunResult<Vec<&'static str>> {
    let mut inputs = Vec::new();

    if std::path::Path::new(DEFAULT_WORK_CONF).exists() {
        inputs.push(DEFAULT_WORK_CONF);
    }
    if std::path::Path::new(DEFAULT_ADM_CONF).exists() {
        inputs.push(DEFAULT_ADM_CONF);
    }

    if inputs.is_empty() {
        return Err(RunReason::Args.to_err().with_detail(format!(
            "expected at least one config file: {} or {}",
            DEFAULT_WORK_CONF, DEFAULT_ADM_CONF
        )));
    }

    Ok(inputs)
}

fn collect_git_extern_mod_names(conf: &str) -> RunResult<Vec<String>> {
    let code = std::fs::read_to_string(conf)
        .source_err(RunReason::from_conf(), format!("read config file: {conf}"))?;
    collect_git_extern_mod_names_from_code(code.as_str())
}

fn collect_git_extern_mod_names_from_code(code: &str) -> RunResult<Vec<String>> {
    let mut input = code;
    let mut mods = Vec::new();

    loop {
        let (chunk, status) = ExternParser::parse_code(&mut input).map_err(|e| {
            RunReason::Gxl
                .to_err()
                .with_detail(format!("parse extern mod list failed: {e}"))
        })?;
        let _ = chunk;
        match status {
            DslStatus::Extern => {
                let mod_ref = gal_extern_mod(&mut input).map_err(|e| {
                    RunReason::Gxl
                        .to_err()
                        .with_detail(format!("parse extern mod ref failed: {e}"))
                })?;
                if let crate::components::gxl_extend::ModAddr::Git(_) = mod_ref.addr() {
                    mods.extend(mod_ref.mods().iter().cloned());
                }
            }
            DslStatus::End => break,
            DslStatus::Code | DslStatus::Data => break,
        }
    }

    mods.sort();
    mods.dedup();
    Ok(mods)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Human,
    Machine,
}

fn output_mode(cmd: &GxCmd) -> OutputMode {
    match cmd {
        GxCmd::Run(RunCmd { cmd }) | GxCmd::Adm(AdmCmd { cmd }) if cmd.quiet => OutputMode::Machine,
        GxCmd::SelfUpdate(SelfCmd::Check(args)) if args.json => OutputMode::Machine,
        _ => OutputMode::Human,
    }
}

async fn do_self_cmd(cmd: SelfCmd) -> RunResult<()> {
    let svc = SelfUpdateService::new()?;
    match cmd {
        SelfCmd::Status => {
            let status = svc.status()?;
            println!("current_version={}", status.current_version);
            println!("install_dir={}", status.install_dir.display());
            if let Some(v) = status.state.last_remote_version {
                println!("state.last_remote_version={v}");
            }
            if let Some(v) = status.state.last_result {
                println!("state.last_result={v}");
            }
            if let Some(v) = status.state.last_error {
                println!("state.last_error={v}");
            }
        }
        SelfCmd::Check(args) => {
            let channel = parse_channel(args.channel.as_str())?;
            let req = CheckRequest { channel };
            let out = svc.check(req).await?;
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "channel": out.channel.as_str(),
                        "current_version": out.current_version,
                        "remote_version": out.remote_version,
                        "has_update": out.has_update
                    }))
                    .source_raw_err(RunReason::Exec, "serialize self-check json")?
                );
            } else {
                print_self_check_report(&out)?;
            }
        }
        SelfCmd::Update(args) => {
            let channel = parse_channel(args.channel.as_str())?;
            let req = UpdateRequest {
                channel,
                to_version: args.to_version.clone(),
                yes: args.yes,
                dry_run: args.dry_run,
                force: args.force,
            };
            let out = svc.update(req).await?;
            println!("channel={}", out.channel.as_str());
            println!("from={}", out.from_version);
            println!("to={}", out.to_version);
            println!("updated={}", out.updated);
            if let Some(id) = out.backup_id {
                println!("backup_id={id}");
            }
        }
        SelfCmd::Rollback(args) => {
            let out = svc.rollback(args.backup_id.as_deref())?;
            println!("rollback=true");
            if let Some(id) = out.backup_id {
                println!("backup_id={id}");
            }
        }
    }
    Ok(())
}

fn parse_channel(input: &str) -> RunResult<ReleaseChannel> {
    ReleaseChannel::parse(input).ok_or_else(|| {
        RunReason::Args
            .to_err()
            .with_detail(format!("channel={input}, expected=stable|alpha|beta"))
    })
}

fn print_self_check_report(out: &CheckResult) -> RunResult<()> {
    print!("{}", format_self_check_report(out, should_use_color())?);
    Ok(())
}

fn format_self_check_report(out: &CheckResult, use_color: bool) -> RunResult<String> {
    let relation =
        compare_versions_str(&out.current_version, &out.remote_version).map_err(|e| {
            RunReason::Exec.to_err().with_detail(format!(
                "compare self-update versions failed: current={}, remote={}, error={}",
                out.current_version, out.remote_version, e
            ))
        })?;

    let mut lines = vec![
        "Self-check result".to_string(),
        format!(
            "  Channel  : {}",
            render_self_update_channel(out.channel.as_str(), use_color)
        ),
        format!("  Current  : {}", out.current_version),
        format!(
            "  Remote   : {}",
            render_remote_version(&out.remote_version, relation, use_color)
        ),
        format!(
            "  Status   : {}",
            render_relation_message(relation, use_color)
        ),
    ];

    if relation == VersionRelation::UpdateAvailable {
        lines.push(format!(
            "  Action   : gx self update --channel {} --yes",
            out.channel.as_str()
        ));
    }

    Ok(format!("{}\n", lines.join("\n")))
}

fn render_self_update_channel(channel: &str, use_color: bool) -> String {
    if !use_color {
        return channel.to_string();
    }

    let code = match channel {
        "stable" => "32",
        "beta" => "33",
        "alpha" => "35",
        _ => return channel.to_string(),
    };
    format!("\x1b[{}m{}\x1b[0m", code, channel)
}

fn render_remote_version(version: &str, relation: VersionRelation, use_color: bool) -> String {
    if !use_color {
        return version.to_string();
    }

    match relation {
        VersionRelation::UpdateAvailable => format!("\x1b[1;92m{}\x1b[0m", version),
        VersionRelation::AheadOfChannel => format!("\x1b[90m{}\x1b[0m", version),
        VersionRelation::UpToDate => version.to_string(),
    }
}

fn render_relation_message(relation: VersionRelation, use_color: bool) -> String {
    let message = relation_message(relation);
    if !use_color {
        return message.to_string();
    }

    match relation {
        VersionRelation::UpdateAvailable => format!("\x1b[1;92m{}\x1b[0m", message),
        VersionRelation::AheadOfChannel => format!("\x1b[90m{}\x1b[0m", message),
        VersionRelation::UpToDate => format!("\x1b[32m{}\x1b[0m", message),
    }
}

fn should_use_color() -> bool {
    match std::env::var("TERM") {
        Ok(term) => term != "dumb",
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    use clap::Parser;

    use super::{
        DEFAULT_ADM_CONF, DEFAULT_WORK_CONF, OutputMode, collect_git_extern_mod_names_from_code,
        collect_mod_update_inputs, format_self_check_report, normalized_argv, output_mode,
    };
    use crate::cmd::gx_cmd::{GxCmd, SelfCheckArgs, SelfCmd};
    use crate::err::RunReason;
    use crate::self_update::{CheckResult, ReleaseChannel};

    fn mod_update_config_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct ConfigBackup {
        original: &'static str,
        backup: Option<std::path::PathBuf>,
    }

    impl ConfigBackup {
        fn hide_if_exists(original: &'static str) -> Self {
            let path = Path::new(original);
            let backup = path.exists().then(|| path.with_extension("gxl.bak-codex"));
            if let Some(backup_path) = &backup {
                std::fs::rename(path, backup_path).expect("config backup should succeed");
            }
            Self { original, backup }
        }

        fn restore(&mut self) {
            if let Some(backup_path) = self.backup.take() {
                std::fs::rename(backup_path, self.original).expect("config restore should succeed");
            }
        }
    }

    impl Drop for ConfigBackup {
        fn drop(&mut self) {
            self.restore();
        }
    }

    #[test]
    fn normalize_grun_to_run() {
        let args = normalized_argv(["grun", "conf"]);
        assert_eq!(args, vec!["grun", "run", "conf"]);
    }

    #[test]
    fn normalize_gadm_to_adm() {
        let args = normalized_argv(["gadm", "conf"]);
        assert_eq!(args, vec!["gadm", "adm", "conf"]);
    }

    #[test]
    fn collect_mod_update_inputs_errors_when_no_config_exists() {
        let _guard = mod_update_config_lock()
            .lock()
            .expect("mod update config lock should not be poisoned");
        let mut work_backup = ConfigBackup::hide_if_exists(DEFAULT_WORK_CONF);
        let mut adm_backup = ConfigBackup::hide_if_exists(DEFAULT_ADM_CONF);

        let err = collect_mod_update_inputs().expect_err("missing configs should fail");
        assert!(matches!(err.reason(), RunReason::Args));
        assert!(
            err.detail()
                .as_deref()
                .unwrap_or_default()
                .contains("expected at least one config file")
        );

        work_backup.restore();
        adm_backup.restore();
    }

    #[test]
    fn collect_mod_update_inputs_keeps_existing_configs() {
        let _guard = mod_update_config_lock()
            .lock()
            .expect("mod update config lock should not be poisoned");
        let inputs = collect_mod_update_inputs().expect("repo default configs should be used");
        let mut expected = Vec::new();
        if Path::new(DEFAULT_WORK_CONF).exists() {
            expected.push(DEFAULT_WORK_CONF);
        }
        if Path::new(DEFAULT_ADM_CONF).exists() {
            expected.push(DEFAULT_ADM_CONF);
        }
        assert_eq!(inputs, expected);
    }

    #[test]
    fn collect_git_extern_mod_names_from_code_keeps_only_git_mods() {
        let code = r#"
extern mod ver, git { git = "https://example.com/tooling.git", branch = "main" }
extern mod local_only { path = "./_gal/mods" }
extern mod cfm { git = "https://example.com/cfm.git", tag = "v1.0.0" }
mod main {}
"#;

        let mods = collect_git_extern_mod_names_from_code(code)
            .expect("git extern mods should be collected");
        assert_eq!(
            mods,
            vec!["cfm".to_string(), "git".to_string(), "ver".to_string()]
        );
    }

    #[test]
    fn output_mode_uses_machine_for_json_check() {
        let cmd = GxCmd::SelfUpdate(SelfCmd::Check(SelfCheckArgs {
            channel: "stable".to_string(),
            json: true,
        }));

        assert_eq!(output_mode(&cmd), OutputMode::Machine);
    }

    #[test]
    fn output_mode_uses_machine_for_quiet_run() {
        let cmd = GxCmd::parse_from(["gx", "run", "--quiet"]);

        assert_eq!(output_mode(&cmd), OutputMode::Machine);
    }

    #[test]
    fn output_mode_uses_machine_for_quiet_adm() {
        let cmd = GxCmd::parse_from(["gx", "adm", "--quiet"]);

        assert_eq!(output_mode(&cmd), OutputMode::Machine);
    }

    #[test]
    fn format_self_check_report_shows_update_action() {
        let report = CheckResult {
            channel: ReleaseChannel::Alpha,
            current_version: "0.13.9".to_string(),
            remote_version: "0.13.10-alpha.1".to_string(),
            has_update: true,
        };

        let rendered = format_self_check_report(&report, false).expect("report should render");

        assert!(rendered.contains("Self-check result"));
        assert!(rendered.contains("Channel  : alpha"));
        assert!(rendered.contains("Status   : update available"));
        assert!(rendered.contains("Action   : gx self update --channel alpha --yes"));
    }

    #[test]
    fn format_self_check_report_marks_ahead_of_channel() {
        let report = CheckResult {
            channel: ReleaseChannel::Alpha,
            current_version: "0.13.10".to_string(),
            remote_version: "0.13.9-alpha".to_string(),
            has_update: false,
        };

        let rendered = format_self_check_report(&report, false).expect("report should render");

        assert!(rendered.contains("Current  : 0.13.10"));
        assert!(rendered.contains("Remote   : 0.13.9-alpha"));
        assert!(rendered.contains("Status   : ahead of channel manifest"));
        assert!(!rendered.contains("Action   :"));
    }

    #[test]
    fn format_self_check_report_marks_up_to_date() {
        let report = CheckResult {
            channel: ReleaseChannel::Stable,
            current_version: "0.13.10".to_string(),
            remote_version: "0.13.10".to_string(),
            has_update: false,
        };

        let rendered = format_self_check_report(&report, false).expect("report should render");

        assert!(rendered.contains("Channel  : stable"));
        assert!(rendered.contains("Status   : up-to-date"));
        assert!(!rendered.contains("Action   :"));
    }
}
