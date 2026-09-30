use crate::{
    ExecReason, ExecResult,
    traits::{Getter, Setter},
    var::VarDict,
};
use orion_error::conversion::ToStructError;
use orion_sec::sec::{NoSecConv, SecFrom, SecValueType};
use regex::{Captures, Regex};
use std::env;
#[allow(unused_imports)]
use std::io::prelude::*;

pub trait VarParser<T> {
    fn eval(&self, content: T) -> ExecResult<String>;
    fn safe_eval(&self, content: T) -> String;
    fn sec_eval(&self, content: T) -> ExecResult<String>;
}

pub struct EnvExpress {
    regex: Regex,
    data: VarDict,
}
impl Default for EnvExpress {
    fn default() -> Self {
        EnvExpress::from_env()
    }
}

impl EnvExpress {
    pub fn new(data: VarDict) -> EnvExpress {
        let regex = Regex::new(r"(\$\{([[:alnum:]_\.]+(?:\[[^\]]*\])?)(?::([^}]*))?\})")
            .expect("EnvExpress Regex new fail!");
        EnvExpress { regex, data }
    }
    #[allow(dead_code)]
    pub fn from_env() -> EnvExpress {
        let mut data = VarDict::global_new();
        for (key, value) in env::vars() {
            data.set(&key, value);
        }
        EnvExpress::new(data)
    }
    pub fn from_env_mix(map: VarDict) -> EnvExpress {
        let mut data = VarDict::global_new();
        for (key, value) in env::vars() {
            data.set(&key, value);
        }
        data.merge_dict(map);
        EnvExpress::new(data)
    }
    pub fn insert_nor<S>(&mut self, key: String, val: S)
    where
        SecValueType: SecFrom<S>,
    {
        self.data.set(&key, SecValueType::nor_from(val));
    }
    pub fn insert_from<S>(&mut self, key: String, val: S)
    where
        SecValueType: From<S>,
    {
        self.data.set(&key, SecValueType::from(val));
    }
    pub fn eval_val(&self, key: &str) -> Option<String> {
        self.data
            .get_copy(key)
            .map(|v| v.clone().no_sec().to_string())
    }
    pub fn sec_val(&self, key: &str) -> Option<String> {
        self.data.get_copy(key).map(|v| v.to_string())
    }
    pub fn safe_eval_val(&self, key: &str) -> String {
        if let Some(val) = self.eval_val(key) {
            val.clone()
        } else {
            //error!("{}", self.data);
            format!("__NO[{key}]__")
        }
    }
    pub fn sec_eval_val(&self, key: &str) -> String {
        if let Some(val) = self.sec_val(key) {
            return val.clone();
        }
        format!("__NO[{key}]__",)
    }
}

impl VarParser<&String> for EnvExpress {
    fn eval(&self, content: &String) -> ExecResult<String> {
        self.eval(content.as_str())
    }
    fn safe_eval(&self, content: &String) -> String {
        self.safe_eval(content.as_str())
    }
    fn sec_eval(&self, content: &String) -> ExecResult<String> {
        self.sec_eval(content.as_str())
    }
}

impl VarParser<&str> for EnvExpress {
    fn safe_eval(&self, content: &str) -> String {
        let fun = |caps: &Captures| {
            if let Some(val) = self.eval_val(&caps[2]) {
                val
            } else if let Some(default) = caps.get(3) {
                default.as_str().to_string()
            } else {
                format!("__NO[{}]__", &caps[2])
            }
        };
        let mut target = content.to_string();
        loop {
            if self.regex.find(target.as_str()).is_none() {
                break;
            }
            let new_string = self.regex.replace_all(target.as_str(), &fun).to_string();
            // target not change , need break;
            if new_string == target {
                break;
            }
            target = new_string;
        }
        target
    }

    fn eval(&self, content: &str) -> ExecResult<String> {
        let target = self.safe_eval(content);
        if target.contains("__NO") {
            return Err(ExecReason::Miss.to_err().with_detail(target));
        }
        Ok(target)
    }
    // 对传入的content进行正则表达式匹配和替换操作，返回替换后的字符串
    fn sec_eval(&self, content: &str) -> ExecResult<String> {
        // 定义一个闭包，用于替换匹配到的字符串
        let fun = |caps: &Captures| self.sec_eval_val(&caps[2]);
        // 将content转换为字符串
        let mut target = content.to_string();
        // 循环进行正则表达式匹配和替换操作
        loop {
            // 如果没有匹配到正则表达式，则跳出循环
            if self.regex.find(target.as_str()).is_none() {
                break;
            }
            // 使用闭包进行替换操作
            target = self.regex.replace_all(target.as_str(), &fun).to_string();
        }
        // 如果替换后的字符串中包含"__NO"，则返回错误
        if target.contains("__NO") {
            return Err(ExecReason::Miss.to_err().with_detail(target));
        }
        // 返回替换后的字符串
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use crate::str_map;

    use super::*;
    #[test]
    pub fn regex_basic() {
        let re = Regex::new(r"(\$\{([[:alnum:]]+)\})").unwrap();
        let fun = |caps: &Captures| caps[2].to_string();
        let newc = re.replace_all("${HOME}/bin", &fun);
        assert_eq!("HOME/bin", newc);
        let newc = re.replace_all("${HOME}/bin/${USER}", &fun);
        assert_eq!("HOME/bin/USER", newc);
        let newc = re.replace_all("${HOME/bin", &fun);
        assert_eq!("${HOME/bin", newc);

        let newc = re.replace_all("{HOME}/bin", &fun);
        assert_eq!("{HOME}/bin", newc);
    }
    #[test]
    pub fn array_variables() {
        // 准备测试数据：包含数组的变量
        let mut data = VarDict::default();
        data.set(
            "SYS",
            SecValueType::List(vec![
                SecValueType::nor_from("Linux".to_string()),
                SecValueType::nor_from("Windows".to_string()),
            ]),
        );

        let ex = EnvExpress::new(data);

        // 测试解析数组元素
        assert_eq!(ex.eval("OS: ${SYS[0]}").unwrap(), "OS: Linux");
        assert_eq!(ex.eval("OS: ${SYS[1]}").unwrap(), "OS: Windows");

        // 测试无效索引
        assert!(ex.eval("OS: ${SYS[2]}").is_err()); // 越界
        assert!(ex.eval("OS: ${SYS[abc]}").is_err()); // 非数字索引
    }

    #[test]
    pub fn simple_eval() {
        let data = str_map!(
        "HOME" => "/home/galaxy",
        "USER" => "galaxy"
        );

        let ex = EnvExpress::new(VarDict::from(data));
        assert_eq!(
            ex.eval("${HOME}/bin").unwrap(),
            String::from("/home/galaxy/bin")
        );
        assert!(ex.eval("${HOME}/bin").unwrap() != "/home/galaxy1/bin");
        assert_eq!(
            ex.eval("${HOME}/${USER}/bin").unwrap(),
            String::from("/home/galaxy/galaxy/bin")
        );
        assert_eq!(
            ex.eval("${HOME2}"),
            Err(ExecReason::Miss.to_err().with_detail("__NO[HOME2]__"))
        );
        assert_eq!(ex.eval("HOME2").unwrap(), String::from("HOME2"));

        let content = "HOME2".to_string();
        assert_eq!(ex.eval(&content).unwrap(), String::from("HOME2"));
    }

    #[test]
    pub fn nested_eval() {
        let data = str_map!(
        "HOME"      => "/home/galaxy",
        "USER"      => "galaxy",
        "CUR_DIR"   => "${HOME}/prj",
        );
        let ex = EnvExpress::from_env_mix(VarDict::from(data));
        assert_eq!(
            ex.eval("${HOME}/bin").unwrap(),
            String::from("/home/galaxy/bin")
        );
        println!("{}", ex.eval("${CUR_DIR}/bin").unwrap());
        assert_eq!(
            ex.eval("${CUR_DIR}/bin").unwrap(),
            String::from("/home/galaxy/prj/bin")
        );
    }
    #[test]
    pub fn default_values() {
        let data = str_map!(
            "EXISTING" => "value",
            "EMPTY" => ""
        );

        let ex = EnvExpress::new(VarDict::from(data));

        // Existing variable
        assert_eq!(ex.eval("${EXISTING:default}").unwrap(), "value");

        // Missing variable with default
        assert_eq!(
            ex.eval("${MISSING:default_value}").unwrap(),
            "default_value"
        );

        // Empty variable with default
        assert_eq!(ex.eval("${EMPTY:default}").unwrap(), "");

        // Missing variable without default
        assert!(ex.eval("${MISSING}").is_err());

        // Array with default
        assert_eq!(
            ex.eval("${MISSING[0]:array_default}").unwrap(),
            "array_default"
        );

        // Complex default value
        assert_eq!(
            ex.eval("${MISSING:default with spaces and $pecial!}")
                .unwrap(),
            "default with spaces and $pecial!"
        );
    }
}
