use chrono;

// 🎯 工具调用结果结构
use getset::{Getters, MutGetters, Setters, WithSetters};
use orion_sec::sec::{SecFrom, SecValueObj, SecValueType};
use orion_variate::vars::UpperKey;
#[derive(Clone, Debug, Getters, Setters, WithSetters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct ToolCallResult {
    tool_name: String,              // 工具名称
    result: Result<String, String>, // 成功结果（如果成功）
    timestamp: String,              // 执行时间戳
}
impl Default for ToolCallResult {
    fn default() -> Self {
        Self {
            tool_name: "no_tool".into(),
            result: Err("no_exec".into()),
            timestamp: chrono::Local::now().to_rfc3339(),
        }
    }
}
impl ToolCallResult {
    pub fn new<S: Into<String>>(tool_name: S, result: Result<String, String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            result,
            timestamp: chrono::Local::now().to_rfc3339(),
        }
    }
}

//impl From<SecValueObj> for ToolCallResult {}

impl From<ToolCallResult> for SecValueObj {
    fn from(value: ToolCallResult) -> Self {
        let mut ins = SecValueObj::new();
        ins.insert(
            UpperKey::from("tool_name"),
            SecValueType::nor_from(value.tool_name().clone()),
        );

        match value.result() {
            Ok(c) => {
                ins.insert(UpperKey::from("result_flag"), SecValueType::nor_from(true));
                ins.insert(
                    UpperKey::from("result_content"),
                    SecValueType::nor_from(c.clone()),
                );
            }
            Err(c) => {
                ins.insert(UpperKey::from("result_flag"), SecValueType::nor_from(false));
                ins.insert(
                    UpperKey::from("result_content"),
                    SecValueType::nor_from(c.clone()),
                );
            }
        }
        ins
    }
}

// 🎯 执行会话状态
#[derive(Clone, Debug, Getters, Setters, WithSetters, MutGetters, Default)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct ExecutionSession {
    tool_results: ToolCallResult, // 所有工具调用结果
}

impl ExecutionSession {
    pub fn store_exec_result(&mut self, tr: ToolCallResult) {
        self.tool_results = tr;
    }
    pub fn export(self) -> SecValueObj {
        SecValueObj::from(self.tool_results)
    }
}

pub fn build_retry_prompt(
    base_prompt: &str,
    tool_results: &ToolCallResult,
    current_round: usize,
) -> String {
    if current_round == 0 {
        format!("请完成以下任务：{}", base_prompt)
    } else {
        let mut context = format!("请重新尝试完成以下任务：{}\n\n", base_prompt);
        context.push_str(&format!("当前是第 {} 次尝试。\n\n", current_round,));

        context.push_str(&format!(
            "之前：工具 '{}' 执行失败 - {}\n",
            tool_results.tool_name(),
            tool_results
                .result()
                .clone()
                .err()
                .unwrap_or("unknow error".to_string()),
        ));
        context
    }
}
