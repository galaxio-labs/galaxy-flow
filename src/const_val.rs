/// GXL相关常量定义
pub mod gxl_const {

    /// 命令干运行标记
    pub const CMD_DRYRUN: &str = "GXL_CMD_DRYRUN";
    /// 模块更新标记
    pub const CMD_MODUP: &str = "GXL_CMD_MODUP";
    pub const CMD_ARG: &str = "GXL_CMD_ARG";
    /// 项目根目录变量
    pub const PRJ_ROOT: &str = "GXL_PRJ_ROOT";
    /// Git分支变量
    pub const GIT_BRANCH: &str = "GXL_GIT_BRANCH";
    /// 系统类型变量
    pub const OS_SYS: &str = "GXL_OS_SYS";
    /// 启动根目录变量
    pub const START_ROOT: &str = "GXL_START_ROOT";
    /// 当前目录变量
    pub const CUR_DIR: &str = "GXL_CUR_DIR";
    /// 错误消息前缀
    pub const ERROR_PREFIX: &str = "GXL ERROR: ";
    pub const CONFIG_FILE: &str = "conf.toml";
    pub const REDIRECT_FILE: &str = ".galaxy/redirect.yml";
    pub const NET_ACCS_CTRL_PATH_FILE: &str = ".galaxy/net_accessor_ctrl.yml";
    pub const NET_ACCESS_CTRL_FILE: &str = "net_accessor_ctrl.yml";
    pub const AI_CONF_FILE: &str = "ai.yml";
    pub const AI_ROLE_FILE: &str = "ai-roles.yml";

    /// 主模块名：未限定的流程名默认归属该模块。
    pub const MAIN_MOD: &str = "main";

    /// 将流程名归一化为限定名 `<mod>.<flow>`：
    /// 已含 `.` 的视为 `mod.flow` 限定名；否则归属 `MAIN_MOD`。
    ///
    /// `gx.exists` / `--exists` 与运行期解析共用同一规则，避免判定与“能不能跑”不一致。
    pub fn full_flow_name(name: &str) -> String {
        if name.contains('.') {
            name.to_string()
        } else {
            format!("{MAIN_MOD}.{name}")
        }
    }
}

/// AI工具相关常量定义
pub mod ai_const {
    /// AI工具变量前缀
    pub const AI_TOOLS_VAR_PREFIX: &str = "ai";

    /// 状态后缀
    pub const STATUS_SUFFIX: &str = "status";

    /// 结果后缀
    pub const RESULT_SUFFIX: &str = "result";

    /// 错误后缀
    pub const ERROR_SUFFIX: &str = "error";

    /// 时间戳后缀
    pub const TIMESTAMP_SUFFIX: &str = "timestamp";

    /// 分隔符
    pub const VAR_SEPARATOR: &str = "_";
}
