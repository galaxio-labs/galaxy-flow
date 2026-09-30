#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate regex;
#[allow(unused_imports)]
#[macro_use]
extern crate duct;
extern crate duct_sh;
#[macro_use]
extern crate derive_getters;
#[macro_use]
extern crate mockall;
extern crate once_cell;
extern crate shells;
extern crate toml;
#[macro_use]
extern crate derive_builder;
extern crate handlebars;
// extern crate json ;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

//extern crate ini;
extern crate url;

//#[macro_use]
//extern crate simple_log;

#[macro_use]
pub mod err;
#[macro_use]
pub mod model;
pub mod ability;
pub mod calculate;
pub mod cli;
pub mod cmd;
pub mod conf;
pub mod const_val;
pub mod debug;
mod evaluator;
pub mod friendly;
mod loader;
pub mod menu;
pub mod parser;
#[macro_use]
pub mod util;
pub mod galaxy;
pub mod help;
pub mod infra;
pub mod runner;
pub mod self_update;
pub mod skills;
pub mod types;
pub use crate::loader::{GxLoader, get_parse_code};
pub use model::*;
