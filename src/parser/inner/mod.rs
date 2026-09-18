pub mod archive;
pub mod assert;
pub mod call;
pub mod cmd;
pub mod common;
pub mod funs;
pub mod gxl;
mod load;
pub mod patch;
pub mod read;
pub mod shell;
pub mod tpl;
pub mod ver;
pub use assert::gal_assert;
pub use cmd::gal_cmd;

// AI 能力已下线：ai_chat / ai_regst / ai_task 已隔离到 experimental/ai/src/parser/inner/，见 experimental/ai/README.md。
//pub mod ai_chat;
//pub mod ai_regst;
//pub mod ai_task;
pub use common::*;
pub use load::*;
pub use patch::*;
pub use read::*;
pub use tpl::gal_tpl;
pub use ver::*;
mod prelude;
