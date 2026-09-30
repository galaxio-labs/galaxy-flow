use std::collections::HashMap;
pub type StrMap = HashMap<String, String>;

pub trait Mustable {
    fn must_get(&self, key: &str) -> &String;
}
impl Mustable for StrMap {
    fn must_get(&self, key: &str) -> &String {
        self.get(key)
            .unwrap_or_else(|| panic!("not get {key} value"))
    }
}

#[derive(Clone, Copy)]
pub enum LogicScope {
    Outer,
    Inner,
}

#[derive(Clone, Debug, Builder, PartialEq)]
pub struct ShellOption {
    pub quiet: bool,
    //用于Git 调用的内部命令
    pub inner_print: bool,
    pub sudo: bool,
    pub err: Option<String>,
    pub suc: Option<String>,
    pub secrecy: bool,
    pub ok_codes: Vec<i32>,
    pub log_lev: Option<log::Level>,
    pub stream: bool,
}
impl Default for ShellOption {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellOption {
    pub fn new() -> Self {
        ShellOption {
            quiet: false,
            inner_print: false,
            sudo: false,
            err: None,
            suc: None,
            secrecy: false,
            ok_codes: vec![0],
            log_lev: Some(log::Level::Info),
            stream: false,
        }
    }
    pub fn new_explicit(outer: bool, inner: bool) -> Self {
        ShellOption {
            quiet: outer,
            inner_print: inner,
            sudo: true,
            err: None,
            suc: None,
            secrecy: false,
            ok_codes: vec![0],
            log_lev: Some(log::Level::Info),
            stream: false,
        }
    }
    pub fn quiet(&self, scope: LogicScope) -> bool {
        match scope {
            LogicScope::Outer => self.quiet,
            LogicScope::Inner => self.inner_print,
        }
    }
}

#[macro_export]
macro_rules! str_map {
    {} => ($crate::expect::StrMap::new());
    // In this implementation, key/value pairs separated by commas.
    { $( $key:expr => $value:expr ),* } => {
         str_map!( $( $key => $value, )* )
    };

    { $( $key:expr => $value:expr, )* } => ({
        let mut map = $crate::expect::StrMap::new();
        $(
            map.insert(String::from($key), String::from($value));
         )*
        map
     })
}
