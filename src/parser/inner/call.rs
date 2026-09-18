use super::super::prelude::*;
use orion_parse::symbol::symbol_comma;
use winnow::combinator::separated;

use crate::ability::delegate::ActCall;
use crate::parser::abilities::param::gal_actual_param;
use crate::parser::domain::{gal_call_beg, gal_call_end, gal_var_input};
use crate::primitive::GxlAParam;
pub fn gal_call(input: &mut &str) -> Result<ActCall> {
    let name = take_var_path
        .context(wn_desc("<call-name>"))
        .parse_next(input)?;
    let args = fun_call_args.parse_next(input)?;
    let dto = ActCall::from((name, args));
    Ok(dto)
}

pub fn fun_call_args(input: &mut &str) -> Result<Vec<GxlAParam>> {
    gal_call_beg.parse_next(input)?;
    //let props: Vec<GxlAParam> = separated(0.., fun_arg, symbol_comma).parse_next(input)?;
    let props: Vec<GxlAParam> = separated(0.., gal_actual_param, symbol_comma).parse_next(input)?;
    opt(symbol_comma).parse_next(input)?;
    gal_call_end.parse_next(input)?;
    Ok(props)
}

pub fn action_call_args(input: &mut &str) -> Result<Vec<(String, String)>> {
    gal_call_beg.parse_next(input)?;
    let props: Vec<(String, String)> =
        separated(0.., gal_var_input, symbol_comma).parse_next(input)?;
    opt(symbol_comma).parse_next(input)?;
    gal_call_end.parse_next(input)?;
    Ok(props)
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;

    #[test]
    fn call_test() {
        let mut data = r#"
             os.path ( dist: "./tests/" , keep: "true" ) ;"#;
        let found = run_gxl(gal_call, &mut data).assert();
        let expect = ActCall::from((
            "os.path".to_string(),
            vec![
                GxlAParam::from_val("dist", "./tests/"),
                GxlAParam::from_val("keep", "true"),
            ],
        ));
        assert_eq!(found.actual_params(), expect.actual_params());
        assert_eq!(data, "");

        let mut data = r#"
             path ( dist: "./tests/" , keep: "true" ) ;"#;
        let found = run_gxl(gal_call, &mut data).assert();
        let expect = ActCall::from((
            "path".to_string(),
            vec![
                GxlAParam::from_val("dist", "./tests/"),
                GxlAParam::from_val("keep", "true"),
            ],
        ));
        assert_eq!(found.actual_params(), expect.actual_params());
        assert_eq!(data, "");
    }
}
