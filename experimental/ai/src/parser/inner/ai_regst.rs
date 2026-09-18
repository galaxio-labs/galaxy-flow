use super::super::prelude::*;
use super::call::action_call_args;

use crate::ability::ai::ai_call::GxlAiRegist;
use crate::parser::domain::gal_keyword;
use crate::util::OptionFrom;

pub fn gal_ai_regist(input: &mut &str) -> Result<GxlAiRegist> {
    let mut call = GxlAiRegist::default();
    gal_keyword("gx.ai_regist", input)?;
    let props = action_call_args.parse_next(input)?;
    for one in props {
        let key = one.0.to_lowercase();
        if key == "default" || key == "flow" {
            call.set_flow(one.1);
        } else if key == "role" {
            call.set_role(one.1.to_opt());
        } else if key == "desp" {
            call.set_desp(one.1);
        }
    }
    Ok(call)
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use super::*;

    #[test]
    fn ai_chat_msg() {
        let mut data = r#"
             gx.ai_regist( flow : "start" ,desp: "start serve" ) ;"#;
        let obj = gal_ai_regist(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.flow().as_str(), "start");
        assert_eq!(obj.desp().as_str(), "start serve");
    }
}
