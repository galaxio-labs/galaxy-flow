// AI 能力已下线：整个 ai 模块已隔离到 experimental/ai/，见该目录下的 README。
// 恢复方式：git mv experimental/ai/src/ability/ai src/ability/ai，并还原下面这行。
//pub mod ai;
pub mod archive;
pub mod assert;
pub mod cmd;
pub mod delegate;
pub mod echo;
pub mod gxl;
pub mod load;
pub mod patch;
pub mod prelude;
pub mod read;
pub mod tpl;
//pub mod vault;
pub mod shell;
pub mod version;
use prelude::VarSpace;

use crate::cmd::GxlCmd;
use crate::const_val::gxl_const;
use crate::execution::global::detect_git_branch;
use crate::{ExecResult, context::ExecContext, infra::once_init_log, traits::Setter};

pub struct StubFlowAbi {}

#[allow(dead_code)]
pub fn ability_env_init() -> (ExecContext, VarSpace) {
    once_init_log();
    let context = ExecContext::new(GxlCmd::default());
    let cur_path = std::path::Path::new(context.cur_path());
    let mut def = VarSpace::default();
    def.global_mut()
        .set(gxl_const::PRJ_ROOT, context.cur_path().as_str());
    if let Some(branch) = detect_git_branch(cur_path) {
        def.global_mut().set(gxl_const::GIT_BRANCH, branch);
    } else {
        def.global_mut().set(gxl_const::GIT_BRANCH, "UNDEFIN");
    }
    (context, def)
}

pub fn sudo_cmd(sudo: bool) -> String {
    if sudo {
        "sudo".to_string()
    } else {
        "".to_string()
    }
}

pub fn parse_ok_codes(codes: &str) -> Vec<i32> {
    let codes: Vec<&str> = codes.split(',').collect();
    let mut parsed = Vec::new();
    for i in codes {
        let i = i.trim();
        let mut val = 0;
        if !i.is_empty() {
            val = i.parse::<i32>().unwrap_or_else(|_| panic!("bad number{i}"));
        }
        parsed.push(val);
    }
    parsed
}

pub use crate::ability::{
    assert::GxAssert,
    cmd::GxCmd,
    echo::GxEcho,
    read::GxRead,
    tpl::GxTpl,
    tpl::TplDTO,
    tpl::TplDTOBuilder,
    version::{GxlSn, GxlSnBuilder, GxlVersion, GxlVersionBuilder, SnAction},
};
use crate::evaluator::EnvExpress;

pub trait EnvRender {
    fn render(&mut self, exp: &EnvExpress) -> ExecResult<()>;
}

pub use gxl::GxRun;
pub use gxl::GxRunBuilder;

pub use load::{GxDownLoad, GxDownLoadBuilder, GxUpLoad, GxUpLoadBuilder};
