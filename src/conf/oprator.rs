use orion_conf::TomlIO;
use orion_error::conversion::{SourceErr, ToStructError};
use orion_error::reason::UnifiedReason as UvsReason;
use orion_infra::path::ensure_path;
use std::path::PathBuf;

use crate::{
    err::{RunReason, RunResult},
    task_report::task_rc_config::TASK_REPORT_CENTER,
};

use super::gxlconf::GxlConf;
use crate::task_report::task_rc_config::ReportCenterConf;

// 加载配置文件
pub fn load_gxl_config() {
    let task_config_path = conf_path().unwrap_or_default();

    let content = match std::fs::read_to_string(&task_config_path) {
        Ok(content) => content,
        Err(e) => {
            warn!(
                "Failed to read config file at {}: {}",
                task_config_path.display(),
                e
            );
            return;
        }
    };

    match toml::from_str::<GxlConf>(&content) {
        Ok(config) => {
            if TASK_REPORT_CENTER
                .set(tokio::sync::RwLock::new(config.task_report().clone()))
                .is_err()
            {
                warn!("Failed to set task report center");
            }
        }
        Err(e) => {
            warn!("Failed to parse config file: {}", e.message());
        }
    }
}

pub fn conf_path() -> Option<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        let galaxy_root = home_dir.join(".galaxy");
        let conf_file = galaxy_root.join("conf.toml");
        if conf_file.exists() {
            return Some(conf_file);
        }
    }
    None
}

pub fn conf_init() -> RunResult<()> {
    if let Some(home_dir) = dirs::home_dir() {
        let galaxy_root = home_dir.join(".galaxy");
        ensure_path(&galaxy_root).source_err(UvsReason::logic_error().into(), "source error")?;
        let conf_file = galaxy_root.join("conf.toml");
        let conf = GxlConf::new(ReportCenterConf::local(), true);
        conf.save_toml(&conf_file)
            .source_err(UvsReason::resource_error().into(), "source error")?;
        return Ok(());
    }
    Err(RunReason::from_sys()
        .to_err()
        .with_detail("get home dir failed!"))
}

#[cfg(test)]
mod tests {
    use std::fs::remove_file;

    use crate::{
        conf::{conf_init, conf_path, oprator::load_gxl_config},
        task_report::task_rc_config::TASK_REPORT_CENTER,
    };
    use orion_error::dev::testing::TestAssert;

    // 加载任务配置测试
    #[tokio::test]
    async fn test_load_task_config() {
        // 创建临时目录和文件
        //let dir = PathBuf::from("./temp");
        let file_path = conf_path();

        match file_path {
            Some(_) => {
                load_gxl_config();
            }
            None => {
                // 写入测试内容
                conf_init().assert();
                load_gxl_config();
                let path = conf_path().assert();
                // 删除临时文件
                remove_file(path).assert();
            }
        }

        let task_result_config = TASK_REPORT_CENTER.get();
        assert!(task_result_config.is_some());
        // TASK_REPORT_CENTER.take();
    }
}
