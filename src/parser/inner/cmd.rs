use super::common::shell_opt_setting;
use super::prelude::*;

use crate::ability::GxCmd;
use crate::ability::cmd::GxCmdDtoBuilder;
use crate::expect::ShellOption;
use crate::parser::domain::{gal_keyword, gal_keyword_alt};

pub fn gal_cmd(input: &mut &str) -> Result<GxCmd> {
    let mut builder = GxCmdDtoBuilder::default();
    gal_keyword_alt("gx.cmd", "rg.cmd", input)?;
    let props = action_call_args.parse_next(input)?;
    let mut shell_opt = ShellOption::default();
    builder.shell_opt(ShellOption::default());
    for one in props {
        let key = one.0.to_lowercase();
        if key == "default" || key == "cmd" {
            builder.cmd(one.1);
        } else {
            shell_opt_setting(key, one.1, &mut shell_opt);
        }
    }
    builder.shell_opt(shell_opt);
    if let Ok(dto) = builder.build() {
        Ok(GxCmd::dto_new(dto))
    } else {
        fail.parse_next(input)
    }
}

/// read ```cmd  ... ``` to GxCmd;
pub fn gal_cmd_block(input: &mut &str) -> Result<GxCmd> {
    let mut builder = GxCmdDtoBuilder::default();
    builder.shell_opt(ShellOption::default());
    // 1. 匹配开始的 ```cmd
    gal_keyword("```cmd", input)?;
    // 2. 跳过可能的空白和换行
    //*input = input.trim_start();
    let cmd_content = take_until(0.., "```")
        .context(wn_desc("cmd block"))
        .parse_next(input)?;
    "```".context(wn_desc("block-end")).parse_next(input)?;
    builder.cmd(format_shell_script(cmd_content));
    multispace0.parse_next(input)?;
    // 6. 构建并返回 GxCmd
    if let Ok(dto) = builder.build() {
        Ok(GxCmd::dto_new(dto))
    } else {
        fail.context(wn_desc("cmd-block build fail"))
            .parse_next(input)
    }
}
fn format_shell_script(input: &str) -> String {
    // 替换转义的引号和换行符
    let input = input.replace(r#"\""#, r#"""#);
    let input = input.replace(r#"\n"#, "\n");

    // 分成几行并处理每一行
    let lines: Vec<&str> = input.lines().collect();

    // 找到最小缩进
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    // 删除每行的最小缩进
    lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim() // 删除任何尾随空格
        .to_string()
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::parser::{inner::common::run_gxl, stc_blk::gal_block};

    use super::*;

    #[test]
    fn cmd_test_simple() {
        let shell_opt = ShellOption::default();
        let mut data = r#"
             gx.cmd(
             cmd : "${PRJ_ROOT}/do.sh",
             ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        //let (input, obj) = show_err(data, RgCmdParser::default().parse(ctx, data)).unwrap();
        let xpt = GxCmdDtoBuilder::default()
            .cmd("${PRJ_ROOT}/do.sh".into())
            .shell_opt(shell_opt)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }
    #[test]
    fn cmd_test_default() {
        let shell_opt = ShellOption::default();
        let mut data = r#"
             gx.cmd( "${PRJ_ROOT}/do.sh" ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        //let (input, obj) = show_err(data, RgCmdParser::default().parse(ctx, data)).unwrap();
        let xpt = GxCmdDtoBuilder::default()
            .cmd("${PRJ_ROOT}/do.sh".into())
            .shell_opt(shell_opt)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }
    #[test]
    fn cmd_test_raw_string() {
        let shell_opt = ShellOption::default();
        let mut data = "
             gx.cmd(
               cmd  : r#\"git branch --show-current |  sed -E \"s/(feature|develop|ver-dev|release|master|issue)(\\/.*)?/_branch_\\1/g\" \"# ,
             ) ;";
        let obj = gal_cmd(&mut data).assert();
        //let (input, obj) = show_err(data, RgCmdParser::default().parse(ctx, data)).unwrap();
        let xpt = GxCmdDtoBuilder::default()
            .cmd( r#"git branch --show-current |  sed -E "s/(feature|develop|ver-dev|release|master|issue)(\/.*)?/_branch_\1/g" "#
                    .into())
            .shell_opt(shell_opt)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }

    #[test]
    fn cmd_test2() {
        let mut shell_opt = ShellOption {
            log_lev: Some(log::Level::Info),
            ..Default::default()
        };
        let mut data = r#"
             gx.cmd(
             cmd : "${PRJ_ROOT}/do.sh",
             err : "you err",
             log : "1",
             ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        shell_opt.err = Some(String::from("you err"));
        let xpt = GxCmdDtoBuilder::default()
            .cmd("${PRJ_ROOT}/do.sh".into())
            .shell_opt(shell_opt)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }

    #[test]
    fn cmd_test3() {
        let mut shell_opt = ShellOption {
            log_lev: Some(log::Level::Info),
            ..Default::default()
        };
        let mut data = r#"
             gx.cmd(
             "${PRJ_ROOT}/do.sh",
             err : "you err",
             log : "1",
             ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        shell_opt.err = Some(String::from("you err"));
        let xpt = GxCmdDtoBuilder::default()
            .cmd("${PRJ_ROOT}/do.sh".into())
            .shell_opt(shell_opt)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }

    #[test]
    fn cmd_test_ok_codes() {
        let shell_opt = ShellOption {
            ok_codes: vec![0, 2],
            ..Default::default()
        };
        let mut data = r#"
             gx.cmd(
             cmd : "echo ok",
             ok_codes : "0,2",
             ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        let xpt = GxCmdDtoBuilder::default()
            .cmd("echo ok".into())
            .shell_opt(shell_opt)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }

    #[test]
    fn cmd_test_stream() {
        let shell_opt = ShellOption {
            stream: true,
            ..Default::default()
        };
        let mut data = r#"
             gx.cmd(
             cmd : "echo ok",
             stream : "true",
             ) ;"#;
        let obj = gal_cmd(&mut data).assert();
        let xpt = GxCmdDtoBuilder::default()
            .cmd("echo ok".into())
            .shell_opt(shell_opt)
            .build()
            .unwrap();
        assert_eq!(data, "");
        assert_eq!(obj, GxCmd::dto_new(xpt));
    }

    #[test]
    fn cmd_block_1() {
        let mut data = r#"```cmd echo ${HOME};```"#;
        let obj = run_gxl(gal_cmd_block, &mut data).assert();
        assert_eq!(obj.dto().cmd, "echo ${HOME};");

        assert_eq!(data, "");
    }
    #[test]
    fn cmd_block_2() {
        let mut data = r#"```cmd
echo ${HOME};
echo ${HOME};
            ```"#;
        let obj = run_gxl(gal_cmd_block, &mut data).assert();
        assert_eq!(obj.dto().cmd, "echo ${HOME};\necho ${HOME};");

        assert_eq!(data, "");
    }
    #[test]
    fn cmd_block_3() {
        let mut data = r#"
            {
            gx.cmd ( cmd : "echo ${HOME}" )
            ```cmd
                cp /a /b;
                echo ${HOME};
                echo ${HOME};
            ```
            }
            "#;
        let obj = run_gxl(gal_block, &mut data).assert();
        assert_eq!(obj.items().len(), 2);
        assert_eq!(data, "");
    }
}
