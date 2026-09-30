use derive_more::From;
use orion_error::conversion::ToStructError;
use orion_error::reason::UnifiedReason as UvsReason;
use orion_sec::load_secfile;
use orion_sec::sec::{SecFrom, SecString, SecValueType, ValueGetter};
use orion_variate::vars::EnvDict;

use super::global::{setup_gxlrun_vars, setup_start_vars};
use crate::evaluator::{EnvExpress, VarParser};
use crate::{
    ExecReason, ExecResult,
    primitive::{GxlAParams, GxlFParams, GxlObject},
    traits::Setter,
    var::VarDict,
};

#[derive(Debug, Clone, Default, PartialEq, From, Getters)]
pub struct VarSpace {
    inherited: VarDict,
    global: VarDict,
    //nameds: HashMap<String, VarDict>,
}

impl From<&VarSpace> for EnvDict {
    fn from(value: &VarSpace) -> Self {
        EnvDict::from(value.global.export().clone())
    }
}

impl VarSpace {
    pub fn sys_init() -> ExecResult<VarSpace> {
        let mut var_space = VarSpace::default();
        let sec_dict = load_secfile().map_err(|e| {
            let detail = e.to_string();
            match e.reason() {
                orion_sec::OrionSecReason::Sec(_sec_reason) => {
                    ExecReason::Sec.to_err().with_detail(detail)
                }
                orion_sec::OrionSecReason::General(uvs_reason) => {
                    ExecReason::from(map_legacy_uvs_reason(uvs_reason))
                        .to_err()
                        .with_detail(detail)
                }
            }
        })?;
        //let sec_dict = load_secfile()?;
        var_space.inherited = VarDict::from(sec_dict);
        //load_secfile(&mut var_space.inherited)?;
        setup_start_vars(&mut var_space.inherited)?;
        setup_gxlrun_vars(&mut var_space.inherited)?;
        var_space.global = var_space.inherited.clone();
        Ok(var_space)
    }
    pub fn inherit_init(mut origin: VarSpace, isolate: bool) -> ExecResult<VarSpace> {
        if isolate {
            let mut ins = Self {
                inherited: origin.inherited.clone(),
                global: origin.inherited,
            };
            setup_gxlrun_vars(&mut ins.inherited)?;
            Ok(ins)
        } else {
            setup_gxlrun_vars(&mut origin.inherited)?;
            setup_gxlrun_vars(&mut origin.global)?;
            Ok(origin)
        }
    }
    pub fn global_mut(&mut self) -> &mut VarDict {
        &mut self.global
    }
    pub fn get(&self, path: &str) -> Option<SecValueType> {
        self.global().maps().value_get(path)
    }
    pub fn must_get(&self, path: &str) -> ExecResult<SecValueType> {
        self.global()
            .maps()
            .value_get(path)
            .ok_or_else(|| ExecReason::Miss.to_err().with_detail(path.to_string()))
    }

    pub fn merge_args_to(
        &self,
        f_params: &GxlFParams,
        a_params: &GxlAParams,
    ) -> ExecResult<VarSpace> {
        let mut cur_vars = self.clone();
        let exp = EnvExpress::from_env_mix(self.global().clone());
        for param in f_params {
            let found = if param.is_default() {
                //use default actura name
                a_params.get(param.name()).or(a_params.get("default"))
            } else {
                a_params.get(param.name())
            };
            if let Some(a_param) = found {
                match a_param.value() {
                    GxlObject::VarRef(name) => {
                        let value = cur_vars
                            .get(name.as_str())
                            .ok_or_else(|| ExecReason::Miss.to_err().with_detail(name.clone()))?;
                        cur_vars.global_mut().set(param.name().clone(), value);
                    }
                    GxlObject::Value(value) => {
                        // 先求值处理
                        let conv_value = if let SecValueType::String(str_val) = value {
                            let cur_value = exp.safe_eval(str_val.value().as_str());
                            if str_val.is_secret() {
                                SecValueType::String(SecString::sec_from(cur_value))
                            } else {
                                SecValueType::String(SecString::nor_from(cur_value))
                            }
                        } else {
                            value.clone()
                        };
                        cur_vars.global_mut().set(param.name().clone(), conv_value);
                    }
                }
            } else {
                //use formal default value;
                if let Some(default_v) = param.default_value() {
                    cur_vars.global_mut().set(param.name(), default_v.clone());
                }
            }
        }
        Ok(cur_vars)
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
#[derive(Debug, Clone, Default, PartialEq, From)]
pub enum DictUse {
    #[default]
    Global,
    Named(String),
}

#[cfg(test)]
mod tests {
    use crate::{traits::Setter, var::UpperStrMap};

    use orion_variate::vars::UpperKey;

    use super::*;

    #[test]
    fn test_get_top_level_key() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set(
            "name".to_string(),
            SecValueType::nor_from("Test User".to_string()),
        );

        let value = var_space.get("name");
        assert!(value.is_some());
        if let Some(SecValueType::String(s)) = value {
            assert_eq!(s.value(), "Test User");
        } else {
            panic!("Expected String variant");
        }
    }

    #[test]
    fn test_get_nested_key() {
        let mut var_space = VarSpace::default();

        // Create nested structure: user = { name: "Test", id: 42 }
        let mut user = UpperStrMap::new();
        user.insert(
            UpperKey::from("name"),
            SecValueType::nor_from("Test User".to_string()),
        );
        user.insert(UpperKey::from("id"), SecValueType::nor_from(42u64));

        var_space.global_mut().set("user", SecValueType::Obj(user));

        // Test nested access
        let name = var_space.get("user.name");
        assert!(name.is_some());
        if let Some(SecValueType::String(s)) = name {
            assert_eq!(s.value(), "Test User");
        } else {
            panic!("Expected String variant");
        }

        let id = var_space.get("user.id");
        assert!(id.is_some());
        if let Some(SecValueType::Number(n)) = id {
            assert_eq!(*n.value(), 42);
        } else {
            panic!("Expected Number variant");
        }
    }

    #[test]
    fn test_get_multi_level_nested_key() {
        let mut var_space = VarSpace::default();

        // Create nested structure: app.user.profile.name
        let mut profile = UpperStrMap::new();
        profile.insert(
            UpperKey::from("name"),
            SecValueType::nor_from("Test User".to_string()),
        );

        let mut user = UpperStrMap::new();
        user.insert(UpperKey::from("profile"), SecValueType::Obj(profile));

        var_space.global_mut().set("app", SecValueType::Obj(user));

        let name = var_space.get("app.profile.name");
        assert!(name.is_some());
        if let Some(SecValueType::String(s)) = name {
            assert_eq!(s.value(), "Test User");
        } else {
            panic!("Expected String variant");
        }
    }

    #[test]
    fn test_get_non_existent_key() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set("exists", "value");

        assert!(var_space.get("does_not_exist").is_none());
        assert!(var_space.get("exists.invalid").is_none());
    }

    #[test]
    fn test_get_key_in_middle_of_path() {
        let mut var_space = VarSpace::default();

        // Create structure: parent = { child: "value" }
        let mut parent = UpperStrMap::new();
        parent.insert(
            UpperKey::from("child"),
            SecValueType::nor_from("value".to_string()),
        );

        var_space
            .global_mut()
            .set("parent", SecValueType::Obj(parent));

        // Try to get intermediate object
        let parent_obj = var_space.get("parent");
        assert!(parent_obj.is_some());
        if let Some(SecValueType::Obj(_)) = parent_obj {
            // This is expected
        } else {
            panic!("Expected Obj variant");
        }

        // Try to get non-existent child
        assert!(var_space.get("parent.invalid").is_none());
    }

    #[test]
    fn test_get_with_empty_path() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set("valid", "value");

        assert!(var_space.get("").is_none());
    }

    #[test]
    fn test_get_after_mutation() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set("counter", "1");

        // Initial value
        let initial = var_space.get("counter");
        assert!(initial.is_some());

        // Update value
        var_space.global_mut().set("counter", "2");

        // Get updated value
        let updated = var_space.get("counter");
        assert!(updated.is_some());
        if let Some(SecValueType::String(s)) = updated {
            assert_eq!(s.value(), "2");
        } else {
            panic!("Expected String variant");
        }
    }

    #[test]
    fn test_get_with_secret_values() {
        let mut var_space = VarSpace::default();

        // Create secret value
        let secret = SecString::sec_from("secret-value".to_string());

        var_space
            .global_mut()
            .set("api_key".to_string(), SecValueType::String(secret));

        let value = var_space.get("api_key");
        assert!(value.is_some());
        if let Some(SecValueType::String(s)) = value {
            assert!(s.is_secret());
            assert_eq!(s.value(), "secret-value");
        } else {
            panic!("Expected String variant");
        }
    }
}
