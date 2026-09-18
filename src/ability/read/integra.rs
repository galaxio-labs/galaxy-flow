use async_trait::async_trait;
use derive_more::From;

use crate::{
    ExecReason,
    ability::prelude::{AsyncRunnableTrait, ComponentMeta, TaskResult, VarSpace},
    context::ExecContext,
    meta::GxlMeta,
};
use orion_error::conversion::ToStructError;

use super::{cmd::CmdDTO, file::FileDTO, stdin::StdinDTO};

#[derive(Debug, Default, Builder, PartialEq, Clone, Getters, From)]
pub struct GxRead {
    imp: ReadMode,
}
#[derive(Debug, PartialEq, Clone, Default, From)]
pub enum ReadMode {
    #[default]
    UNDEF,
    CMD(CmdDTO),
    FILE(FileDTO),
    STDIN(StdinDTO),
}

#[async_trait]
impl AsyncRunnableTrait for GxRead {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.execute_impl(ctx, vars_dict)
    }
}

impl ComponentMeta for GxRead {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.read")
    }
}
impl GxRead {
    fn execute_impl(&self, ctx: ExecContext, dict: VarSpace) -> TaskResult {
        match &self.imp {
            ReadMode::CMD(o) => o.execute(ctx, dict),
            ReadMode::FILE(o) => o.execute(ctx, dict),
            ReadMode::STDIN(o) => o.execute(ctx, dict),
            _ => Err(ExecReason::Gxl.to_err().with_detail("not implementation")),
        }
    }
}
