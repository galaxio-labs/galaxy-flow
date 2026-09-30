use super::super::prelude::*;
use super::call::fun_call_args;

use crate::primitive::GxlObject;
use crate::{
    calculate::defined::{FnDefined, FnDefinedBuilder},
    calculate::exist::FnExists,
    parser::domain::gal_keyword,
};

pub fn gal_defined(input: &mut &str) -> Result<FnDefined> {
    let mut builder = FnDefinedBuilder::default();
    gal_keyword("defined", input)?;
    let props = fun_call_args.parse_next(input)?;
    for one in props {
        let key = one.name().to_lowercase();
        if key == "default" || key == "var" {
            if let GxlObject::VarRef(vref) = one.value() {
                builder.name(vref.clone());
            } else {
                return fail.context(wn_desc("defined(not var)")).parse_next(input);
            }
        }
    }

    match builder.build() {
        Ok(obj) => Ok(obj),
        Err(e) => {
            error!("{e}");
            fail.context(wn_desc("defined")).parse_next(input)
        }
    }
}

/// 解析 `gx.exists(flow: "<name>")`（也接受裸值 `gx.exists("<name>")` 或变量 `${P}`）。
pub fn gal_gx_exists(input: &mut &str) -> Result<FnExists> {
    gal_keyword("gx.exists", input)?;
    let props = fun_call_args.parse_next(input)?;
    let mut flow_obj: Option<GxlObject> = None;
    for one in props {
        let key = one.name().to_lowercase();
        if key == "flow" || key == "name" || key == "default" {
            flow_obj = Some(one.value().clone());
        }
    }
    match flow_obj {
        Some(obj) => Ok(FnExists::new(obj)),
        None => fail
            .context(wn_desc("gx.exists(flow: ...)"))
            .parse_next(input),
    }
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;
    use orion_sec::sec::SecFrom;

    use crate::infra::once_init_log;

    use super::*;

    #[test]
    fn defined_correct() {
        once_init_log();
        let mut data = r#"
             defined(${HOME}) ;"#;
        let obj = gal_defined(&mut data).assert();
        assert_eq!(obj.name(), "HOME");
    }
    #[test]
    fn defined_wrong() {
        once_init_log();
        let mut data = r#"
             defined("HOME") ;"#;
        assert!(gal_defined(&mut data).is_err());
    }

    #[test]
    fn gx_exists_named_and_positional() {
        once_init_log();
        let mut data = r#" gx.exists(flow: "localize") ;"#;
        let obj = gal_gx_exists(&mut data).assert();
        assert_eq!(
            obj.flow(),
            &GxlObject::Value(orion_sec::sec::SecValueType::nor_from(
                "localize".to_string()
            ))
        );

        let mut data = r#" gx.exists("main.install") ;"#;
        let obj = gal_gx_exists(&mut data).assert();
        assert_eq!(
            obj.flow(),
            &GxlObject::Value(orion_sec::sec::SecValueType::nor_from(
                "main.install".to_string()
            ))
        );

        let mut data = r#" gx.exists(${P}) ;"#;
        let obj = gal_gx_exists(&mut data).assert();
        assert_eq!(obj.flow(), &GxlObject::from_ref("P"));
    }

    #[test]
    fn gx_exists_requires_a_flow_argument() {
        once_init_log();
        // 无参数
        let mut data = r#" gx.exists() ;"#;
        assert!(gal_gx_exists(&mut data).is_err());
        // 只有无关 key
        let mut data = r#" gx.exists(noise: "x") ;"#;
        assert!(gal_gx_exists(&mut data).is_err());
    }
}
