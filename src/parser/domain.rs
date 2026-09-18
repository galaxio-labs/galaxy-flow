use crate::primitive::{GxlAParam, GxlObject};

use super::prelude::*;
use orion_parse::{
    atom::{skip_spaces_block, take_var_name},
    define::{gal_raw_str, take_string},
    symbol::{
        symbol_assign, symbol_brace_beg, symbol_brace_end, symbol_bracket_beg, symbol_bracket_end,
        symbol_colon, symbol_semicolon, wn_desc,
    },
};
use winnow::combinator::separated;

use super::atom::take_var_ref_name;

pub fn parse_log(pair: (&str, &str)) -> log::Level {
    if pair.0 == "log"
        && let Ok(level_int) = pair.1.parse::<u32>()
    {
        match level_int {
            1 => log::Level::Info,
            2 => log::Level::Debug,
            3 => log::Level::Trace,
            _ => log::Level::Info,
        }
    } else {
        log::Level::Info
    }
}

pub fn ext_meta_names(input: &mut &str) -> Result<String> {
    let fst = take_var_name(input)?;
    Ok(fst)
}

//take :  key, or ${key}
pub fn gal_mix_item(input: &mut &str) -> Result<String> {
    alt((
        take_var_name,
        take_var_ref_name.map(|x| format!("${{{x}}}")),
    ))
    .parse_next(input)
}
pub fn gal_mix_in(input: &mut &str) -> Result<Vec<String>> {
    (multispace0, symbol_colon).parse_next(input)?;
    let found: Vec<String> = separated(0.., gal_mix_item, ",").parse_next(input)?;
    Ok(found)
}

pub fn take_version(input: &mut &str) -> Result<(i32, i32, i32, Option<i32>)> {
    let (a, _, b, _, c) = (digit1, ".", digit1, ".", digit1).parse_next(input)?;
    let build = opt((".", digit1)).parse_next(input)?;
    let a = a.parse::<i32>().unwrap();
    let b = b.parse::<i32>().unwrap();
    let c = c.parse::<i32>().unwrap();
    let d = build.map(|x| x.1.parse::<i32>().unwrap());
    Ok((a, b, c, d))
}

pub fn gal_var_input(input: &mut &str) -> Result<(String, String)> {
    let _ = multispace0.parse_next(input)?;
    let key_opt = opt(
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
            .context(wn_desc("<var-name>")),
    )
    .parse_next(input)?;
    if key_opt.is_some() {
        symbol_colon.parse_next(input)?;
    }
    let _ = multispace0.parse_next(input)?;
    let val = alt((take_string, gal_raw_str))
        .context(wn_desc("<var-val>"))
        .parse_next(input)?;
    multispace0(input)?;
    let key = key_opt.unwrap_or("default");
    //(multispace0, alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    Ok((key.to_string(), val.to_string()))
}

pub fn gal_assign_exp(input: &mut &str) -> Result<(String, String)> {
    let _ = multispace0.parse_next(input)?;
    let key_opt = opt(
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
            .context(wn_desc("<var-name>")),
    )
    .parse_next(input)?;
    if key_opt.is_some() {
        symbol_assign.parse_next(input)?;
    }
    let _ = multispace0.parse_next(input)?;
    let val = alt((take_string, gal_raw_str))
        .context(wn_desc("<var-val>"))
        .parse_next(input)?;
    multispace0(input)?;
    let key = key_opt.unwrap_or("default");
    Ok((key.to_string(), val.to_string()))
}

pub fn fun_arg(input: &mut &str) -> Result<GxlAParam> {
    let _ = multispace0.parse_next(input)?;
    let key_opt = opt(
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
            .context(wn_desc("<var-name>")),
    )
    .parse_next(input)?;
    if key_opt.is_some() {
        symbol_colon.parse_next(input)?;
    }
    let _ = multispace0.parse_next(input)?;
    let val = alt((
        take_var_ref_name.map(GxlObject::VarRef),
        take_string.map(GxlObject::from_val),
        //gal_raw_string,
    ))
    .context(wn_desc("<var-val>"))
    .parse_next(input)?;
    multispace0(input)?;
    Ok(GxlAParam::new(key_opt.unwrap_or("default"), val))
}

pub fn gal_call_beg(input: &mut &str) -> Result<()> {
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    symbol_bracket_beg
        .context(wn_desc("<call-beg>"))
        .parse_next(input)?;
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    Ok(())
}
pub fn gal_call_end(input: &mut &str) -> Result<()> {
    skip_spaces_block.parse_next(input)?;
    symbol_bracket_end
        .context(wn_desc("<call-end>"))
        .parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    opt(symbol_semicolon).parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    Ok(())
}

pub fn gal_sentence_beg(input: &mut &str) -> Result<()> {
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    symbol_brace_beg.parse_next(input)?;
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    Ok(())
}
pub fn gal_sentence_end(input: &mut &str) -> Result<()> {
    skip_spaces_block.parse_next(input)?;
    symbol_brace_end.parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    opt(symbol_semicolon).parse_next(input)?;
    Ok(())
}

pub fn gal_block_beg(input: &mut &str) -> Result<()> {
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    symbol_brace_beg.parse_next(input)?;
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    Ok(())
}

pub fn gal_block_end(input: &mut &str) -> Result<()> {
    skip_spaces_block.parse_next(input)?;
    symbol_brace_end.parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    Ok(())
}

pub fn gal_keyword(keyword: &'static str, input: &mut &str) -> Result<()> {
    (multispace0, keyword)
        .context(wn_desc(keyword))
        .parse_next(input)?;
    Ok(())
}

pub fn gal_keyword_alt(
    keyword: &'static str,
    keyword2: &'static str,
    input: &mut &str,
) -> Result<()> {
    (multispace0, alt((keyword, keyword2)), multispace0)
        .context(wn_desc(keyword))
        .parse_next(input)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::parser::{abilities::define::gal_var_assign_obj, inner::run_gxl};

    use super::*;

    #[test]
    fn test_mix() {
        let mut data = r#": dev,base"#;
        let vals = gal_mix_in(&mut data).assert();
        assert_eq!(vals, vec!["dev", "base"]);

        let mut data = r#": dev,${base}"#;
        let vals = gal_mix_in(&mut data).assert();
        assert_eq!(vals, vec!["dev", "${base}"]);
    }

    #[test]
    fn test_ver() {
        let mut data = "1.0.0.123";
        let (a, b, c, d) = take_version(&mut data).assert();
        assert_eq!(a, 1);
        assert_eq!(b, 0);
        assert_eq!(c, 0);
        assert_eq!(d, Some(123));
    }
    #[test]
    fn test_assign() {
        let mut data = "data= r#\"{\"branchs\" : [{ \"name\": \"develop\" }, { \"name\" : \"release/1\"}]}\"#;";
        let (key, val) = run_gxl(gal_var_assign_obj, &mut data).assert();
        assert_eq!(key, "data".to_string());
        assert_eq!(
            val,
            GxlObject::from_val(
                r#"{"branchs" : [{ "name": "develop" }, { "name" : "release/1"}]}"#.to_string()
            )
        );
    }
}
