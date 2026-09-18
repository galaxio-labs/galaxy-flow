use std::sync::mpsc::Sender;

use orion_error::conversion::ToStructError;
use orion_sec::sec::SecValueType;

use super::prelude::*;
use crate::{
    ability::prelude::TaskValue,
    execution::{runnable::AsyncRunnableWithSenderTrait, task::Task},
    traits::Setter,
    util::redirect::ReadSignal,
};

use super::gxl_block::BlockNode;

#[derive(Clone, Getters)]
pub struct GxlLoop {
    cur_name: String,
    var_name: String,
    body: BlockNode,
}

impl GxlLoop {
    pub fn new(cur_name: String, dct_name: String, body: BlockNode) -> Self {
        Self {
            cur_name,
            var_name: dct_name,
            body,
        }
    }
}

#[async_trait]
impl AsyncRunnableWithSenderTrait for GxlLoop {
    async fn async_exec(
        &self,
        ctx: ExecContext,
        dict: VarSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult {
        let mut task = Task::from("loop");
        let mut cur_dict = dict.clone();
        if let Some(found) = dict.get(self.var_name()) {
            match found {
                SecValueType::Obj(obj) => {
                    for (_k, v) in obj.iter() {
                        cur_dict
                            .global_mut()
                            .set(self.cur_name().clone(), v.clone());
                        let TaskValue { vars, rec, .. } = self
                            .body
                            .async_exec(ctx.clone(), cur_dict, sender.clone())
                            .await?;
                        cur_dict = vars;
                        task.append(rec);
                    }
                }
                SecValueType::List(list) => {
                    for i in list.iter() {
                        cur_dict
                            .global_mut()
                            .set(self.cur_name().clone(), i.clone());
                        let TaskValue { vars, rec, .. } = self
                            .body
                            .async_exec(ctx.clone(), cur_dict, sender.clone())
                            .await?;
                        cur_dict = vars;
                        task.append(rec);
                    }
                }
                _ => {
                    return Err(ExecReason::from_logic()
                        .to_err()
                        .with_detail(format!("loop only support obj,list {}", self.var_name())));
                }
            }
            return Ok(TaskValue::from((cur_dict, ExecOut::Task(task))));
        }
        Err(ExecReason::Miss.to_err().with_detail(self.var_name()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ability::GxEcho, components::gxl_block::BlockAction,
        model::components::gxl_block::BlockNode, traits::Getter,
    };
    use orion_error::dev::testing::TestAssertWithMsg;
    use orion_sec::sec::{SecFrom, SecValueObj};
    use orion_variate::vars::UpperKey;
    use rstest::*;

    #[fixture]
    fn test_ctx() -> ExecContext {
        ExecContext::default()
    }

    #[fixture]
    fn test_dict() -> VarSpace {
        VarSpace::default()
    }

    #[rstest]
    #[tokio::test]
    async fn test_loop_exec_success(
        #[from(test_ctx)] ctx: ExecContext,
        #[from(test_dict)] mut dict: VarSpace,
    ) {
        let obj1 = {
            let mut n = SecValueObj::default();
            n.insert(
                UpperKey::from("key1"),
                SecValueType::nor_from("value1".to_string()),
            );
            n.insert(
                UpperKey::from("key2"),
                SecValueType::nor_from("value2".to_string()),
            );
            n
        };
        let obj2 = {
            let mut n = SecValueObj::default();
            n.insert(
                UpperKey::from("key1"),
                SecValueType::nor_from("value3".to_string()),
            );
            n.insert(
                UpperKey::from("key2"),
                SecValueType::nor_from("value4".to_string()),
            );
            n
        };
        // 准备测试数据
        let named_dict = {
            let mut n = SecValueObj::default();
            n.insert(UpperKey::from("key1"), SecValueType::from(obj1));
            n.insert(UpperKey::from("key2"), SecValueType::from(obj2));
            n
        };

        dict.global_mut()
            .set("test_dict".to_string(), SecValueType::from(named_dict));

        // 创建一个空的 BlockNode 作为循环体
        let mut body = BlockNode::new();
        body.append(BlockAction::from(GxEcho::new("loop: ${CURRENT.KEY1}")));
        let loop_node = GxlLoop::new("current".to_string(), "test_dict".to_string(), body);

        // 执行测试
        let result = loop_node
            .async_exec(ctx, dict, None)
            .await
            .assert("loop assert");

        let TaskValue { rec, .. } = result;
        if let ExecOut::Task(_task) = rec {
            //assert_eq!(job.tasks().len(), 2); // 应该有两个任务，对应两个键值对
        } else {
            panic!("Expected Job output");
        }
    }
    #[rstest]
    #[tokio::test]
    async fn test_loop_exec_with_list(
        #[from(test_ctx)] ctx: ExecContext,
        #[from(test_dict)] mut dict: VarSpace,
    ) {
        // Create a test list with SecValueType items
        let test_list = vec![
            SecValueType::nor_from("value1".to_string()),
            SecValueType::nor_from("value2".to_string()),
            SecValueType::nor_from(42u64),
            SecValueType::nor_from(true),
        ];

        dict.global_mut()
            .set("test_list".to_string(), SecValueType::List(test_list));

        // Create a block that will echo each value
        let mut body = BlockNode::new();
        body.append(BlockAction::from(GxEcho::new("loop: ${current}")));
        let loop_node = GxlLoop::new("current".to_string(), "test_list".to_string(), body);

        // Execute the loop
        loop_node.async_exec(ctx, dict, None).await.assert("loop");
    }

    #[rstest]
    #[tokio::test]
    async fn test_loop_exec_missing_dict(
        #[from(test_ctx)] ctx: ExecContext,
        #[from(test_dict)] dict: VarSpace,
    ) {
        // 创建一个空的 BlockNode 作为循环体
        let body = BlockNode::new();
        let loop_node = GxlLoop::new("current".to_string(), "missing_dict".to_string(), body);

        // 执行测试
        let result = loop_node.async_exec(ctx, dict, None).await;

        // 验证错误情况
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason(), ExecReason::Miss));
        assert_eq!(err.detail().as_deref(), Some("missing_dict"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_loop_exec_variable_setting(
        #[from(test_ctx)] ctx: ExecContext,
        #[from(test_dict)] mut dict: VarSpace,
    ) {
        // 准备测试数据
        let named_dict = {
            let mut n = SecValueObj::default();
            n.insert(
                UpperKey::from("key1"),
                SecValueType::nor_from("value1".to_string()),
            );
            n
        };

        dict.global_mut()
            .set("test_dict".to_string(), SecValueType::from(named_dict));

        // 创建一个会检查当前变量的 BlockNode
        let body = BlockNode::new(); // 这里可以添加验证逻辑
        let loop_node = GxlLoop::new("current".to_string(), "test_dict".to_string(), body);

        // 执行测试
        let result = loop_node
            .async_exec(ctx.clone(), dict.clone(), None)
            .await
            .assert("loop assert");

        // 验证变量设置
        let TaskValue { vars, .. } = result;
        assert_eq!(
            vars.global().get_copy("current").unwrap().to_string(),
            "value1" // 最后一次循环设置的值
        );
    }
}
