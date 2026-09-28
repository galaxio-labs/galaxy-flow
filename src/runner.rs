use crate::{
    GxLoader,
    ability::prelude::TaskValue,
    cmd::GxlCmd,
    err::{RunReason, RunResult},
    execution::VarSpace,
    util::redirect::ReadSignal,
};
use orion_error::conversion::{ConvErr, ErrorWith, ToStructError};
use std::{path::Path, sync::mpsc::Sender};

/// Galaxy Flow 运行器
///
/// GxlRunner负责执行GxlCmd命令，加载配置文件并运行指定的流程。
///
/// Galaxy Flow Runner
///
/// GxlRunner is responsible for executing GxlCmd commands, loading configuration files, and running specified flows.
pub struct GxlRunner {}
impl GxlRunner {
    /// 执行Galaxy Flow命令
    ///
    /// 此方法执行以下步骤：
    /// 1. 验证命令参数
    /// 2. 加载配置文件
    /// 3. 解析流程名称
    /// 4. 执行指定的流程
    ///
    /// Execute Galaxy Flow command
    ///
    /// This method performs the following steps:
    /// 1. Validate command parameters
    /// 2. Load configuration file
    /// 3. Parse flow names
    /// 4. Execute specified flows
    pub async fn run(
        cmd: GxlCmd,
        vars: VarSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> RunResult<TaskValue> {
        let loader = GxLoader::new();
        if let Some(ref conf) = cmd.conf {
            // 检查配置文件是否存在 / Check if configuration file exists
            if !Path::new(conf.as_str()).exists() {
                return Err(RunReason::from_conf()
                    .to_err()
                    .with_detail("gx run conf not exists"))
                .with_context(("conf", conf.clone()));
            }

            let spc = loader
                .parse_file(conf.as_str(), false, &vars)
                .await?
                .assemble()
                .conv_err()?;

            if cmd.flows.is_empty() {
                spc.show().conv_err()?;
            } else {
                // 解析环境列表 / Parse environment list
                return spc.exec(cmd, vars, sender).await;
            }
        }
        Err(RunReason::from_conf()
            .to_err()
            .with_detail("gx run exec fail!"))
    }

    /// 判定若干流程是否存在（只读：加载 + 装配，**不执行**）。
    ///
    /// 返回不存在的流程名列表。conf 缺失或解析失败（含 extern 未就绪）按
    /// 「无法确认存在」处理并返回 `Err`；调用方（CLI `--exists`）统一映射为退出码 1。
    pub async fn exists(
        conf: Option<String>,
        flows: &[String],
        vars: VarSpace,
    ) -> RunResult<Vec<String>> {
        let Some(conf) = conf else {
            return Err(RunReason::from_conf()
                .to_err()
                .with_detail("gx exists missing gxl file"));
        };
        if !Path::new(conf.as_str()).exists() {
            return Err(RunReason::from_conf()
                .to_err()
                .with_detail("gx exists conf not exists"))
            .with_context(("conf", conf.clone()));
        }
        let loader = GxLoader::new();
        let spc = loader
            .parse_file(conf.as_str(), false, &vars)
            .await?
            .assemble()
            .conv_err()?;
        Ok(flows
            .iter()
            .filter(|flow| !spc.has_flow(flow))
            .cloned()
            .collect())
    }

    pub async fn info(conf: Option<String>, vars: VarSpace) -> RunResult<()> {
        if let Some(ref conf) = conf {
            // 检查配置文件是否存在 / Check if configuration file exists
            if !Path::new(conf.as_str()).exists() {
                return Err(RunReason::from_conf()
                    .to_err()
                    .with_detail("gx run conf not exists"))
                .with_context(("conf", conf.clone()));
            }
            let loader = GxLoader::new();

            let spc = loader
                .parse_file(conf.as_str(), false, &vars)
                .await?
                .assemble()
                .conv_err()?;
            spc.show().conv_err()?;
            Ok(())
        } else {
            Err(RunReason::from_conf()
                .to_err()
                .with_detail("gx run missing gxl file"))
        }
    }
}
