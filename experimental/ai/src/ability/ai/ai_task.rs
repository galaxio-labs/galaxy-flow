use crate::ability::ai::{AI_CALL_RESULT, AI_CALL_VALUE, AI_CONTENT};
use crate::{ability::prelude::*, traits::Setter};
use async_trait::async_trait;

use orion_ai::types::ExecutionStatus;
use orion_ai::{AiConfig, AiExecUnit, AiExecUnitBuilder};
use orion_error::conversion::ConvErr;
use orion_sec::sec::SecFrom;
use orion_sec::sec::SecValueType;

use getset::{Getters, MutGetters, Setters};
use orion_variate::vars::{EnvDict, ValueType};

/// AI执行器，支持函数调用的AI任务执行器
///
/// 这个结构体提供了简化的AI任务执行接口，复用orion_ai的FunctionRegistry
/// 来实现工具调用功能。
///
/// # 示例
///
#[derive(Clone, Debug, Getters, MutGetters, Setters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct AiTaskExecutor {
    role: Option<String>,
    task: Option<String>,
    config: Option<AiConfig>,
    tools: Vec<String>,
    // 最大重试轮次，默认为3
    max_rounds: usize,
}

impl Default for AiTaskExecutor {
    fn default() -> Self {
        Self {
            role: None,
            task: None,
            config: None,
            tools: Vec::new(),
            max_rounds: 3, // 默认最大重试3次
        }
    }
}

impl AiTaskExecutor {
    /// 执行AI任务
    ///
    /// 这是主要的执行入口点，负责协调整个AI任务的执行流程。
    pub async fn execute(&self, mut ctx: ExecContext, mut vars: VarSpace) -> TaskResult {
        // 设置执行上下文
        ctx.append("gx.ai_task");
        let mut action = Action::from("gx.ai_task");
        let task_prompt = self.task.as_deref().unwrap_or("请完成任务");

        // 初始化执行单元
        let exec_unit = self.setup_exec_unit(&vars)?;

        // 执行AI请求
        let response = exec_unit.execute_with_func(task_prompt).await.err_conv()?;

        // 存储结果
        vars.global_mut()
            .set(AI_CONTENT, SecValueType::nor_from(response.content.clone()));
        match response.status {
            ExecutionStatus::Success => {
                let mut call_result = Vec::new();
                for call in response.tool_calls {
                    //println!("✅  {:#} ", call.result);
                    call_result.push(ValueType::from(call.result.to_string()));
                }
                vars.global_mut()
                    .set(AI_CALL_VALUE, SecValueType::nor_from(call_result));
                vars.global_mut()
                    .set(AI_CALL_RESULT, SecValueType::nor_from(true));
            }
            _ => {
                vars.global_mut()
                    .set(AI_CALL_RESULT, SecValueType::nor_from(false));
            }
        }

        action.finish();
        Ok(TaskValue::from((vars, ExecOut::Action(action))))
    }

    /// 创建执行单元
    ///
    /// 使用配置创建AI执行单元，封装客户端、角色和函数注册表。
    fn setup_exec_unit(&self, vars: &VarSpace) -> ExecResult<AiExecUnit> {
        // 使用构建器创建执行单元
        AiExecUnitBuilder::new(EnvDict::from(vars.global().export()))
            .with_config_opt(self.config.clone())
            .with_role_opt(self.role.clone())
            .with_tools(self.tools.clone())
            .build()
            .err_conv()
            .doing("create ai exec unit")
    }
}

impl ComponentMeta for AiTaskExecutor {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("ai_fun")
    }
}

#[async_trait]
impl AsyncRunnableTrait for AiTaskExecutor {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace) -> TaskResult {
        self.execute(ctx, vars).await
    }
}

#[cfg(test)]
mod tests {
    use orion_ai::{AiResult, GlobalFunctionRegistry, types::ExecutionStatus};
    use orion_error::dev::testing::TestAssert;
    use orion_sec::load_sec_dict;

    use crate::infra::once_init_log;

    use super::*;

    #[tokio::test]
    async fn test_executor_with_tools() -> AiResult<()> {
        GlobalFunctionRegistry::initialize().assert();
        let ai_builder = AiExecUnitBuilder::new(load_sec_dict().unwrap());
        let ai_exec = ai_builder
            .clone()
            .with_role("developer")
            .with_tools(vec!["git-status".to_string()])
            .build()
            .doing("create ai exec unit")?;

        //let mut executor = AiExecutor::default();

        let response = ai_exec
            .execute_with_func("请检查当前Git仓库的状态，看看有哪些文件被修改了")
            .await?;

        match response.status {
            ExecutionStatus::Success => {
                println!("✅  {} ", response.content);
                for call in response.tool_calls {
                    println!("✅  {:#} ", call.result);
                }
            }
            _ => {
                eprintln!("❌ {}", response.content);
                panic!("false");
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_component_meta() {
        let executor = AiTaskExecutor::default();
        let meta = executor.gxl_meta();

        assert_eq!(meta, GxlMeta::from("ai_fun"));
    }

    #[tokio::test]
    async fn test_clone_and_debug() -> ExecResult<()> {
        once_init_log();
        GlobalFunctionRegistry::initialize().assert();
        let mut executor = AiTaskExecutor::default();
        executor.set_role(Some("developer".to_string()));
        executor.set_task(Some(
            "请检查当前Git仓库的状态，看看有哪些文件被修改了".to_string(),
        ));
        executor.set_tools(vec!["git-status".to_string()]);
        let x = executor
            .async_exec(ExecContext::default(), VarSpace::sys_init()?)
            .await?;
        println!("{:#}", x.vars.get(AI_CONTENT).assert());
        println!("{:#}", x.vars.get(AI_CALL_RESULT).assert());
        println!("{:#}", x.vars.get(AI_CALL_VALUE).assert());
        Ok(())
    }
}
