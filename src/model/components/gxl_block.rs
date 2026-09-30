use super::gxl_cond::GxlCond;
use super::gxl_loop::GxlLoop;
use super::gxl_spc::GxlSpace;
use super::gxl_var::GxlVar;
use super::prelude::*;
use async_trait::async_trait;
use derive_more::From;
use std::sync::mpsc::Sender;

use crate::ability::archive::GxTar;
use crate::ability::archive::GxUnTar;
use crate::ability::delegate::ActCall;
use crate::ability::patch::GxPatchFile;
use crate::ability::prelude::TaskValue;
use crate::ability::shell::GxShell;
use crate::ability::{
    GxAssert, GxCmd, GxDownLoad, GxEcho, GxRead, GxRun, GxTpl, GxUpLoad, GxlSn, GxlVersion,
};
use crate::calculate::cond::CondExec;
use crate::context::ExecContext;
use crate::execution::runnable::{AsyncRunnableWithSenderTrait, TaskResult};
use crate::execution::task::Task;
use crate::util::redirect::ReadSignal;

#[derive(Clone, From)]
pub enum BlockAction {
    Shell(GxShell),
    Command(GxCmd),
    GxlRun(GxRun),
    Cond(GxlCond),
    Loop(GxlLoop),
    Echo(GxEcho),
    Assert(GxAssert),
    Version(GxlVersion),
    Sn(GxlSn),
    Read(GxRead),
    Call(Box<ActCall>),
    Tpl(GxTpl),
    Tar(GxTar),
    UnTar(GxUnTar),
    DownLoad(GxDownLoad),
    UpLoad(GxUpLoad),
    PatchFile(GxPatchFile),
}

#[derive(Clone, Getters, Default)]
pub struct BlockNode {
    props: Vec<GxlVar>,
    items: Vec<BlockAction>,
}

impl BlockNode {
    pub fn new() -> Self {
        Self {
            props: vec![],
            items: vec![],
        }
    }
}

#[async_trait]
impl CondExec for BlockNode {
    async fn cond_exec(&self, ctx: ExecContext, def: VarSpace) -> TaskResult {
        self.async_exec(ctx, def, None).await
    }
}
#[async_trait]
impl AsyncRunnableWithSenderTrait for BlockAction {
    async fn async_exec(
        &self,
        ctx: ExecContext,
        dct: VarSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult {
        match self {
            BlockAction::GxlRun(o) => o.async_exec(ctx, dct, sender).await,
            BlockAction::Loop(o) => o.async_exec(ctx, dct, sender).await,
            BlockAction::Shell(o) => o.async_exec(ctx, dct).await,
            BlockAction::Command(o) => o.async_exec(ctx, dct).await,
            BlockAction::Echo(o) => o.async_exec(ctx, dct).await,
            BlockAction::Assert(o) => o.async_exec(ctx, dct).await,
            BlockAction::Cond(o) => o.async_exec(ctx, dct).await,
            BlockAction::Tpl(o) => o.async_exec(ctx, dct).await,
            BlockAction::Tar(o) => o.async_exec(ctx, dct).await,
            BlockAction::UnTar(o) => o.async_exec(ctx, dct).await,
            BlockAction::Call(o) => o.async_exec(ctx, dct).await,
            BlockAction::Version(o) => o.async_exec(ctx, dct).await,
            BlockAction::Sn(o) => o.async_exec(ctx, dct).await,
            BlockAction::Read(o) => o.async_exec(ctx, dct).await,
            BlockAction::UpLoad(o) => o.async_exec(ctx, dct).await,
            BlockAction::DownLoad(o) => o.async_exec(ctx, dct).await,
            BlockAction::PatchFile(o) => o.async_exec(ctx, dct).await,
        }
    }
}

#[async_trait]
impl AsyncRunnableWithSenderTrait for BlockNode {
    async fn async_exec(
        &self,
        ctx: ExecContext,
        var_dict: VarSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult {
        //ctx.append("block");
        let mut task = Task::from("block");
        let mut cur_var_dict = var_dict;
        self.export_props(ctx.clone(), cur_var_dict.global_mut(), "")?;

        for item in &self.items {
            let TaskValue { vars, rec } = item
                .async_exec(ctx.clone(), cur_var_dict, sender.clone())
                .await?;
            cur_var_dict = vars;
            // task.stdout.push_str(out.as_str());
            task.append(rec);
        }
        task.finish();
        Ok(TaskValue::from((cur_var_dict, ExecOut::Task(task))))
    }
}
impl DependTrait<&GxlSpace> for BlockNode {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        //let mut ins = BlockNode::default();
        //ins.props = self.props().clone();
        let mut ins = BlockNode {
            props: self.props().clone(),
            ..Default::default()
        };
        for x in self.items {
            let item = match x {
                BlockAction::Tpl(v) => BlockAction::Tpl(v.clone()),
                BlockAction::Tar(v) => BlockAction::Tar(v.clone()),
                BlockAction::UnTar(v) => BlockAction::UnTar(v.clone()),
                BlockAction::Cond(v) => BlockAction::Cond(v.clone()),
                BlockAction::Loop(v) => BlockAction::Loop(v.clone()),
                BlockAction::Read(v) => BlockAction::Read(v.clone()),
                BlockAction::Echo(v) => BlockAction::Echo(v.clone()),
                BlockAction::Assert(v) => BlockAction::Assert(v.clone()),
                BlockAction::Version(v) => BlockAction::Version(v.clone()),
                BlockAction::Sn(v) => BlockAction::Sn(v.clone()),
                BlockAction::Command(v) => BlockAction::Command(v.clone()),
                BlockAction::Shell(v) => BlockAction::Shell(v.clone()),
                BlockAction::GxlRun(v) => BlockAction::GxlRun(v.clone()),
                BlockAction::Call(v) => BlockAction::Call(Box::new(v.assemble(mod_name, src)?)),
                BlockAction::DownLoad(v) => BlockAction::DownLoad(v.clone()),
                BlockAction::UpLoad(v) => BlockAction::UpLoad(v.clone()),
                BlockAction::PatchFile(v) => BlockAction::PatchFile(v.clone()),
            };
            ins.append(item);
        }
        Ok(ins)
    }
}
impl PropsTrait for BlockNode {
    fn fetch_props(&self) -> Vec<GxlVar> {
        self.props.clone()
    }
}

impl AppendAble<GxlVar> for BlockNode {
    fn append(&mut self, prop: GxlVar) {
        self.props.push(prop);
    }
}
impl AppendAble<BlockAction> for BlockNode {
    fn append(&mut self, hold: BlockAction) {
        self.items.push(hold);
    }
}
impl AppendAble<Vec<BlockAction>> for BlockNode {
    fn append(&mut self, holds: Vec<BlockAction>) {
        for h in holds {
            self.items.push(h);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{cmd::GxlCmd, friendly::New2};
    use orion_sec::sec::{NoSecConv, SecFrom, SecValueObj, SecValueType, SecValueVec};
    use orion_variate::vars::UpperKey;

    //test RgBlock append
    use super::*;
    use crate::{model::components::gxl_block::BlockNode, traits::Getter, var::VarDict};
    #[test]
    fn test_append() {
        let mut block = BlockNode::new();
        let prop = GxlVar::new("test", "hello");
        block.append(prop);
        assert_eq!(block.props.len(), 1);
    }
    //test RgBlock exec method
    #[tokio::test]
    async fn test_exec() {
        let mut block = BlockNode::new();
        let prop = GxlVar::new("test", "hello");
        block.append(prop);
        let ctx = ExecContext::new(GxlCmd::default());
        let def = VarSpace::default();
        let res = block.async_exec(ctx, def, None).await;
        assert!(res.is_ok());
    }

    #[test]
    fn test_props_export_with_complex_data() {
        use crate::primitive::GxlObject;
        use orion_variate::vars::ValueType;

        // 创建测试数据
        let mut sys_a = SecValueObj::new();
        sys_a.insert(
            UpperKey::from("mod1"),
            SecValueType::nor_from("A".to_string()),
        );
        sys_a.insert(
            UpperKey::from("mod2"),
            SecValueType::nor_from("B".to_string()),
        );

        let sys_b = SecValueVec::from(vec![
            SecValueType::nor_from("C".to_string()),
            SecValueType::nor_from("D".to_string()),
        ]);

        // 创建 BlockNode 的 props
        let mut block = BlockNode::new();
        block.append(GxlVar::new(
            "sys_a",
            GxlObject::from(SecValueType::Obj(sys_a)),
        ));
        block.append(GxlVar::new(
            "sys_b",
            GxlObject::Value(SecValueType::List(sys_b)),
        ));
        block.append(GxlVar::new(
            "sys_c",
            GxlObject::VarRef("SYS_B[1]".to_string()),
        ));
        block.append(GxlVar::new(
            "sys_d",
            GxlObject::VarRef("SYS_A.MOD1".to_string()),
        ));

        // 创建执行上下文
        let ctx = ExecContext::new(GxlCmd::default());
        let mut var_dict = VarDict::default();

        // 导出 props
        block
            .export_props(ctx, &mut var_dict, "")
            .expect("Export props failed");

        // 验证导出的变量
        assert_eq!(
            var_dict.get_copy("SYS_A").unwrap().clone().no_sec(),
            ValueType::Obj(
                vec![
                    ("MOD1".to_string(), ValueType::String("A".to_string())),
                    ("MOD2".to_string(), ValueType::String("B".to_string()))
                ]
                .into_iter()
                .collect()
            )
        );

        assert_eq!(
            var_dict.get_copy("SYS_B").unwrap().clone().no_sec(),
            ValueType::from(vec![
                ValueType::String("C".to_string()),
                ValueType::String("D".to_string())
            ])
        );

        assert_eq!(
            var_dict.get_copy("SYS_C").unwrap().clone().no_sec(),
            ValueType::String("D".to_string())
        );

        assert_eq!(
            var_dict.get_copy("SYS_D").unwrap().clone().no_sec(),
            ValueType::String("A".to_string())
        );
    }
}
