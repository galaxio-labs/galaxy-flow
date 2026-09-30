use crate::{
    ExecError,
    task_report::main_task::{create_main_task, get_task_parent_id},
    util::redirect::{init_redirect_file, platform::StdoutRedirect},
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

lazy_static! {
    pub static ref TASK_REPORT_CENTER: OnceCell<RwLock<TaskCenter>> = OnceCell::new();
}

pub async fn report_enable() -> bool {
    if let Some(config) = TASK_REPORT_CENTER.get() {
        return config.read().await.report_enable;
    }
    false
}

pub async fn set_report_enable(enable: bool) {
    if let Some(config) = TASK_REPORT_CENTER.get() {
        let mut task_config = config.write().await;
        task_config.report_enable = enable;
    }
}

// 任务结果配置
#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Clone)]
pub struct TaskCenter {
    pub report_enable: bool,
    pub report_svr: ReportCenterConf,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters, Clone)]
pub struct ReportCenterConf {
    domain: String,
    port: u16,
}
impl ReportCenterConf {
    pub fn new<S: Into<String>>(domain: S, port: u16) -> Self {
        Self {
            domain: domain.into(),
            port,
        }
    }

    // 定义一个名为local的函数，返回Self类型
    pub fn local() -> Self {
        // 调用Self的new方法，传入参数
        Self::new(
            // 传入IP地址
            "127.0.0.1",
            // 传入端口号
            8066,
        )
    }
}

pub enum TaskUrlType {
    TaskNotice,
    TaskReport,
    MainTaskCreate,
}

pub async fn build_task_url(url_type: TaskUrlType) -> Option<String> {
    let task_config = TASK_REPORT_CENTER.get()?;
    let task_config = task_config.read().await;
    let report_svr = task_config.report_svr.clone();
    match url_type {
        TaskUrlType::TaskNotice => Some(format!(
            "http://{}:{}/api/task/subtasks:batchCreate",
            report_svr.domain(),
            report_svr.port(),
        )),
        TaskUrlType::TaskReport => Some(format!(
            "http://{}:{}/api/task/subtasks:update",
            report_svr.domain(),
            report_svr.port()
        )),
        TaskUrlType::MainTaskCreate => Some(format!(
            "http://{}:{}/api/task/maintasks",
            report_svr.domain(),
            report_svr.port()
        )),
    }
}

/// 初始化重定向和父任务
pub async fn init_redirect_and_parent_task(
    task_name: String,
    ai_diagnose: bool,
) -> Result<Option<StdoutRedirect>, ExecError> {
    let mut redirect: Option<StdoutRedirect> = None;

    if ai_diagnose || report_enable().await {
        // 处理日志路径
        let log_path = init_redirect_file()?;
        // macOS平台特定逻辑
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let stdout_redirect = match StdoutRedirect::start(&log_path) {
                Some(r) => r,
                None => return Ok(None),
            };
            redirect = Some(stdout_redirect);
        }

        // 处理父任务逻辑
        if get_task_parent_id().is_none() {
            create_main_task(task_name).await;
        }
    }

    Ok(redirect)
}
