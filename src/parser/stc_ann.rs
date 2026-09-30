use std::collections::HashMap;

use crate::data::AnnDto;
use crate::data::FunDto;

use super::prelude::*;

use orion_parse::atom::take_var_name;
use winnow::ascii::multispace0;
use winnow::combinator::separated;

use winnow::combinator::delimited;
use winnow::token::take_while;

/// Parse a key-value assignment (e.g., `x = 1`, `x= "A"`, `1`, `"A"`).
pub fn gal_fun_arg_item(input: &mut &str) -> Result<(Option<String>, String)> {
    // Parse the key (optional)
    multispace0.parse_next(input)?;
    let key = opt((
        take_var_name,                   // Parse the key (alphanumeric)
        (multispace0, "=", multispace0), // Skip optional whitespace and equals sign
    ))
    .parse_next(input)?;

    // Parse the value (either a number or a quoted string)
    let value = if input.starts_with('"') {
        // Parse a quoted string
        delimited('"', take_while(0.., |c| c != '"'), '"')
            .context(wn_desc("\"<arg>\""))
            .parse_next(input)?
    } else {
        // Parse a number or unquoted string
        take_while(1.., |c: char| {
            c.is_alphanumeric() || c == '.' || c == '-' || c == '_'
        })
        .context(wn_desc("<arg>"))
        .parse_next(input)?
    };

    // Extract the key if it exists
    let key = key.map(|(k, _)| k.to_string());
    Ok((key, value.to_string()))
}

/// Parse function arguments from a string and return a HashMap.
/// If a key is missing, it is replaced with a placeholder like `_1`, `_2`, etc.
pub fn gal_fun_args_map(input: &mut &str) -> Result<HashMap<String, String>> {
    // Parse the arguments list enclosed in parentheses
    let args: Vec<(Option<String>, String)> = delimited(
        ('(', multispace0), // Start with '('
        separated(
            0..,
            gal_fun_arg_item,                // Parse each key-value pair
            (multispace0, ',', multispace0), // Separated by commas and optional whitespace
        ),
        (multispace0, ')'), // End with ')'
    )
    .context(wn_desc("<fun-args>"))
    .parse_next(input)?;

    // Convert Vec<(Option<String>, String)> to HashMap<String, String>
    let args_map = args
        .into_iter()
        .enumerate()
        .map(|(index, (key, value))| {
            let key = key.unwrap_or_else(|| format!("_{}", index + 1)); // Use _1, _2, etc. for missing keys
            (key, value)
        })
        .collect();

    Ok(args_map)
}

/// Parse a function call into a `FunDto`.
pub fn gal_fun(input: &mut &str) -> Result<FunDto> {
    // Parse the function name
    let keyword = take_var_name.parse_next(input)?;
    //let keyword = ident1.parse_next(input)?;

    //let now: &str = input;
    let args = if starts_with("(", input) {
        // Parse the arguments
        gal_fun_args_map(input)?
    } else {
        HashMap::new()
    };

    Ok(FunDto {
        keyword: keyword.to_string(),
        args,
    })
}

pub fn gal_fun_vec(input: &mut &str) -> Result<Vec<FunDto>> {
    separated(0.., gal_fun, (multispace0, ",", multispace0)).parse_next(input)
}

/// Parse an annotation into an `AnnDto`.
pub fn gal_ann(input: &mut &str) -> Result<AnnDto> {
    // Parse the annotation prefix `#[`
    let _ = (multispace0, "#", "[").parse_next(input)?;

    // Parse the list of function calls
    let funs = gal_fun_vec.parse_next(input)?;

    // Parse the closing `]`
    let _ = "]".parse_next(input)?;

    // Return the parsed annotation
    funs.iter().for_each(|x| debug!(target: "parse","{x}" ));
    Ok(AnnDto { funs })
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::{
        annotation::{FST_ARG_TAG, SEC_ARG_TAG},
        parser::inner::run_gxl,
        str_map,
    };

    use super::*;

    #[test]
    fn test_gal_fun_args_with_args() {
        let mut input = r#"(x="1",y="2")"#;
        let expected = str_map!(
            String::from("x") => String::from("1"),
            String::from("y") => String::from("2"),
        );
        let output = gal_fun_args_map(&mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gal_fun_args_no_args() {
        let mut input = r#"()"#;
        let expected = str_map!();
        let output = gal_fun_args_map(&mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gal_fun_args_invalid_format() {
        let mut input = r#"(x=1,y=2"#; // Missing closing parenthesis
        let result = gal_fun_args_map(&mut input);

        assert!(result.is_err());
    }

    #[test]
    fn test_gal_fun_args_whitespace() {
        let mut input = r#"( x = "1" , y = "2" )"#;
        let expected = str_map!(
            String::from("x") => String::from("1"),
            String::from("y") => String::from("2"),
        );
        let output = run_gxl(gal_fun_args_map, &mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gal_fun_args_simple() {
        let mut input = r#"( "1" , "2" )"#;
        let expected = str_map!(
            String::from(FST_ARG_TAG) => String::from("1"),
            String::from(SEC_ARG_TAG) => String::from("2"),
        );
        let output = run_gxl(gal_fun_args_map, &mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);

        let mut input = r#"( _step1,_step2)"#;
        let expected = str_map!(
            String::from(FST_ARG_TAG) => String::from("_step1"),
            String::from(SEC_ARG_TAG) => String::from("_step2"),
        );
        let output = run_gxl(gal_fun_args_map, &mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_task_ann() {
        let mut input = r#"#[task(name="this@task")]"#;
        let expected = AnnDto {
            funs: [FunDto::new("task", [("name", "this@task")].to_vec())].to_vec(),
        };
        assert_ext_ann(&mut input, expected);

        let mut input = r#"#[task(name="${MOD}@task")]"#;
        let expected = AnnDto {
            funs: [FunDto::new("task", [("name", "${MOD}@task")].to_vec())].to_vec(),
        };
        assert_ext_ann(&mut input, expected);
    }

    #[test]
    fn test_ext_extern_ann() {
        let mut input = r#"#[load(a,b)]"#;
        let expected = AnnDto {
            funs: [FunDto::new(
                "load",
                [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec(),
            )]
            .to_vec(),
        };
        assert_ext_ann(&mut input, expected);

        let mut input = r#"#[undo(step1,step2)]"#;
        let expected = AnnDto {
            funs: [FunDto::new(
                "undo",
                [(FST_ARG_TAG, "step1"), (SEC_ARG_TAG, "step2")].to_vec(),
            )]
            .to_vec(),
        };
        assert_ext_ann(&mut input, expected);

        let mut input = r#"#[undo(_step1,_step2)]"#;
        let expected = AnnDto {
            funs: [FunDto::new(
                "undo",
                [(FST_ARG_TAG, "_step1"), (SEC_ARG_TAG, "_step2")].to_vec(),
            )]
            .to_vec(),
        };
        assert_ext_ann(&mut input, expected);

        let mut input = r#"#[load(a,b),unload()]"#;
        let expected = AnnDto {
            funs: [
                FunDto::new("load", [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec()),
                FunDto::new("unload", [].to_vec()),
            ]
            .to_vec(),
        };
        assert_ext_ann(&mut input, expected);
    }

    #[test]
    fn test_ext_ann_transaction() {
        let mut input = r#"#[transaction,undo(uninstall)]"#;
        let expected = AnnDto {
            funs: [
                FunDto::new("transaction", [].to_vec()),
                FunDto::new("undo", [(FST_ARG_TAG, "uninstall")].to_vec()),
            ]
            .to_vec(),
        };
        assert_ext_ann(&mut input, expected);
    }

    fn assert_ext_ann(input: &mut &str, expected: AnnDto) {
        let output = gal_ann(input).assert();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_ext_ann_dryrun() {
        let mut input = r#"#[dryrun(_dryrun_flow)]"#;
        let expected = AnnDto {
            funs: [FunDto::new(
                "dryrun",
                [(FST_ARG_TAG, "_dryrun_flow")].to_vec(),
            )]
            .to_vec(),
        };
        assert_ext_ann(&mut input, expected);
    }

    #[test]
    fn test_gal_multi_fun() {
        let mut input = r#"load(a,b)"#;
        let expected = [FunDto::new(
            "load",
            [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec(),
        )]
        .to_vec();
        assert_funs(&mut input, expected);
        let mut input = r#"load(a,b),unload(a,b)"#;
        let expected = [
            FunDto::new("load", [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec()),
            FunDto::new("unload", [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec()),
        ]
        .to_vec();
        assert_funs(&mut input, expected);
        let mut input = r#"load(a="1",b="2"),unload(a="3",b)"#;
        let expected = [
            FunDto::new("load", [("a", "1"), ("b", "2")].to_vec()),
            FunDto::new("unload", [("a", "3"), ("_2", "b")].to_vec()),
        ]
        .to_vec();
        assert_funs(&mut input, expected);
        let mut input = r#"load(a="1",b="2"),unload"#;
        let expected = [
            FunDto::new("load", [("a", "1"), ("b", "2")].to_vec()),
            FunDto::new("unload", [].to_vec()),
        ]
        .to_vec();
        assert_funs(&mut input, expected);
    }

    fn assert_funs(input: &mut &str, expected: Vec<FunDto>) {
        let output = gal_fun_vec(input).unwrap();
        assert_eq!(output, expected);
    }

    mod test_fun {
        use crate::parser::stc_ann::gal_fun_arg_item;

        #[test]
        fn test_gal_fun_arg_item_with_key() {
            let mut input = r#"x = 1"#;
            let expected = (Some("x".to_string()), "1".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"x="A""#;
            let expected = (Some("x".to_string()), "A".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_without_key() {
            let mut input = r#"1"#;
            let expected = (None, "1".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#""A""#;
            let expected = (None, "A".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_whitespace() {
            let mut input = r#"  x   =   123"#;
            let expected = (Some("x".to_string()), "123".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"  "A""#;
            let expected = (None, "A".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_invalid_format() {
            let mut input = r#"x="#; // Missing value
            let result = gal_fun_arg_item(&mut input);

            assert!(result.is_err());

            let mut input = r#""#; // Empty input
            let result = gal_fun_arg_item(&mut input);

            assert!(result.is_err());
        }

        #[test]
        fn test_gal_fun_arg_item_unquoted_string() {
            let mut input = r#"x=hello"#;
            let expected = (Some("x".to_string()), "hello".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"hello"#;
            let expected = (None, "hello".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_negative_number() {
            let mut input = r#"x=-123"#;
            let expected = (Some("x".to_string()), "-123".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"-123"#;
            let expected = (None, "-123".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_decimal_number() {
            let mut input = r#"x=3.14"#;
            let expected = (Some("x".to_string()), "3.14".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"3.14"#;
            let expected = (None, "3.14".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }
    }
}
