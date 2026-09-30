//! skills 安装：解析来源 → 校验 `SKILL.md` → 落地到 agent skills 目录。
//!
//! 与 `gops self skill` / `install.sh` 语义对齐：远程来源浅 clone，安装前逐个校验
//! `SKILL.md` 的 YAML frontmatter，非法即中止、不落盘。

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use walkdir::WalkDir;

use super::model::{ResolvedTarget, SkillInstallReport, SkillPlatform, SkillSource, SkillTarget};
use super::prelude::*;

/// 整包安装时无法从来源推导名字时的兜底目标名。
pub const DEFAULT_COLLECTION_NAME: &str = "gx-skills";

/// 安装请求。
#[derive(Clone, Debug)]
pub struct InstallRequest {
    pub source: SkillSource,
    /// `skills/` 下的单个 skill 名；`None` 表示安装整包。
    pub skill: Option<String>,
    /// 显式目标；为空则自动探测已存在的平台目录。
    pub targets: Vec<SkillTarget>,
    /// 用符号链接替代复制（仅本地来源）。
    pub symlink: bool,
    /// 目标已存在时是否跳过确认。
    pub yes: bool,
}

/// skills 安装服务。
#[derive(Clone, Debug)]
pub struct SkillService {
    home: PathBuf,
}

impl SkillService {
    pub fn new() -> RunResult<Self> {
        let home = home::home_dir().ok_or_else(|| {
            RunReason::from_res()
                .to_err()
                .with_detail("cannot resolve home directory: skill install needs home dir")
        })?;
        Ok(Self { home })
    }

    #[cfg(test)]
    pub(crate) fn with_home(home: PathBuf) -> Self {
        Self { home }
    }

    /// 列出来源仓库 `skills/` 下可安装的 skill 名。
    pub fn list(&self, source: &SkillSource) -> RunResult<Vec<String>> {
        let materialized = self.materialize(source)?;
        let skills_dir = materialized.path().join("skills");
        let mut names = Vec::new();
        if skills_dir.is_dir() {
            for entry in fs::read_dir(&skills_dir).map_err(|e| io_err(e, "read skills dir"))? {
                let entry = entry.map_err(|e| io_err(e, "read skills entry"))?;
                let path = entry.path();
                if path.join("SKILL.md").is_file()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    /// 安装 skills。
    pub fn install(&self, req: &InstallRequest) -> RunResult<SkillInstallReport> {
        if req.symlink && !req.source.is_local() {
            return Err(RunReason::Args
                .to_err()
                .with_detail("--symlink 仅支持本地来源（--source 指向已存在的目录）"));
        }

        let materialized = self.materialize(&req.source)?;
        let root = materialized.path();
        let src_dir = match &req.skill {
            Some(name) => resolve_skill_dir(root, name)?,
            None => root.to_path_buf(),
        };

        let skill_files = validate_skills(&src_dir)?;
        let name = req
            .skill
            .clone()
            .unwrap_or_else(|| collection_name(&req.source));

        let targets = self.resolve_targets(req)?;
        confirm_overwrite(&targets, &name, req.yes)?;

        let mut installed = Vec::new();
        for target in &targets {
            let dst = target.dir.join(&name);
            fs::create_dir_all(&target.dir).map_err(|e| io_err(e, "create skills dir"))?;
            if dst.exists() {
                fs::remove_dir_all(&dst).map_err(|e| io_err(e, "remove existing skill dir"))?;
            }
            if req.symlink {
                symlink_dir(&src_dir, &dst)?;
            } else {
                copy_dir(&src_dir, &dst)?;
                let git_dir = dst.join(".git");
                if git_dir.exists() {
                    fs::remove_dir_all(&git_dir)
                        .map_err(|e| io_err(e, "strip vcs metadata from installed skill"))?;
                }
            }
            installed.push(ResolvedTarget {
                platform: target.platform.clone(),
                dir: dst,
            });
        }

        Ok(SkillInstallReport {
            name,
            source_dir: src_dir,
            skill_files,
            installed,
        })
    }

    /// 把来源物化为本地目录（远程来源做浅 clone 到临时目录）。
    fn materialize(&self, source: &SkillSource) -> RunResult<Materialized> {
        match source {
            SkillSource::Local { path } => {
                if !path.is_dir() {
                    return Err(RunReason::from_res().to_err().with_detail(format!(
                        "local skill source is not a directory: {}",
                        path.display()
                    )));
                }
                Ok(Materialized {
                    path: path.clone(),
                    _temp: None,
                })
            }
            SkillSource::Remote { url, git_ref } => {
                let temp = TempDir::new()?;
                clone_shallow(url, git_ref, temp.path())?;
                let path = temp.path().to_path_buf();
                Ok(Materialized {
                    path,
                    _temp: Some(temp),
                })
            }
        }
    }

    /// 解析安装目标：显式 `--target` / `--dir` 优先；否则探测已存在的平台目录，
    /// 都不存在时回退到三个平台（与 install.sh 一致）。
    fn resolve_targets(&self, req: &InstallRequest) -> RunResult<Vec<ResolvedTarget>> {
        let mut resolved = Vec::new();
        let mut seen: HashSet<PathBuf> = HashSet::new();

        if req.targets.is_empty() {
            let existing: Vec<SkillPlatform> = SkillPlatform::ALL
                .into_iter()
                .filter(|p| p.dir_in(&self.home).is_dir())
                .collect();
            let chosen = if existing.is_empty() {
                SkillPlatform::ALL.to_vec()
            } else {
                existing
            };
            for platform in chosen {
                add_target(
                    &mut resolved,
                    &mut seen,
                    platform.display_name().to_string(),
                    platform.dir_in(&self.home),
                );
            }
        } else {
            for target in &req.targets {
                match target {
                    SkillTarget::Platform(platform) => add_target(
                        &mut resolved,
                        &mut seen,
                        platform.display_name().to_string(),
                        platform.dir_in(&self.home),
                    ),
                    SkillTarget::Dir(dir) => {
                        add_target(&mut resolved, &mut seen, "custom".to_string(), dir.clone())
                    }
                }
            }
        }
        Ok(resolved)
    }
}

fn add_target(
    resolved: &mut Vec<ResolvedTarget>,
    seen: &mut HashSet<PathBuf>,
    platform: String,
    dir: PathBuf,
) {
    if seen.insert(dir.clone()) {
        resolved.push(ResolvedTarget { platform, dir });
    }
}

/// 目标已存在时的覆盖确认：`--yes` 或非交互环境直接覆盖（与 install.sh 一致）。
fn confirm_overwrite(targets: &[ResolvedTarget], name: &str, yes: bool) -> RunResult<()> {
    let existing: Vec<PathBuf> = targets
        .iter()
        .map(|t| t.dir.join(name))
        .filter(|path| path.exists())
        .collect();
    if existing.is_empty() || yes {
        return Ok(());
    }
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return Ok(());
    }
    println!("以下目录已存在，将被替换：");
    for path in &existing {
        println!("  - {}", path.display());
    }
    print!("继续？[y/N] ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| io_err(e, "read confirm input"))?;
    if matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        Ok(())
    } else {
        Err(RunReason::Args.to_err().with_detail("aborted by user"))
    }
}

/// 整包安装时的目标名：取来源仓库名（远程 URL 的最后一段 / 本地目录 basename）。
fn collection_name(source: &SkillSource) -> String {
    let name = match source {
        SkillSource::Remote { url, .. } => {
            let trimmed = url.trim_end_matches('/').trim_end_matches(".git");
            trimmed.rsplit(['/', ':']).next().unwrap_or("").to_string()
        }
        SkillSource::Local { path } => path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
    };
    if name.trim().is_empty() {
        DEFAULT_COLLECTION_NAME.to_string()
    } else {
        name
    }
}

/// 定位单个 skill 目录（`<root>/skills/<name>`）。
fn resolve_skill_dir(root: &Path, name: &str) -> RunResult<PathBuf> {
    let dir = root.join("skills").join(name);
    if dir.is_dir() {
        return Ok(dir);
    }
    let mut available = Vec::new();
    if let Ok(entries) = fs::read_dir(root.join("skills")) {
        for entry in entries.flatten() {
            if entry.path().join("SKILL.md").is_file()
                && let Some(n) = entry.file_name().to_str()
            {
                available.push(n.to_string());
            }
        }
    }
    available.sort();
    let available = if available.is_empty() {
        "none".to_string()
    } else {
        available.join(", ")
    };
    Err(RunReason::from_res()
        .to_err()
        .with_detail(format!("skill not found: {name} (available: {available})")))
}

/// 校验 `src_dir` 下所有 `SKILL.md` 的 frontmatter，返回校验通过的文件列表。
fn validate_skills(src_dir: &Path) -> RunResult<Vec<PathBuf>> {
    let files = collect_skill_files(src_dir);
    if files.is_empty() {
        return Err(RunReason::from_res()
            .to_err()
            .with_detail(format!("no SKILL.md found under {}", src_dir.display())));
    }
    let mut errors = Vec::new();
    for file in &files {
        if let Err(reason) = validate_frontmatter(file) {
            errors.push(format!("{}: {reason}", file.display()));
        }
    }
    if !errors.is_empty() {
        return Err(RunReason::Uvs(UvsReason::validation_error())
            .to_err()
            .with_detail(format!(
                "invalid SKILL.md frontmatter:\n  - {}",
                errors.join("\n  - ")
            )));
    }
    Ok(files)
}

/// 递归收集 `src_dir` 下的 `SKILL.md`（跳过 `.git`）。
fn collect_skill_files(src_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = WalkDir::new(src_dir)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git");
    for entry in walker.filter_map(Result::ok) {
        if entry.file_type().is_file() && entry.file_name() == "SKILL.md" {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    files
}

/// 校验单个 `SKILL.md`：必须有 YAML frontmatter，且 `name` / `description` 非空。
fn validate_frontmatter(path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let frontmatter = extract_frontmatter(&text)
        .ok_or_else(|| "missing YAML frontmatter (expected a leading `---` block)".to_string())?;
    let value: serde_yaml::Value =
        serde_yaml::from_str(&frontmatter).map_err(|e| format!("invalid YAML: {e}"))?;
    let map = value
        .as_mapping()
        .ok_or_else(|| "frontmatter is not a YAML mapping".to_string())?;
    for key in ["name", "description"] {
        let present = map
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !present {
            return Err(format!("missing or empty `{key}`"));
        }
    }
    Ok(())
}

/// 取首个 `---` 与第二个 `---` 之间的 frontmatter。
fn extract_frontmatter(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| l.trim() == "---")? + 1;
    Some(lines[1..end].join("\n"))
}

/// 递归复制目录（跳过 `.git`）。
fn copy_dir(src: &Path, dst: &Path) -> RunResult<()> {
    fs::create_dir_all(dst).map_err(|e| io_err(e, "create skill dir"))?;
    for entry in fs::read_dir(src).map_err(|e| io_err(e, "read skill source dir"))? {
        let entry = entry.map_err(|e| io_err(e, "read skill source entry"))?;
        let from = entry.path();
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let to = dst.join(&name);
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| io_err(e, "copy skill file"))?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn symlink_dir(src: &Path, dst: &Path) -> RunResult<()> {
    std::os::unix::fs::symlink(src, dst).map_err(|e| io_err(e, "symlink skill dir"))
}

#[cfg(windows)]
fn symlink_dir(src: &Path, dst: &Path) -> RunResult<()> {
    std::os::windows::fs::symlink_dir(src, dst).map_err(|e| io_err(e, "symlink skill dir"))
}

/// 浅 clone；指定 ref 失败时回退到默认分支（等价 install.sh 的 `git clone` 兜底）。
fn clone_shallow(url: &str, git_ref: &str, dest: &Path) -> RunResult<()> {
    let git_ref = git_ref.trim();
    if git_ref.is_empty() {
        return clone_once(url, None, dest).map_err(|e| clone_error(url, "", &e));
    }
    match clone_once(url, Some(git_ref), dest) {
        Ok(()) => Ok(()),
        Err(first) => {
            let _ = fs::remove_dir_all(dest);
            clone_once(url, None, dest).map_err(|_| clone_error(url, git_ref, &first))
        }
    }
}

fn clone_once(url: &str, branch: Option<&str>, dest: &Path) -> Result<(), git2::Error> {
    let mut fetch = git2::FetchOptions::new();
    fetch.depth(1);
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch);
    if let Some(branch) = branch {
        builder.branch(branch);
    }
    builder.clone(url, dest)?;
    Ok(())
}

fn clone_error(url: &str, git_ref: &str, e: &git2::Error) -> RunError {
    RunReason::from_res().to_err().with_detail(format!(
        "git clone failed: url={url}, ref={git_ref}, error={e}"
    ))
}

/// 来源物化结果：`_temp` 存在时随作用域结束删除临时目录。
struct Materialized {
    path: PathBuf,
    _temp: Option<TempDir>,
}

impl Materialized {
    fn path(&self) -> &Path {
        &self.path
    }
}

/// 临时目录；`Drop` 时递归删除。
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> RunResult<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("gx-skill-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).map_err(|e| io_err(e, "create skill temp dir"))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_COLLECTION_NAME, InstallRequest, SkillService, collect_skill_files,
        collection_name, extract_frontmatter, validate_frontmatter, validate_skills,
    };
    use crate::skills::model::{SkillSource, SkillTarget};
    use std::path::Path;

    fn write_skill(dir: &Path, name: &str, description: &str) {
        std::fs::create_dir_all(dir).expect("mk skill dir");
        std::fs::write(
            dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n"),
        )
        .expect("write SKILL.md");
    }

    fn fake_repo(root: &Path) {
        write_skill(root, "gx-skills", "collection router");
        write_skill(&root.join("skills/foo"), "foo", "the foo skill");
        std::fs::create_dir_all(root.join(".git")).expect("mk git");
        std::fs::write(root.join(".git/config"), "junk").expect("write git config");
    }

    fn local_req(root: &Path, skill: Option<&str>, target: &Path, symlink: bool) -> InstallRequest {
        InstallRequest {
            source: SkillSource::Local {
                path: root.to_path_buf(),
            },
            skill: skill.map(|s| s.to_string()),
            targets: vec![SkillTarget::Dir(target.to_path_buf())],
            symlink,
            yes: true,
        }
    }

    #[test]
    fn extract_frontmatter_basic() {
        assert_eq!(
            extract_frontmatter("---\nname: a\n---\nbody\n").as_deref(),
            Some("name: a")
        );
        assert_eq!(extract_frontmatter("# no frontmatter\n"), None);
    }

    #[test]
    fn validate_skills_requires_at_least_one() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        assert!(validate_skills(tmp.path()).is_err());
        fake_repo(tmp.path());
        assert_eq!(validate_skills(tmp.path()).expect("validate").len(), 2);
    }

    #[test]
    fn validate_frontmatter_rejects_missing_field() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let bad = tmp.path().join("bad/SKILL.md");
        std::fs::create_dir_all(tmp.path().join("bad")).expect("mk");
        std::fs::write(&bad, "---\nname: bad\n---\n").expect("write");
        assert!(validate_frontmatter(&bad).is_err());
    }

    #[test]
    fn collect_skill_files_skips_git() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        fake_repo(tmp.path());
        let files = collect_skill_files(tmp.path());
        assert!(!files.is_empty());
        assert!(
            files
                .iter()
                .all(|f| !f.to_string_lossy().contains("/.git/"))
        );
    }

    #[test]
    fn install_single_skill_copies_and_strips_git() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let repo = tmp.path().join("repo");
        fake_repo(&repo);
        let target = tmp.path().join("target");
        let svc = SkillService::with_home(tmp.path().join("home"));

        let report = svc
            .install(&local_req(&repo, Some("foo"), &target, false))
            .expect("install");

        assert_eq!(report.name, "foo");
        assert!(target.join("foo/SKILL.md").is_file());
        assert!(!target.join("foo/.git").exists());
        assert_eq!(report.installed[0].platform, "custom");
    }

    #[test]
    fn install_collection_name_derives_from_source_repo() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let repo = tmp.path().join("gx-skills");
        fake_repo(&repo);
        let target = tmp.path().join("target");
        let svc = SkillService::with_home(tmp.path().join("home"));

        let report = svc
            .install(&local_req(&repo, None, &target, false))
            .expect("install");

        assert_eq!(report.name, "gx-skills");
        assert!(target.join("gx-skills").join("SKILL.md").is_file());
    }

    #[test]
    fn collection_name_derives_from_source() {
        let remote = |url: &str| SkillSource::Remote {
            url: url.to_string(),
            git_ref: "main".to_string(),
        };
        assert_eq!(
            collection_name(&remote("https://github.com/galaxio-labs/gx-skills.git")),
            "gx-skills"
        );
        assert_eq!(
            collection_name(&remote("git@github.com:galaxio-labs/gops-skills.git")),
            "gops-skills"
        );
        assert_eq!(
            collection_name(&SkillSource::Local {
                path: std::path::PathBuf::from("/tmp/a/gops-skills")
            }),
            "gops-skills"
        );
        assert_eq!(DEFAULT_COLLECTION_NAME, "gx-skills");
    }

    #[test]
    fn install_symlink_rejected_for_remote_source() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let svc = SkillService::with_home(tmp.path().join("home"));
        let req = InstallRequest {
            source: SkillSource::Remote {
                url: "https://github.com/a/b.git".to_string(),
                git_ref: "main".to_string(),
            },
            skill: None,
            targets: vec![SkillTarget::Dir(tmp.path().join("target"))],
            symlink: true,
            yes: true,
        };
        assert!(svc.install(&req).is_err());
    }

    #[test]
    fn list_returns_local_skill_names_sorted() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let repo = tmp.path().join("repo");
        write_skill(&repo.join("skills/zeta"), "zeta", "z");
        write_skill(&repo.join("skills/alpha"), "alpha", "a");
        let svc = SkillService::with_home(tmp.path().join("home"));
        let names = svc.list(&SkillSource::Local { path: repo }).expect("list");
        assert_eq!(names, vec!["alpha".to_string(), "zeta".to_string()]);
    }
}
