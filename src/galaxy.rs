use home::home_dir;
use orion_accessor::addr::access_ctrl::{Rule, Unit, serv::NetAccessCtrl};
use orion_conf::YamlIO;
use orion_error::conversion::{SourceErr, ToStructError};
use orion_error::reason::UnifiedReason as UvsReason;

use crate::{
    conf::{conf_init, conf_path},
    const_val::gxl_const::{CONFIG_FILE, NET_ACCESS_CTRL_FILE},
    err::{RunReason, RunResult},
};

const WORK_TEMPLATE: &str = include_str!("templates/work.gxl");
const ADM_TEMPLATE: &str = include_str!("templates/adm.gxl");

pub struct Galaxy {}

impl Galaxy {
    /// 初始化 Galaxy 环境
    ///
    /// 包括：
    /// - 创建 `${HOME}/.galaxy` 目录
    /// - 基于 RedirectService demo 数据创建序列化的 redirect.yml 文件
    pub fn env_init() -> RunResult<()> {
        // 获取家目录并构建环境目录
        let galaxy_dir = home_dir()
            .ok_or_else(|| {
                RunReason::from_res()
                    .to_err()
                    .with_detail("Cannot find home directory")
            })?
            .join(".galaxy");

        // 创建目录
        if !galaxy_dir.exists() {
            std::fs::create_dir_all(&galaxy_dir)
                .source_err(UvsReason::resource_error().into(), "source error")?;
        }

        if conf_path().is_none() {
            conf_init()?;
        } else {
            eprintln!(
                " {} exists! , global conf init ignore",
                galaxy_dir.join(CONFIG_FILE).display()
            );
        }

        // 构建正确的 RedirectService demo 数据

        let net_ctrl_path = galaxy_dir.join(NET_ACCESS_CTRL_FILE);
        if net_ctrl_path.exists() {
            eprintln!(
                " {} exists! , net access ctrl init ignore",
                net_ctrl_path.display()
            );
        } else {
            let rules = vec![Rule::new("https://google.com/*", "https://google.cn/")];
            let unit = Unit::new(rules, None, None);
            let service = NetAccessCtrl::new(vec![unit], true);
            service
                .save_yaml(&net_ctrl_path)
                .source_err(UvsReason::resource_error().into(), "source error")?;
        }

        Ok(())
    }

    /// 初始化项目目录（本地初始化，不依赖远程模板）
    ///
    /// 创建：
    /// - `./_gal/` 目录
    /// - `./_gal/work.gxl` 基本工作流配置
    /// - `./_gal/adm.gxl` 基本管理流配置
    pub fn project_init() -> RunResult<()> {
        let gal_dir = std::path::Path::new("./_gal");

        if gal_dir.exists() {
            eprintln!("{} already exists, skipping init", gal_dir.display());
            return Ok(());
        }

        std::fs::create_dir_all(gal_dir)
            .source_err(UvsReason::resource_error().into(), "source error")?;

        // Create basic work.gxl from template
        let work_gxl = gal_dir.join("work.gxl");
        std::fs::write(&work_gxl, WORK_TEMPLATE)
            .source_err(UvsReason::resource_error().into(), "source error")?;
        eprintln!("created: {}", work_gxl.display());

        // Create basic adm.gxl from template
        let adm_gxl = gal_dir.join("adm.gxl");
        std::fs::write(&adm_gxl, ADM_TEMPLATE)
            .source_err(UvsReason::resource_error().into(), "source error")?;
        eprintln!("created: {}", adm_gxl.display());

        eprintln!("project initialized in ./_gal/");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use orion_error::dev::testing::TestAssertWithMsg;

    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn test_env_init_success() {
        // 使用临时目录进行测试
        let temp_dir = PathBuf::from("./temp");
        let galaxy_dir = temp_dir.join(".galaxy");

        // 临时修改HOME环境变量
        let old_home = std::env::var("HOME").unwrap();
        unsafe {
            std::env::set_var("HOME", temp_dir);
        }

        // 确保清理
        let _cleanup = || unsafe {
            std::env::set_var("HOME", &old_home);
        };

        // 执行初始化
        let result = Galaxy::env_init();
        assert!(result.is_ok(), "Environment init should succeed");
        let conf_path = galaxy_dir.join(NET_ACCESS_CTRL_FILE);
        let global_conf_path = galaxy_dir.join(CONFIG_FILE);

        // 验证目录和文件创建
        assert!(galaxy_dir.exists());
        assert!(conf_path.exists());
        assert!(global_conf_path.exists());

        NetAccessCtrl::load_yaml(&conf_path).assert("redict");
        // 验证文件内容包含关键字段
        let content = fs::read_to_string(conf_path).unwrap();
        println!("{content}");
        assert!(content.contains("units:"));
        assert!(content.contains("enable: true"));
    }
}
