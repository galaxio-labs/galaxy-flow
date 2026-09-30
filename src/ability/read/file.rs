use std::path::PathBuf;

use crate::ability::prelude::*;
use crate::traits::Setter;

use orion_conf::{IniIO, JsonIO, YamlIO};
use orion_error::conversion::{SourceErr, ToStructError};
use orion_sec::sec::{SecFrom, SecValueType};
use orion_variate::vars::ValueType;

#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct FileDTO {
    pub file: String,
    #[builder(default = "None")]
    pub name: Option<String>,
    #[builder(default = "None")]
    pub entity: Option<String>,
}

impl FileDTO {
    pub fn execute(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.read_file");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let file = self.file.clone();
        let file_path = PathBuf::from(exp.eval(&file)?);
        let values = if file_path.extension() == PathBuf::from("*.ini").extension() {
            ValueType::load_ini(&file_path)
                .source_err(UvsReason::data_error().into(), "source error")?
        } else if file_path.extension() == PathBuf::from("*.json").extension() {
            ValueType::load_json(&file_path)
                .source_err(UvsReason::data_error().into(), "source error")?
        } else if file_path.extension() == PathBuf::from("*.yml").extension() {
            ValueType::load_yaml(&file_path)
                .source_err(UvsReason::data_error().into(), "source error")?
        } else {
            return Err(ExecReason::Args
                .to_err()
                .with_detail(format!("not support format :{}", file_path.display())));
        };
        info!(target: ctx.path(),"read file from {} data len:{}", file_path.display(), values.len());
        let sec_values = SecValueType::nor_from(values);
        match sec_values {
            SecValueType::Obj(obj) => {
                if let Some(name) = self.name.clone() {
                    vars_dict.global_mut().set(name, SecValueType::from(obj));
                } else {
                    vars_dict.global_mut().merge(obj);
                }
            }
            SecValueType::List(list) => {
                if let Some(name) = self.name.clone() {
                    vars_dict.global_mut().set(name, SecValueType::from(list));
                } else {
                    return Err(ExecReason::from_logic()
                        .to_err()
                        .with_detail("list cannot set to VarSpace by no name "));
                }
            }
            _ => {
                return Err(ExecReason::from_logic()
                    .to_err()
                    .with_detail("read file only support list and map "));
            }
        }
        Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
    }
}

#[cfg(test)]
mod tests {

    use orion_sec::sec::SecFrom;

    use super::*;
    use crate::ability::{read::integra::ReadMode, *};

    #[tokio::test]
    async fn read_ini_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/examples/read");
        let dto = FileDTO {
            file: String::from("${CONF_ROOT}/var.ini"),
            ..Default::default()
        };
        let res = GxRead::from(ReadMode::from(dto));
        let TaskValue { vars, .. } = res.async_exec(context, def).await.unwrap();
        assert_eq!(
            vars.get("RUST"),
            Some(SecValueType::nor_from("100".to_string()))
        );
        assert_eq!(
            vars.get("JAVA"),
            Some(SecValueType::nor_from("90".to_string()))
        );
    }
    #[tokio::test]
    async fn read_json_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/examples/read");
        let dto = FileDTO {
            file: String::from("${CONF_ROOT}/var.json"),
            ..Default::default()
        };
        let res = GxRead::from(ReadMode::from(dto));
        let TaskValue { vars, .. } = res.async_exec(context, def).await.unwrap();
        assert_eq!(vars.get("RUST"), Some(SecValueType::nor_from(100)));
        assert_eq!(vars.get("JAVA"), Some(SecValueType::nor_from(90)));
    }

    #[tokio::test]
    async fn read_yaml_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/examples/read");
        let dto = FileDTO {
            file: String::from("${CONF_ROOT}/var.yml"),
            ..Default::default()
        };
        let res = GxRead::from(ReadMode::from(dto));
        let TaskValue { vars, .. } = res.async_exec(context, def).await.unwrap();
        assert_eq!(vars.get("MEMBER.RUST"), Some(SecValueType::nor_from(100)));
        assert_eq!(vars.get("MEMBER.JAVA"), Some(SecValueType::nor_from(90)));
    }
    #[tokio::test]
    async fn read_yaml_arr() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/examples/read");
        let dto = FileDTO {
            file: String::from("${CONF_ROOT}/var_arr.yml"),
            name: Some("DATA".to_string()),
            ..Default::default()
        };
        let res = GxRead::from(ReadMode::from(dto));
        let TaskValue { vars, .. } = res.async_exec(context, def).await.unwrap();
        assert_eq!(
            vars.get("DATA[1].MEMBER.RUST"),
            Some(SecValueType::nor_from(110))
        );
        assert_eq!(
            vars.get("DATA[1].MEMBER.JAVA"),
            Some(SecValueType::nor_from(190))
        );
    }
}
