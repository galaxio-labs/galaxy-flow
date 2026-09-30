use super::{inner::act_param_define, prelude::*};
use winnow::{Parser, Result};

use crate::components::gxl_act::{activity::Activity, meta::ActivityMeta};

use super::stc_base::gal_act_head;

pub fn gal_activity(input: &mut &str) -> Result<Activity> {
    let name = gal_act_head("activity", input)?;

    let params = act_param_define
        .context(wn_desc("<activity-end>"))
        .parse_next(input)?;
    let meta = ActivityMeta::build(name).with_params(params);
    let obj = Activity::new(meta);
    Ok(obj)
}
#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;
    use orion_sec::sec::{SecFrom, SecValueType};

    use crate::{parser::inner::run_gxl, primitive::GxlFParam, util::OptionFrom};

    use super::*;
    #[test]
    fn activity_test() {
        let mut data = r#"
        activity copy {
            src = "" ;
            dst = "" ;
            log = "1";
            executer = "copy_act.sh" ;
        }
  "#;
        let obj = run_gxl(gal_activity, &mut data).assert();
        let meta = ActivityMeta::build("copy").with_params(vec![
            GxlFParam::new("src")
                .with_default_value(SecValueType::nor_from("".to_string()).to_opt()),
            GxlFParam::new("dst")
                .with_default_value(SecValueType::nor_from("".to_string()).to_opt()),
            GxlFParam::new("log")
                .with_default_value(SecValueType::nor_from("1".to_string()).to_opt()),
            GxlFParam::new("executer")
                .with_default_value(SecValueType::nor_from("copy_act.sh".to_string()).to_opt()),
        ]);
        assert_eq!(data.trim(), "");
        assert_eq!(obj.meta(), &meta);
    }
}
