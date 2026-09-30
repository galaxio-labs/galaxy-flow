use clap::{ArgAction, Args};
use getset::{Getters, Setters, WithSetters};

#[derive(Debug, Clone, WithSetters, Setters, Getters)] // requires `derive` feature
#[getset(set_with = "pub", get = "pub")]
pub struct GxlCmd {
    env: String,

    pub flows: String,

    pub debug: usize,

    pub conf: Option<String>,

    pub log: Option<String>,

    pub quiet: bool,

    pub cmd_args: Vec<String>,

    pub dryrun: bool,
    pub ai: bool,
}

impl GxlCmd {
    pub fn get_env_list(&self) -> Vec<String> {
        if self.env.is_empty() {
            Vec::new()
        } else {
            self.env
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    }
}

impl Default for GxlCmd {
    fn default() -> Self {
        Self {
            env: "default".to_string(),
            flows: "unknow".to_string(),
            debug: 0,
            conf: None,
            log: None,
            quiet: false,
            cmd_args: Vec::new(),
            dryrun: false,
            ai: false,
        }
    }
}
/// Galaxy Flow Command Line Interface
///
/// GxlCmd是Galaxy Flow的命令行接口结构体，用于解析和处理命令行参数。
///
/// Galaxy Flow Command Line Interface
///
/// GxlCmd is the command line interface structure for Galaxy Flow, used to parse and process command line arguments.
#[derive(Args, Debug, Clone, WithSetters, Setters)] // requires `derive` feature
#[getset(set_with = "pub")]
pub struct GFlowCmd {
    /// 环境名称 / Environment name
    ///
    /// 指定运行环境，例如：dev, test, prod
    ///
    /// Specify the runtime environment, e.g.: dev, test, prod
    /// 示例/Example: -e dev
    #[arg(short = 'e', long = "env", default_value = "default")]
    #[getset(set = "pub")]
    env: String,

    /// 位置参数流程名称 / Positional flow names
    ///
    /// 作为位置参数指定的流程名称列表
    ///
    /// List of flow names specified as positional arguments
    #[arg()] // Positional arguments
    pub flows: Vec<String>,

    /// 调试级别 / Debug level
    ///
    /// 设置调试输出级别，数值越大输出越详细
    ///
    /// Set the debug output level, higher values produce more detailed output
    /// 示例/Example: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,

    /// 配置文件路径 / Configuration file path
    ///
    /// 指定GXL配置文件的路径
    /// 默认值由子命令决定
    ///
    /// Specify the path to the GXL configuration file
    /// Default value depends on the subcommand
    /// 示例/Example: -c ./config.gxl
    #[arg(short = 'c', long = "conf")]
    pub conf: Option<String>,

    /// 日志配置 / Log configuration
    ///
    /// 配置日志输出级别和格式
    ///
    /// Configure log output level and format
    /// 示例/Example: --log cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,

    /// 静默模式 / Quiet mode
    ///
    /// 启用静默模式，减少输出信息
    ///
    /// Enable quiet mode to reduce output information
    /// 示例/Example: -q
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue,default_value = "false" )]
    pub quiet: bool,

    /// 命令参数 / Command arguments
    ///
    /// 传递给流程的命令行参数，允许以连字符开头的值
    ///
    /// Command line arguments passed to the flow, allows values starting with hyphens
    /// 示例/Example: --cmd-arg "-x -y"
    #[arg(
        long = "cmd-arg",
        value_name = "ARG",
        num_args = 1,
        allow_hyphen_values = true
    )]
    pub cmd_args: Vec<String>,

    /// 试运行模式 / Dry run mode
    ///
    /// 启用试运行模式，只显示将要执行的操作而不实际执行
    ///
    /// Enable dry run mode, only show what operations would be performed without actually executing them
    /// 示例/Example: --dryrun
    #[arg(long = "dryrun", action = ArgAction::SetTrue, default_value = "false")]
    pub dryrun: bool,

    /// AI辅助模式 / AI assistance mode
    ///
    /// 启用AI辅助功能，在出错时提供智能诊断和建议
    ///
    /// Enable AI assistance features, provide intelligent diagnosis and suggestions when errors occur
    /// 示例/Example: --ai
    #[arg(long = "ai", action = ArgAction::SetTrue, default_value = "false")]
    pub ai: bool,

    /// 仅判断流程是否存在（不执行）/ Check flow existence only (no execution)
    ///
    /// 存在 exit 0，不存在 exit 1；用于「有则调用、无则跳过」。
    ///
    /// Exit 0 when the flow(s) exist, 1 otherwise.
    /// 示例/Example: gx run localize --exists
    #[arg(long = "exists", action = ArgAction::SetTrue, default_value = "false")]
    pub exists: bool,
}

impl GFlowCmd {
    /// 获取所有流程名称
    ///
    /// 返回所有要执行的流程名称列表
    ///
    /// Get all flow names
    ///
    /// Return a list of all flow names to be executed
    pub fn get_all_flows(&self) -> Vec<String> {
        if !self.flows.is_empty() {
            self.flows.clone()
        } else {
            Vec::new()
        }
    }

    /// 将环境字符串按逗号分解为向量
    ///
    /// 将env字段中的字符串按逗号分隔，返回字符串向量
    /// 如果env为空字符串，则返回空向量
    ///
    /// Split environment string by comma into vector
    ///
    /// Split the string in the env field by comma and return a vector of strings
    /// If env is an empty string, return an empty vector
    pub fn get_env_list(&self) -> Vec<String> {
        if self.env.is_empty() {
            Vec::new()
        } else {
            self.env
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    }

    pub fn list_cmd(&self) -> Vec<GxlCmd> {
        let mut cmds = Vec::new();
        for flow in self.get_all_flows() {
            cmds.push(GxlCmd {
                env: self.env.clone(),
                flows: flow.clone(),
                debug: self.debug,
                conf: self.conf.clone(),
                log: self.log.clone(),
                quiet: self.quiet,
                cmd_args: self.cmd_args.clone(),
                dryrun: self.dryrun,
                ai: self.ai,
            })
        }
        cmds
    }
}
