use super::super::prelude::*;

use super::call::action_call_args;
use super::common::shell_opt_setting;
use winnow::combinator::fail;

use winnow::Parser;
use winnow::Result;

use crate::ability::GxRead;
use crate::ability::read::CmdDTOBuilder;
use crate::ability::read::FileDTOBuilder;
use crate::ability::read::ReadMode;
use crate::ability::read::StdinDTO;
use crate::expect::ShellOption;
use crate::parser::domain::gal_keyword;
use crate::parser::domain::gal_keyword_alt;

pub fn gal_read_file(input: &mut &str) -> Result<GxRead> {
    gal_keyword_alt("gx.read_file", "rg.read_file", input)?;
    let props = action_call_args.parse_next(input)?;
    let mut builder = FileDTOBuilder::default();
    for one in props {
        let key = one.0.to_lowercase();
        if key == "name" {
            builder.name(Some(one.1));
        } else if key == "default" || key == "file" {
            builder.file(one.1);
        } else if key == "entity" {
            builder.entity(Some(one.1));
        }
    }
    match builder.build() {
        Ok(dto) => Ok(GxRead::from(ReadMode::from(dto))),
        Err(e) => {
            error!(target: "parse", "{e}",);
            fail.context(wn_desc("read")).parse_next(input)
        }
    }
}
pub fn gal_read_stdin(input: &mut &str) -> Result<GxRead> {
    gal_keyword_alt("gx.read_stdin", "rg.read_stdin", input)?;
    let props = action_call_args.parse_next(input)?;
    let mut dto = StdinDTO::default();
    for one in props {
        let key = one.0.to_lowercase();
        if key == "name" {
            dto.set_name(one.1);
        } else if key == "prompt" {
            dto.set_prompt(one.1);
        }
    }
    match !dto.name().is_empty() {
        true => Ok(GxRead::from(ReadMode::from(dto))),
        false => fail
            .context(wn_desc("read(name is empty)"))
            .parse_next(input),
    }
}

pub fn gal_read_cmd(input: &mut &str) -> Result<GxRead> {
    gal_keyword("gx.read_cmd", input)?;
    let props = action_call_args.parse_next(input)?;
    let mut builder = CmdDTOBuilder::default();
    let mut shell_opt = ShellOption::default();
    builder.shell_opt(shell_opt.clone());
    for one in props {
        let key = one.0.to_lowercase();
        if key == "name" {
            builder.name(one.1);
        } else if key == "cmd" {
            builder.cmd(one.1);
        } else {
            shell_opt_setting(key, one.1, &mut shell_opt);
        }
    }
    builder.shell_opt(shell_opt);
    match builder.build() {
        Ok(dto) => Ok(GxRead::from(ReadMode::from(dto))),
        Err(e) => {
            error!(target: "parse", "{e}");
            fail.context(wn_desc("read_cmd")).parse_next(input)
        }
    }
}
#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::{
        ability::read::{CmdDTO, FileDTO, StdinDTO},
        infra::once_init_log,
        parser::inner::common::run_gxl,
    };

    use super::*;

    #[test]
    fn read_cmd_test1() {
        once_init_log();
        let dto = CmdDTO {
            shell_opt: ShellOption::default(),
            cmd: "echo galaxy-1.0".to_string(),
            name: "RG".to_string(),
        };
        let mut data = r#"
                 gx.read_cmd (
                 name : "RG",
                 cmd  : "echo galaxy-1.0",
                 ) ;"#;
        let obj = run_gxl(gal_read_cmd, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&ReadMode::from(dto), obj.imp());
    }
    #[test]
    fn read_cmd_test2() {
        let mut dto = CmdDTO {
            shell_opt: ShellOption::default(),
            ..Default::default()
        };
        dto.shell_opt.log_lev = Some(log::Level::Info);

        dto.cmd = "echo galaxy-1.0".to_string();
        dto.name = "RG".to_string();
        dto.shell_opt.err = Some("you err".into());

        let mut data = r#"
                 gx.read_cmd(
                 name : "RG",
                 cmd  : "echo galaxy-1.0",
                 err : "you err",
                 log : "1",
                 ) ;"#;

        let obj = run_gxl(gal_read_cmd, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&ReadMode::from(dto), obj.imp());
    }
    #[test]
    fn read_file_test() {
        let dto = FileDTO {
            file: "vars.ini".to_string(),
            ..Default::default()
        };

        let mut data = r#"
                 gx.read_file (
                 file : "vars.ini"
                 ) ;"#;
        let obj = run_gxl(gal_read_file, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&ReadMode::from(dto), obj.imp());
    }
    #[test]
    fn read_file_default() {
        let dto = FileDTO {
            file: "vars.ini".to_string(),
            ..Default::default()
        };

        let mut data = r#"
                 gx.read_file (  "vars.ini" ) ;"#;
        let obj = run_gxl(gal_read_file, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&ReadMode::from(dto), obj.imp());
    }

    #[test]
    fn read_stdin_test() {
        let dto = StdinDTO::default()
            .with_name("name".into())
            .with_prompt("please input you name".into());

        let mut data = r#"
                 gx.read_stdin (
                 prompt : "please input you name",
                 name  : "name",
                 ) ;"#;

        let obj = run_gxl(gal_read_stdin, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&ReadMode::from(dto), obj.imp());
    }
}
