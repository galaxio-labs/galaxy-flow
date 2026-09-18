use crate::{ExecReason, ExecResult, const_val::gxl_const, error::AssembleReason};
use orion_error::reason::UnifiedReason as UvsReason;
use orion_error::{OrionError, StructError};

use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, OrionError)]
pub enum RunReason {
    #[orion_error(identity = "biz.gxl_error")]
    Gxl,
    #[orion_error(identity = "sys.exec_error")]
    Exec,
    #[orion_error(identity = "biz.args_error")]
    Args,
    #[orion_error(identity = "biz.sec_error")]
    Sec,
    #[orion_error(transparent)]
    Uvs(UvsReason),
}

pub type RunError = StructError<RunReason>;
pub type RunResult<T> = Result<T, RunError>;

impl From<UvsReason> for RunReason {
    fn from(value: UvsReason) -> Self {
        Self::Uvs(value)
    }
}

impl RunReason {
    pub fn from_conf() -> Self {
        Self::core_conf()
    }

    pub fn from_sys() -> Self {
        Self::system_error()
    }

    pub fn from_res() -> Self {
        Self::resource_error()
    }
}

#[derive(Debug, PartialEq, Serialize, OrionError)]
pub enum GxlReason {
    #[orion_error(identity = "biz.gxl_parse_error")]
    Parse,
    #[orion_error(identity = "biz.gxl_depend_error")]
    Depend,
    #[orion_error(identity = "biz.gxl_less_error")]
    Less,
    #[orion_error(identity = "biz.gxl_none")]
    None,
    #[orion_error(transparent)]
    Uvs(UvsReason),
}

impl From<UvsReason> for GxlReason {
    fn from(value: UvsReason) -> Self {
        Self::Uvs(value)
    }
}

impl GxlReason {
    pub fn from_logic() -> Self {
        Self::logic_error()
    }
}

pub type GxlError = StructError<GxlReason>;
pub type GxlResult<T> = std::result::Result<T, GxlError>;
pub type NER = ExecResult<()>;

pub fn report_gxl_error(e: RunError) {
    eprintln!("Galaxy Flow Parse Error");
    eprintln!("--------------------------");
    if let Some(target) = e.target_path() {
        eprintln!("[TARGET]:\n{target}\n",);
    }
    eprintln!("[REASON]:");
    match e.reason() {
        RunReason::Uvs(uvs_reason) => match uvs_reason {
            UvsReason::LogicError => {
                eprintln!("LOGIC ERROR\n",);
            }
            UvsReason::BusinessError => {
                eprintln!("BIZ ERROR\n",);
            }
            UvsReason::DataError => {
                eprintln!("DATA ERROR\n",);
            }
            UvsReason::SystemError => {
                eprintln!("SYS ERROR\n",);
            }
            UvsReason::ResourceError => {
                eprintln!("RES ERROR\n",);
            }
            UvsReason::NetworkError => {
                eprintln!("NET ERROR\n",);
            }
            UvsReason::TimeoutError => {
                eprintln!("TIMEOUT\n",);
            }
            UvsReason::ConfigError(e) => {
                eprintln!("CONF ERROR: {e}\n",);
            }
            UvsReason::PermissionError => {
                eprintln!("PERMISSION ERROR\n",);
            }
            UvsReason::ValidationError => {
                eprintln!("VALIDATION ERROR\n",);
            }
            UvsReason::ExternalError => {
                eprintln!("EXTERNAL ERROR\n",);
            }
            UvsReason::NotFoundError => {
                eprintln!("NOT FOUND\n",);
            }
            other => {
                eprintln!("ERROR: {other}\n",);
            }
        },
        RunReason::Gxl => {
            eprintln!("{}GXL ERROR\n", gxl_const::ERROR_PREFIX);
        }
        RunReason::Exec => {
            eprintln!("EXEC ERROR\n",);
        }
        RunReason::Args => {
            eprintln!("ARGS ERROR\n",);
        }
        RunReason::Sec => {
            eprintln!("Sec ERROR\n",);
        }
    }
    if let Some(pos) = e.position() {
        eprintln!("\n[POSITION]:\n{pos}",);
    }
    if let Some(detail) = e.detail() {
        eprintln!("\n[DETAIL]:\n{detail}",);
    }
    eprintln!("\n[CONTEXT]:\n");
    for x in e.contexts().iter() {
        eprintln!("{x}")
    }
}

impl From<ExecReason> for RunReason {
    fn from(value: ExecReason) -> Self {
        match value {
            ExecReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            _ => RunReason::Exec,
        }
    }
}
impl From<RunReason> for ExecReason {
    fn from(value: RunReason) -> Self {
        match value {
            RunReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            _ => Self::Args,
        }
    }
}

impl From<AssembleReason> for RunReason {
    fn from(_value: AssembleReason) -> Self {
        RunReason::Gxl
    }
}
