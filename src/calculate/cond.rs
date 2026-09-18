use async_trait::async_trait;

use super::express::ExpressEnum;
use crate::ExecReason;
use crate::ability::prelude::{TaskResult, TaskValue, VarSpace};
use crate::calculate::traits::Evaluation;
use crate::components::gxl_cond::TGxlCond;
use crate::context::ExecContext;
use crate::execution::runnable::ExecOut;
use orion_error::conversion::ToStructError;
use std::sync::Arc;
#[async_trait]
pub trait CondExec {
    async fn cond_exec(&self, ctx: ExecContext, def: VarSpace) -> TaskResult;
}

pub type CondHandle = Arc<dyn CondExec>;

#[derive(Clone, Debug, Getters)]
pub struct IFExpress<T> {
    express: ExpressEnum,
    true_block: T,
    elseif_blocks: Vec<TGxlCond<T>>,
    false_block: Option<T>,
}
#[async_trait]
impl<T> CondExec for IFExpress<T>
where
    T: CondExec + std::marker::Sync,
{
    async fn cond_exec(&self, ctx: ExecContext, def: VarSpace) -> TaskResult {
        let x = self.express.decide(ctx.clone(), &def).map_err(|err| {
            ExecReason::logic_error()
                .to_err()
                .with_detail(err.to_string())
        })?;
        if x {
            self.true_block.cond_exec(ctx, def).await
        } else {
            for cond in &self.elseif_blocks {
                if let Ok(task_value) = cond.cond.cond_exec(ctx.clone(), def.clone()).await
                    && task_value.rec() != &ExecOut::Ignore
                {
                    return Ok(task_value);
                }
            }
            if let Some(false_cond) = self.false_block.as_ref() {
                return false_cond.cond_exec(ctx, def).await;
            }
            Ok(TaskValue::from((def, ExecOut::Ignore)))
        }
    }
}
pub struct StuBlock {
    pub out: ExecOut,
}
#[async_trait]
impl CondExec for StuBlock {
    async fn cond_exec(&self, _ctx: ExecContext, _def: VarSpace) -> TaskResult {
        Ok(TaskValue::from((_def, self.out.clone())))
    }
}

impl<T> IFExpress<T>
where
    T: CondExec,
{
    pub(crate) fn new(
        express: ExpressEnum,
        true_block: T,
        elseif_blocks: Vec<TGxlCond<T>>,
        false_block: Option<T>,
    ) -> Self {
        Self {
            express,
            true_block,
            elseif_blocks,
            false_block,
        }
    }
}

#[cfg(test)]
mod tests {
    use orion_sec::sec::{SecFrom, SecValueType};

    use super::{CondExec, IFExpress, StuBlock};
    use crate::ability::prelude::{TaskResult, TaskValue, VarSpace};
    use crate::calculate::CmpExpress;
    use crate::calculate::express::ExpressEnum;
    use crate::components::gxl_cond::TGxlCond;
    use crate::context::ExecContext;
    use crate::execution::runnable::ExecOut;
    use crate::primitive::GxlObject;
    use crate::traits::Setter;

    // 创建一个简单的测试块实现 CondExec trait
    #[derive(Clone)]
    struct TestBlock {
        expected_out: ExecOut,
    }

    impl TestBlock {
        fn new(_name: &str, expected_out: ExecOut) -> Self {
            Self { expected_out }
        }
    }

    #[async_trait::async_trait]
    impl CondExec for TestBlock {
        async fn cond_exec(&self, _ctx: ExecContext, def: VarSpace) -> TaskResult {
            Ok(TaskValue::from((def, self.expected_out.clone())))
        }
    }

    // 创建一个总是返回 true 的表达式
    fn create_true_expr() -> ExpressEnum {
        ExpressEnum::Cmp(CmpExpress::eq(
            GxlObject::Value(SecValueType::nor_from("true".to_string())),
            GxlObject::Value(SecValueType::nor_from("true".to_string())),
        ))
    }

    // 创建一个总是返回 false 的表达式
    fn create_false_expr() -> ExpressEnum {
        ExpressEnum::Cmp(CmpExpress::eq(
            GxlObject::Value(SecValueType::nor_from("true".to_string())),
            GxlObject::Value(SecValueType::nor_from("false".to_string())),
        ))
    }

    // 创建一个基于变量值的表达式
    fn create_var_expr(var_name: &str, expected: &str) -> ExpressEnum {
        ExpressEnum::Cmp(CmpExpress::eq(
            GxlObject::VarRef(var_name.to_string()),
            GxlObject::Value(SecValueType::nor_from(expected.to_string())),
        ))
    }

    #[tokio::test]
    async fn test_if_express_true_branch() {
        let true_block = TestBlock::new("true_branch", ExecOut::Ignore);
        let false_block = TestBlock::new("false_branch", ExecOut::Code(1));

        let if_express = IFExpress::new(
            create_true_expr(),
            true_block.clone(),
            vec![],
            Some(false_block),
        );

        let ctx = ExecContext::default();
        let def = VarSpace::default();

        let result = if_express.cond_exec(ctx, def).await.unwrap();
        assert_eq!(result.rec(), &ExecOut::Ignore);
    }

    #[tokio::test]
    async fn test_if_express_false_branch() {
        let true_block = TestBlock::new("true_branch", ExecOut::Ignore);
        let false_block = TestBlock::new("false_branch", ExecOut::Code(1));

        let if_express = IFExpress::new(
            create_false_expr(),
            true_block,
            vec![],
            Some(false_block.clone()),
        );

        let ctx = ExecContext::default();
        let def = VarSpace::default();

        let result = if_express.cond_exec(ctx, def).await.unwrap();
        assert_eq!(result.rec(), &ExecOut::Code(1));
    }

    #[tokio::test]
    async fn test_if_express_no_false_branch() {
        let true_block = TestBlock::new("true_branch", ExecOut::Ignore);

        let if_express = IFExpress::new(create_false_expr(), true_block, vec![], None);

        let ctx = ExecContext::default();
        let def = VarSpace::default();

        let result = if_express.cond_exec(ctx, def).await.unwrap();
        assert_eq!(result.rec(), &ExecOut::Ignore);
    }

    #[tokio::test]
    async fn test_if_express_with_elseif_first_match() {
        let true_block = TestBlock::new("true_branch", ExecOut::Ignore);
        let false_block = TestBlock::new("false_branch", ExecOut::Code(1));
        let elseif1_block = TestBlock::new("elseif1", ExecOut::Code(2));

        // 创建返回 true 的 elseif 条件
        let elseif1 = TGxlCond {
            cond: IFExpress::new(create_true_expr(), elseif1_block.clone(), vec![], None),
        };

        let if_express = IFExpress::new(
            create_false_expr(),
            true_block,
            vec![elseif1],
            Some(false_block),
        );

        let ctx = ExecContext::default();
        let def = VarSpace::default();

        let result = if_express.cond_exec(ctx, def).await.unwrap();
        assert_eq!(result.rec(), &ExecOut::Code(2));
    }

    #[tokio::test]
    async fn test_if_express_with_elseif_no_match() {
        let true_block = TestBlock::new("true_branch", ExecOut::Ignore);
        let false_block = TestBlock::new("false_branch", ExecOut::Code(1));
        let elseif1_block = TestBlock::new("elseif1", ExecOut::Code(2));

        // 创建返回 false 的 elseif 条件
        let elseif1 = TGxlCond {
            cond: IFExpress::new(create_false_expr(), elseif1_block, vec![], None),
        };

        let if_express = IFExpress::new(
            create_false_expr(),
            true_block,
            vec![elseif1],
            Some(false_block.clone()),
        );

        let ctx = ExecContext::default();
        let def = VarSpace::default();

        let result = if_express.cond_exec(ctx, def).await.unwrap();
        assert_eq!(result.rec(), &ExecOut::Code(1));
    }

    #[tokio::test]
    async fn test_if_express_with_variable_condition() {
        let true_block = TestBlock::new("true_branch", ExecOut::Ignore);
        let false_block = TestBlock::new("false_block", ExecOut::Code(1));

        let mut vars = VarSpace::default();
        vars.global_mut().set("test_var", "expected_value");

        let if_express = IFExpress::new(
            create_var_expr("test_var", "expected_value"),
            true_block.clone(),
            vec![],
            Some(false_block),
        );

        let ctx = ExecContext::default();

        let result = if_express.cond_exec(ctx, vars).await.unwrap();
        assert_eq!(result.rec(), &ExecOut::Ignore);
    }

    #[tokio::test]
    async fn test_stu_block() {
        let stu_block = StuBlock {
            out: ExecOut::Ignore,
        };

        let ctx = ExecContext::default();
        let def = VarSpace::default();

        let result = stu_block.cond_exec(ctx, def).await.unwrap();
        assert_eq!(result.rec(), &ExecOut::Ignore);
    }

    #[tokio::test]
    async fn test_stu_block_different_outputs() {
        let outputs = vec![
            ExecOut::Ignore,
            ExecOut::Code(0),
            ExecOut::Code(1),
            ExecOut::Code(2),
        ];

        for out in outputs {
            let stu_block = StuBlock { out: out.clone() };
            let ctx = ExecContext::default();
            let def = VarSpace::default();

            let result = stu_block.cond_exec(ctx, def).await.unwrap();
            assert_eq!(result.rec(), &out);
        }
    }
}
