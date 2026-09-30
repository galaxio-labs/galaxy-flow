use super::super::prelude::*;
use super::call::action_call_args;

use std::str::FromStr;

use crate::ability::GxTpl;
use crate::ability::TplDTOBuilder;
use crate::ability::tpl::TPlEngineType;
use crate::parser::domain::gal_keyword_alt;

pub fn gal_tpl(input: &mut &str) -> Result<GxTpl> {
    gal_keyword_alt("gx.tpl", "rg.tpl", input)?;
    let props = action_call_args.parse_next(input)?;
    let mut builder = TplDTOBuilder::default();
    for one in props {
        let key = one.0.to_lowercase();
        let val: String = one.1;
        if key == "tpl" {
            builder.tpl(val);
        } else if key == "dst" {
            builder.dst(val);
        } else if key == "data" {
            builder.data(Some(val));
        } else if key == "engine" {
            if let Ok(engine) = TPlEngineType::from_str(val.as_str()) {
                builder.engine(engine);
            } else {
                error!(target: "parse", "unknow engine :{val}",);
                return fail.context(wn_desc("gx.tpl build")).parse_next(input);
            }
        } else if key == "file" {
            builder.file(Some(val));
        }
    }
    match builder.build() {
        Ok(dto) => Ok(GxTpl::from(dto)),
        Err(e) => {
            error!(target: "parse", "{e}");
            //println!("{}", e);
            fail.context(wn_desc("gx.tpl build")).parse_next(input)
        }
    }
}
#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::{ability::TplDTO, parser::inner::common::run_gxl};

    use super::*;

    #[test]
    fn tpl_sample() {
        let tpl = "${PRJ_ROOT}/conf_tpl.toml";
        let dst = "${PRJ_ROOT}/conf.toml";
        let mut data = r#"
                 gx.tpl (
                 tpl : "${PRJ_ROOT}/conf_tpl.toml" ,
                 dst : "${PRJ_ROOT}/conf.toml" ,
                 ) ;"#;
        let dto = TplDTO {
            tpl: tpl.to_string(),
            dst: dst.to_string(),
            ..Default::default()
        };
        let obj = run_gxl(gal_tpl, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }
    #[test]
    fn tpl_data() {
        let tpl = "${PRJ_ROOT}/conf_tpl.toml";
        let dst = "${PRJ_ROOT}/conf.toml";
        let mut data = "
                 gx.tpl (
                 tpl : \"${PRJ_ROOT}/conf_tpl.toml\" ,
                 dst : \"${PRJ_ROOT}/conf.toml\",
                 data : r#\"{\"branchs\": [\"develop\",\"issue/11\"]} \"#,
                 ) ;";
        let obj = run_gxl(gal_tpl, &mut data).assert();
        let dto = TplDTO {
            tpl: tpl.to_string(),
            dst: dst.to_string(),
            data: Some(String::from("{\"branchs\": [\"develop\",\"issue/11\"]} ")),
            ..Default::default()
        };
        assert_eq!(data, "");
        assert_eq!(&dto, obj.dto());
    }
}
