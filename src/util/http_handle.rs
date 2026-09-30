use crate::{
    ExecReason,
    execution::task::Task,
    task_report::{
        task_notification::{TaskNotice, TaskOutline},
        task_rc_config::{TaskUrlType, build_task_url, report_enable, set_report_enable},
    },
};
use colored::Colorize;
use orion_error::conversion::SourceRawErr;
use serde::Serialize;
use std::{fmt::Debug, time::Duration};

// 发送http请求
pub async fn send_http_request<T: Serialize + Debug>(payload: T, url: &String) {
    if !report_enable().await {
        return; // 如果报告中心未启用，则直接返回不再发送http请求
    }
    let response = reqwest::Client::new()
        .post(url)
        .json(&payload)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .source_raw_err(ExecReason::NetWork, "HTTP request failed");

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                info!(
                    "HTTP request to {} succeeded. {}",
                    url,
                    resp.text().await.unwrap_or_default()
                );
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                info!(
                    "{}",
                    format!(
                        "HTTP request to {url} failed with status {status}: {text}, payload:{payload:?}",
                    )
                    .yellow()
                    .bold()
                );
                set_report_enable(false).await; // Disable reporting if the request fails
            }
        }
        Err(e) => {
            info!(
                "{}",
                format!("HTTP request to {url} failed: {e}, payload:{payload:?}")
                    .yellow()
                    .bold()
            );
            set_report_enable(false).await; // Disable reporting if the request fails
        }
    }
}

// 创建并发送任务通知
pub async fn create_and_send_task_notice(
    task: &Task,
    task_notice: &TaskNotice,
) -> Result<TaskNotice, ExecReason> {
    let url = build_task_url(TaskUrlType::TaskNotice)
        .await
        .unwrap_or_default();

    let notice = TaskNotice {
        parent_id: task_notice.parent_id.clone(), // 明确初始化
        name: task.name().to_string(),
        description: task.name().to_string(),
        order: task_notice.order, // 明确初始化
    };

    let task_outline = TaskOutline {
        tasks: vec![notice.clone()],
    };
    send_http_request(task_outline, &url).await;
    Ok(notice)
}
