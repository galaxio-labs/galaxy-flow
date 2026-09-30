//! `gx self skill`：安装 / 列出 agent skills。
//!
//! 与 `gops self skill` 的语义对齐（远程浅 clone + 校验 `SKILL.md` frontmatter），
//! 让 `gx` 也能在脱离 `gops` 的环境下安装 gx skills。

mod installer;
mod model;
mod prelude;

pub use installer::{DEFAULT_COLLECTION_NAME, InstallRequest, SkillService};
pub use model::{ResolvedTarget, SkillInstallReport, SkillPlatform, SkillSource, SkillTarget};
