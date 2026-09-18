//pub mod ai_call;
//pub mod ai_chat;
//pub mod ai_task;
//pub mod gxl_fun;
pub mod tool;
// 向后兼容性导出
//pub use ai_chat::AiChatExecutor as GxAIChat;
//pub use ai_task::AiTaskExecutor as GxAIFun;

pub const AI_CONTENT: &str = "AI_CONTENT";
pub const AI_CALL_RESULT: &str = "AI_CALL_RESULT";
pub const AI_CALL_VALUE: &str = "AI_CALL_VALUE";
