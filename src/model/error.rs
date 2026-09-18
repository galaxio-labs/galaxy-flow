use derive_more::From;
use orion_conf::error::SerdeReason;
use orion_error::reason::{DomainReason, ErrorCode, UnifiedReason as UvsReason};
use orion_error::{OrionError, StructError};
use orion_sec::OrionSecReason;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, From, Error)]
pub enum AssembleReason {
    #[error("miss : {0}")]
    Miss(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

impl DomainReason for AssembleReason {}

impl ErrorCode for AssembleReason {
    fn error_code(&self) -> i32 {
        520
    }
}

impl AssembleReason {
    pub fn from_logic() -> Self {
        Self::Uvs(UvsReason::logic_error())
    }
}

pub type AssembleError = StructError<AssembleReason>;
pub type AResult<T> = Result<T, AssembleError>;

#[derive(Debug, PartialEq, Serialize, OrionError)]
pub enum ExecReason {
    #[orion_error(identity = "sys.cmd_error")]
    OsCmd,
    #[orion_error(identity = "sys.io_error")]
    Io,
    #[orion_error(identity = "biz.gxl_error")]
    Gxl,
    #[orion_error(identity = "sys.serv_error")]
    Serv,
    #[orion_error(identity = "logic.assert_fail")]
    Assert,
    #[orion_error(identity = "biz.args_error")]
    Args,
    #[orion_error(identity = "biz.miss")]
    Miss,
    #[orion_error(identity = "sys.serde_error")]
    Serde,
    #[orion_error(transparent)]
    Uvs(UvsReason),
    #[orion_error(identity = "biz.sec_error")]
    Sec,

    #[orion_error(identity = "sys.network_error")]
    NetWork,
}

impl From<reqwest::Error> for ExecReason {
    fn from(_value: reqwest::Error) -> Self {
        ExecReason::NetWork
    }
}

pub type ExecError = StructError<ExecReason>;
pub type ExecResult<T> = Result<T, ExecError>;

impl From<UvsReason> for ExecReason {
    fn from(value: UvsReason) -> Self {
        Self::Uvs(value)
    }
}

impl ExecReason {
    pub fn from_conf() -> Self {
        Self::core_conf()
    }

    pub fn from_res() -> Self {
        Self::resource_error()
    }

    pub fn from_logic() -> Self {
        Self::logic_error()
    }

    pub fn from_data() -> Self {
        Self::data_error()
    }
}

impl From<SerdeReason> for ExecReason {
    fn from(value: SerdeReason) -> Self {
        let _ = value;
        ExecReason::Serde
    }
}

impl From<OrionSecReason> for ExecReason {
    fn from(value: OrionSecReason) -> Self {
        match value {
            OrionSecReason::Sec(_sec_reason) => Self::Sec,
            OrionSecReason::General(uvs_reason) => Self::Uvs(map_legacy_uvs_reason(&uvs_reason)),
        }
    }
}

fn map_legacy_uvs_reason(value: &impl std::fmt::Debug) -> UvsReason {
    let debug = format!("{value:?}");
    match debug.as_str() {
        "ValidationError" => UvsReason::ValidationError,
        "BusinessError" => UvsReason::BusinessError,
        "RunRuleError" => UvsReason::RunRuleError,
        "NotFoundError" => UvsReason::NotFoundError,
        "PermissionError" => UvsReason::PermissionError,
        "DataError" => UvsReason::DataError,
        "SystemError" => UvsReason::SystemError,
        "NetworkError" => UvsReason::NetworkError,
        "ResourceError" => UvsReason::ResourceError,
        "TimeoutError" => UvsReason::TimeoutError,
        "ExternalError" => UvsReason::ExternalError,
        "LogicError" => UvsReason::LogicError,
        "ConfigError(Core)" => UvsReason::core_conf(),
        "ConfigError(Feature)" => UvsReason::feature_conf(),
        "ConfigError(Dynamic)" => UvsReason::dynamic_conf(),
        _ => UvsReason::SystemError,
    }
}
