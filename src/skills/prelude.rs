//! skills 模块公共导入。

pub use crate::err::{RunError, RunReason, RunResult};
pub use orion_error::conversion::{ErrorWith, ToStructError};
pub use orion_error::reason::UnifiedReason as UvsReason;
pub use std::fs;
pub use std::path::{Path, PathBuf};

/// 统一的资源类 IO 错误包装。
pub fn io_err(e: std::io::Error, what: &str) -> RunError {
    RunReason::from_res()
        .to_err()
        .with_detail(format!("{what}: {e}"))
}
