use super::super::prelude::*;
use super::call::action_call_args;
use super::common::shell_opt_setting;

use crate::ability::shell::GxShell;
use crate::expect::ShellOption;
use crate::parser::domain::gal_keyword;
use crate::util::OptionFrom;

pub fn gal_shell(input: &mut &str) -> Result<GxShell> {
    let mut shell = GxShell::default();
    gal_keyword("gx.shell", input)?;
    let props = action_call_args.parse_next(input)?;
    let mut shell_opt = ShellOption::default();
    shell.set_shell_opt(ShellOption::default());
    for one in props {
        let key = one.0.to_lowercase();
        if key == "default" || key == "shell" {
            shell.set_shell(one.1.to_string());
        } else if key == "arg_file" {
            shell.set_arg_file(one.1.to_opt());
        } else if key == "out_var" {
            shell.set_out_var(one.1.to_opt());
        } else {
            shell_opt_setting(key, one.1, &mut shell_opt);
        }
    }
    shell.set_shell_opt(shell_opt);
    if !shell.shell().is_empty() {
        Ok(shell)
    } else {
        fail.parse_next(input)
    }
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use super::*;

    #[test]
    fn cmd_test_simple() {
        let mut data = r#"
             gx.shell( shell: "do.sh" ) ;"#;
        let obj = gal_shell(&mut data).assert();
        //let (input, obj) = show_err(data, RgCmdParser::default().parse(ctx, data)).unwrap();
        assert_eq!(data, "");
        assert_eq!(obj.shell(), "do.sh");
    }
    #[test]
    fn cmd_test_default() {
        let mut data = r#"
             gx.shell( "do.sh" ) ;"#;
        let obj = gal_shell(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.shell(), "do.sh");
    }

    #[test]
    fn cmd_test2() {
        let shell_opt = ShellOption {
            log_lev: Some(log::Level::Info),
            ..Default::default()
        };
        let mut data = r#"
             gx.shell(
             shell: "do.sh",
             arg_file: "arg.json",
             out_var: "OUT_NAME",
             ) ;"#;
        let obj = gal_shell(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.shell(), "do.sh");
        assert_eq!(obj.arg_file(), &"arg.json".to_opt());
        assert_eq!(obj.out_var(), &"OUT_NAME".to_opt());
        assert_eq!(obj.shell_opt(), &shell_opt);
    }
}
