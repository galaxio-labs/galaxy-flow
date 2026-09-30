use super::prelude::*;
use orion_parse::symbol::symbol_comma;
use winnow::combinator::separated;

use crate::ability::parse_ok_codes;
use crate::components::{GxlProps, gxl_var::*};
use crate::expect::ShellOption;
use crate::parser::abilities::define::gal_var_assign_obj;
use crate::parser::abilities::param::gal_formal_param;

pub fn gal_vars(input: &mut &str) -> Result<GxlProps> {
    let mut vars = GxlProps::default();
    gal_keyword("gx.vars", input)?;
    let founds = sentence_body.parse_next(input)?;
    for one in founds {
        vars.insert(one.0, one.1);
    }
    Ok(vars)
}

pub fn object_props(input: &mut &str) -> Result<Vec<(String, String)>> {
    gal_block_beg.parse_next(input)?;
    let props: Vec<(String, String)> =
        separated(0.., gal_assign_exp, alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    opt(alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    gal_block_end.parse_next(input)?;
    Ok(props)
}

pub fn sentence_body(input: &mut &str) -> Result<Vec<(String, GxlObject)>> {
    gal_sentence_beg.parse_next(input)?;
    let args: Vec<(String, GxlObject)> = separated(
        0..,
        gal_var_assign_obj,
        alt((symbol_comma, symbol_semicolon)),
    )
    .parse_next(input)?;
    opt(alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    gal_sentence_end.parse_next(input)?;
    Ok(args)
}

pub fn act_param_define(input: &mut &str) -> Result<Vec<GxlFParam>> {
    gal_sentence_beg.parse_next(input)?;
    let args: Vec<GxlFParam> =
        separated(0.., gal_formal_param, alt((symbol_comma, symbol_semicolon)))
            .parse_next(input)?;
    opt(alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    gal_sentence_end.parse_next(input)?;
    Ok(args)
}

pub fn shell_opt_setting(key: String, value: String, shell_opt: &mut ShellOption) {
    if key == "suc" {
        shell_opt.suc = Some(value);
    } else if key == "ok_codes" {
        shell_opt.ok_codes = parse_ok_codes(value.as_str());
    } else if key == "quiet" {
        if value.to_lowercase() == "true" {
            shell_opt.quiet = true;
        } else if value.to_lowercase() == "true" {
            shell_opt.quiet = false;
        }
    } else if key == "out" {
        if value.to_lowercase() == "true" {
            shell_opt.quiet = false;
        } else if value.to_lowercase() == "true" {
            shell_opt.quiet = true;
        }
    } else if key == "err" {
        shell_opt.err = Some(value);
    } else if key == "sudo" && value.to_lowercase() == "true" {
        shell_opt.sudo = true;
    } else if key == "log" {
        shell_opt.log_lev = Some(parse_log((key.as_str(), value.as_str())));
    } else if key == "silence" && value.to_lowercase() == "true" {
        shell_opt.secrecy = true;
    } else if key == "stream" && value.to_lowercase() == "true" {
        shell_opt.stream = true;
    }
}

pub fn gal_prop(input: &mut &str) -> Result<GxlVar> {
    skip_spaces_block.parse_next(input)?;
    let prop = gal_var_assign_obj.parse_next(input)?;
    alt((symbol_comma, symbol_semicolon)).parse_next(input)?;
    let vars = GxlVar::ext_new(prop.0, "str".into(), prop.1);
    Ok(vars)
}

pub fn run_gxl<T, F>(gal_fn: F, input: &mut &str) -> Result<T>
where
    F: Fn(&mut &str) -> Result<T>,
{
    match gal_fn(input) {
        Ok(v) => Ok(v),
        Err(e) => {
            println!("{e}");
            println!("input@:> _{input}");
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::friendly::New2;

    use super::*;

    #[test]
    fn vars_test() -> Result<()> {
        let mut data = r#"
             gx.vars {
             x = "${PRJ_ROOT}/test/main.py" ;
             y = "${PRJ_ROOT}/test/main.py" ;
             } ;"#;
        let var = gal_vars(&mut data)?;
        let mut expect = GxlProps::default();
        expect.append(GxlVar::new("x", "${PRJ_ROOT}/test/main.py"));
        expect.append(GxlVar::new("y", "${PRJ_ROOT}/test/main.py"));
        assert_eq!(var, expect);
        assert_eq!(data, "");
        Ok(())
    }
}
