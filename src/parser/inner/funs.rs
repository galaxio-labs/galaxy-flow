use super::super::prelude::*;
use super::call::fun_call_args;

use crate::primitive::GxlObject;
use crate::{
    calculate::defined::{FnDefined, FnDefinedBuilder},
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

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

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
}
