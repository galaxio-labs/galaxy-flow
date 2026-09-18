use orion_ai::{AiConfig, AiExecUnitBuilder};
use orion_error::conversion::ConvErr;
use std::path::PathBuf;

use crate::ability::{ai::AI_CONTENT, prelude::*};
use crate::model::traits::Setter;
use getset::{Getters, MutGetters, Setters};
use orion_sec::sec::{SecFrom, SecValueType};
use orion_variate::EnvDict;

/// AI聊天执行器，专注于AI对话功能
///
/// 这个结构体提供了简化的AI对话接口，支持从文件或直接消息中
/// 获取提示词，与AI进行交互。
///
/// # 示例
///
#[derive(Clone, Debug, Default, Getters, MutGetters, Setters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct AiChatExecutor {
    prompt_file: Option<String>,
    prompt_msg: Option<String>,
    config: Option<AiConfig>,
    role: Option<String>,
}

impl AiChatExecutor {
    /// 执行AI聊天任务
    ///
    /// 这是主要的执行入口点，负责协调AI对话的整个执行流程。
    pub async fn execute(&self, mut ctx: ExecContext, mut vars: VarSpace) -> TaskResult {
        // 设置执行上下文
        ctx.append("gx.ai_chat");
        let mut action = Action::from("gx.ai_chat");
        let exp = EnvExpress::from_env_mix(vars.global().clone());
        let mut message = self.prompt_msg.clone().unwrap_or("".to_string());

        // 如果有提示文件，读取文件内容
        if let Some(prompt_file) = &self.prompt_file {
            let prompt_file = PathBuf::from(exp.eval(prompt_file)?);
            if !prompt_file.exists() {
                return ExecReason::Gxl
                    .to_err()
                    .with_detail(format!("{path} not exists", path = prompt_file.display()))
                    .err_result();
            }
            let data = std::fs::read_to_string(prompt_file.as_path()).source_err(
                ExecReason::from_res(),
                format!("read prompt file: {}", prompt_file.display()),
            )?;
            message.push('\n');
            message.push_str(data.as_str());
        }

        // 执行AI对话
        let response = self.execute_chat(&message, &vars).await?;

        // 存储结果
        vars.global_mut()
            .set(AI_CONTENT, SecValueType::nor_from(response.clone()));

        action.finish();
        Ok(TaskValue::from((vars, ExecOut::Action(action))))
    }

    /// 执行AI对话
    ///
    /// 初始化AI客户端并发送对话请求。
    async fn execute_chat(&self, message: &str, vars: &VarSpace) -> ExecResult<String> {
        // 加载AI配置
        let exec_unit = AiExecUnitBuilder::new(EnvDict::from(vars.global().export()))
            .with_config_opt(self.config.clone())
            .with_role_opt(self.role.clone())
            //.with_tools(self.tools.clone())
            .build()
            .err_conv()
            .doing("create ai exec unit")?;

        // 执行AI请求
        let response = exec_unit.execute(message).await.err_conv()?;

        println!(
            "AI Response:\nContent: {}\nModel: {:#?}\n",
            response.content, response.metadata
        );
        Ok(response.content)
    }
}

impl ComponentMeta for AiChatExecutor {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.ai_chat")
    }
}

#[async_trait]
impl AsyncRunnableTrait for AiChatExecutor {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace) -> TaskResult {
        self.execute(ctx, vars).await
    }
}

#[cfg(test)]
mod tests {

    use orion_ai::GlobalFunctionRegistry;
    use orion_error::dev::testing::TestAssert;

    use crate::{ability::ai::AI_CONTENT, infra::once_init_log};

    use super::*;

    #[tokio::test]
    async fn test_default_values() {
        let executor = AiChatExecutor::default();

        assert!(executor.prompt_file().is_none());
        assert!(executor.prompt_msg().is_none());
        assert!(executor.config().is_none());
        assert!(executor.role().is_none());
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let mut executor = AiChatExecutor::default();
        executor.set_role(Some("developer".to_string()));
        executor.set_prompt_msg(Some("test message".to_string()));
        executor.set_prompt_file(Some("test.txt".to_string()));

        assert_eq!(executor.role(), &Some("developer".to_string()));
        assert_eq!(executor.prompt_msg(), &Some("test message".to_string()));
        assert_eq!(executor.prompt_file(), &Some("test.txt".to_string()));
    }

    #[tokio::test]
    async fn test_component_meta() {
        let executor = AiChatExecutor::default();
        let meta = executor.gxl_meta();

        assert_eq!(meta, GxlMeta::from("gx.ai_chat"));
    }

    #[tokio::test]
    async fn test_basic_ai_execution() -> ExecResult<()> {
        once_init_log();
        GlobalFunctionRegistry::initialize().assert();
        let mut executor = AiChatExecutor::default();
        executor.set_role(Some("developer".to_string()));
        executor.set_prompt_msg(Some("请回答：1+1=?".to_string()));
        let x = executor
            .async_exec(ExecContext::default(), VarSpace::sys_init()?)
            .await?;

        println!("{:#}", x.vars.get(AI_CONTENT).assert());
        Ok(())
    }
}
