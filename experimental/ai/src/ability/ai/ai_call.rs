use crate::ability::ai::AI_CALL_RESULT;
use crate::ability::ai::AI_CALL_VALUE;
use crate::ability::gxl::do_gxl_run;
use crate::ability::prelude::*;
use crate::cmd::GxlCmd;
use crate::util::OptionFrom;
use async_trait::async_trait;
use orion_error::OperationContext;
use orion_sec::sec::NoSecConv;
use orion_sec::sec::SecFrom;
use orion_sec::sec::SecValueType;
use std::sync::Arc;
use std::sync::OnceLock;

use orion_ai::{
    AiConfig, AiExecUnit, AiExecUnitBuilder, AiResult, ExecutionResult, FunctionCall,
    FunctionDefinition, FunctionExecutor, FunctionResult, GlobalFunctionRegistry,
};
use orion_error::conversion::{ConvErr, SourceErr};

use getset::{Getters, MutGetters, Setters};
use orion_variate::vars::EnvDict;

/// AI执行器，支持函数调用的AI任务执行器
///
/// 这个结构体提供了简化的AI任务执行接口，复用orion_ai的FunctionRegistry
/// 来实现工具调用功能。
///
/// # 示例
///
#[derive(Debug, Getters, MutGetters, Setters, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct GxlAiRegist {
    role: Option<String>,
    desp: String,
    config: Option<AiConfig>,
    flow: String,
    exe_unit: OnceLock<Arc<AiExecUnit>>,
    exe_vars: OnceLock<VarSpace>,
    exe_cmd: OnceLock<GxlCmd>,
}

impl Default for GxlAiRegist {
    fn default() -> Self {
        Self {
            role: None,
            desp: "unknow".to_string(),
            config: None,
            flow: "unknow".to_string(),
            exe_unit: OnceLock::new(),
            exe_vars: OnceLock::new(),
            exe_cmd: OnceLock::new(),
        }
    }
}

impl GxlAiRegist {
    pub async fn execute_call(&self) -> AiResult<ExecutionResult> {
        if let Some(exec_unit) = self.exe_unit.get() {
            let task_prompt = self.desp();
            return exec_unit.execute_with_func(task_prompt).await;
        } else {
            unreachable!("ai-exec_unit not initialized. Call setup_exec_unit first.")
        }
    }

    /// 创建执行单元
    ///
    /// 使用配置创建AI执行单元，封装客户端、角色和函数注册表。
    fn setup_exec_unit(&self, ctx: ExecContext, vars: &VarSpace) -> ExecResult<()> {
        // 使用构建器创建执行单元
        if self.exe_unit.get().is_none() {
            let exec_unit = Arc::new(
                AiExecUnitBuilder::new(EnvDict::from(vars.global().export()))
                    .with_config_opt(self.config.clone())
                    .with_role_opt(self.role.clone())
                    //.with_tools(self.flow.clone())
                    .build()
                    .err_conv()
                    .doing("create ai exec unit")?,
            );
            self.exe_unit
                .set(exec_unit)
                .expect("OnceLock should not be already set");
        }
        if self.exe_vars.get().is_none() {
            self.exe_vars
                .set(vars.clone())
                .expect("OnceLock should not be already set");
        }
        if self.exe_cmd.get().is_none() {
            self.exe_cmd
                .set(ctx.gxl_cmd().as_ref().clone())
                .expect("OnceLock should not be already set");
        }
        Ok(())
    }
    fn call_key(&self) -> String {
        format!("gxl-{}", self.flow())
    }
    fn call_define(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: self.call_key(),
            description: self.desp().clone(),
            parameters: vec![],
        }
    }
}

impl ComponentMeta for GxlAiRegist {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("ai_regist")
    }
}
#[async_trait::async_trait]
impl FunctionExecutor for GxlAiRegist {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        let cmd = self.exe_cmd().get().cloned().expect("exe_cmd not exists");
        let cmd = cmd.with_flows(self.flow().clone());
        let vars = self.exe_vars().get().cloned().expect("exe_vars not exists");
        let task_value = do_gxl_run(cmd, &vars, true, None)
            .await
            .source_err(UvsReason::network_error().into(), "source error")?;

        if let (Some(call_result), Some(call_value)) = (
            task_value.vars.get(AI_CALL_RESULT),
            task_value.vars.get(AI_CALL_VALUE),
        ) && call_result == SecValueType::sec_from(true)
        {
            return Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!(call_value.no_sec()),
                error: None,
            });
        }
        Ok(FunctionResult {
            name: function_call.function.name.clone(),
            result: serde_json::json!(false),
            error: "call no response".to_opt(),
        })
    }

    fn supported_functions(&self) -> Vec<String> {
        vec![self.call_key()]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        if function_name == self.call_key() {
            Some(self.call_define())
        } else {
            None
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlAiRegist {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace) -> TaskResult {
        let fun_key = self.call_key();
        let mut op_ctx = OperationContext::doing("regist tool")
            .with_auto_log()
            .with_mod_path("gxl/ai");
        op_ctx.record("fun", fun_key.as_str());
        self.setup_exec_unit(ctx, &vars)?;
        GlobalFunctionRegistry::register_function(self.call_define()).err_conv()?;
        GlobalFunctionRegistry::register_executor(fun_key, Arc::new(self.clone())).err_conv()?;
        op_ctx.mark_suc();
        Ok(TaskValue::new(vars, ExecOut::Ignore))
    }
}

#[cfg(test)]
mod tests {}
