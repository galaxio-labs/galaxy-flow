use orion_ai::{AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionResult};

pub struct GxlAiExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for GxlAiExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        Ok(FunctionResult {
            name: function_call.function.name.clone(),
            result: serde_json::json!({"mixed": "custom_result"}),
            error: None,
        })
    }

    fn supported_functions(&self) -> Vec<String> {
        vec!["mixed-custom-tool".to_string()]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        if function_name == "mixed-custom-tool" {
            Some(FunctionDefinition {
                name: "mixed-custom-tool".to_string(),
                description: "Custom tool for mixed test".to_string(),
                parameters: vec![],
            })
        } else {
            None
        }
    }
}
