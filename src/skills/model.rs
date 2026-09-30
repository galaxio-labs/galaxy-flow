//! skills 安装相关的数据类型。

use super::prelude::*;

/// 技能来源：远程 git 仓库，或本地目录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkillSource {
    /// 远程仓库：`url` 为规范化后的 git 地址，`git_ref` 为分支 / 标签。
    Remote { url: String, git_ref: String },
    /// 本地 checkout 目录（可用于 `--symlink`）。
    Local { path: PathBuf },
}

impl SkillSource {
    /// 解析 `--source` / `--ref`。
    ///
    /// - 已存在的路径，或以 `.` / `/` / `~` 开头 → 本地目录；
    /// - 其余按 GitHub `owner/repo`（或完整 git URL）处理。
    pub fn parse(source: &str, git_ref: &str) -> RunResult<Self> {
        let source = source.trim();
        if source.is_empty() {
            return Err(RunReason::Args
                .to_err()
                .with_detail("--source must not be empty"));
        }
        if is_local_source(source) {
            return Ok(SkillSource::Local {
                path: expand_tilde(source),
            });
        }
        Ok(SkillSource::Remote {
            url: normalize_git_url(source),
            git_ref: git_ref.trim().to_string(),
        })
    }

    pub fn is_local(&self) -> bool {
        matches!(self, SkillSource::Local { .. })
    }

    /// 人类可读的来源描述。
    pub fn describe(&self) -> String {
        match self {
            SkillSource::Remote { url, git_ref } => {
                if git_ref.is_empty() {
                    url.clone()
                } else {
                    format!("{url}@{git_ref}")
                }
            }
            SkillSource::Local { path } => path.display().to_string(),
        }
    }
}

/// 支持的 agent 平台（与 `install.sh` 的 `--codex/--claude/--zed` 对齐）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillPlatform {
    Codex,
    Claude,
    Zed,
}

impl SkillPlatform {
    pub const ALL: [SkillPlatform; 3] = [Self::Codex, Self::Claude, Self::Zed];

    /// 展示用平台名。
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude-code",
            Self::Zed => "zed",
        }
    }

    /// 解析 `--target` 取值。
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "codex" => Some(Self::Codex),
            "claude" | "claude-code" | "claudecode" => Some(Self::Claude),
            "zed" | "agents" => Some(Self::Zed),
            _ => None,
        }
    }

    /// 该平台的默认技能目录：`<home>/<…>/skills`。
    pub fn dir_in(self, home: &Path) -> PathBuf {
        match self {
            Self::Codex => home.join(".codex").join("skills"),
            Self::Claude => home.join(".claude").join("skills"),
            Self::Zed => home.join(".agents").join("skills"),
        }
    }
}

/// 安装目标：预置平台，或自定义目录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkillTarget {
    Platform(SkillPlatform),
    Dir(PathBuf),
}

/// 解析后的安装目标。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTarget {
    pub platform: String,
    pub dir: PathBuf,
}

/// 安装结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillInstallReport {
    /// 安装名：单个 skill 时为 skill 名，整包时为来源仓库名。
    pub name: String,
    /// 实际安装来源目录（本地目录，或 clone 出来的临时目录）。
    pub source_dir: PathBuf,
    /// 校验通过的 `SKILL.md` 列表。
    pub skill_files: Vec<PathBuf>,
    /// 每个目标目录的落地位置。
    pub installed: Vec<ResolvedTarget>,
}

fn is_local_source(source: &str) -> bool {
    source.starts_with('.')
        || source.starts_with('/')
        || source.starts_with('~')
        || Path::new(source).is_dir()
}

fn expand_tilde(source: &str) -> PathBuf {
    if let Some(rest) = source.strip_prefix("~/")
        && let Some(home) = home::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(source)
}

fn normalize_git_url(source: &str) -> String {
    if source.contains("://") || source.starts_with("git@") {
        source.to_string()
    } else {
        format!("https://github.com/{source}.git")
    }
}

#[cfg(test)]
mod tests {
    use super::{SkillPlatform, SkillSource, normalize_git_url};
    use std::path::PathBuf;

    #[test]
    fn parse_source_owner_repo_is_remote_github() {
        let source = SkillSource::parse("galaxio-labs/gx-skills", "main").expect("parse");
        assert_eq!(
            source,
            SkillSource::Remote {
                url: "https://github.com/galaxio-labs/gx-skills.git".to_string(),
                git_ref: "main".to_string(),
            }
        );
        assert!(!source.is_local());
    }

    #[test]
    fn parse_source_dot_slash_is_local() {
        let source = SkillSource::parse("./checkout", "main").expect("parse");
        assert!(source.is_local());
        assert_eq!(
            source,
            SkillSource::Local {
                path: PathBuf::from("./checkout")
            }
        );
    }

    #[test]
    fn parse_source_rejects_empty() {
        assert!(SkillSource::parse("   ", "main").is_err());
    }

    #[test]
    fn normalize_url_leaves_git_and_ssh() {
        assert_eq!(
            normalize_git_url("git@github.com:a/b.git"),
            "git@github.com:a/b.git"
        );
        assert_eq!(normalize_git_url("a/b"), "https://github.com/a/b.git");
    }

    #[test]
    fn platform_parse_aliases() {
        assert_eq!(SkillPlatform::parse("codex"), Some(SkillPlatform::Codex));
        assert_eq!(
            SkillPlatform::parse("claude-code"),
            Some(SkillPlatform::Claude)
        );
        assert_eq!(SkillPlatform::parse("agents"), Some(SkillPlatform::Zed));
        assert_eq!(SkillPlatform::parse("vscode"), None);
    }

    #[test]
    fn platform_dirs_match_install_sh() {
        let home = PathBuf::from("/home/u");
        assert_eq!(
            SkillPlatform::Zed.dir_in(&home),
            PathBuf::from("/home/u/.agents/skills")
        );
    }
}
