//! AI模块的预导入模块
//!
//! 这个模块重导出了AI功能相关的常用类型和trait，便于使用。

// 重导出主要的类型和trait
pub use super::ai_executor::AiExecutor;
pub use super::chat_executor::ChatExecutor;

// 为了向后兼容，重导出旧类型（待移除）
pub use super::ai_executor::AiExecutor as GxAIFun;
pub use super::chat_executor::ChatExecutor as GxAIChat;

// 重导出 orion_ai 的相关类型
pub use orion_ai::{
    AiClient, AiClientTrait, AiConfig, AiRoleID, FunctionRegistry,
    provider::{AiResponse, FunctionCall, FunctionDefinition, FunctionResult},
};

// 重导出错误类型
pub use orion_error::AiResult;

/// 重新导出常用的trait，减少使用时的导入
pub use crate::ability::prelude::*;
