use crate::components::gxl_flow::meta::FlowMeta;
use crate::parser::prelude::*;

use crate::components::GxlFlow;
use crate::parser::stc_ann::gal_ann;
use crate::parser::stc_blk::gal_block;

use super::head::galaxy_flow_head;

pub fn gal_stc_flow_body(input: &mut &str) -> Result<GxlFlow> {
    let head = galaxy_flow_head
        .context(wn_desc("<flow-head>"))
        .parse_next(input)?;
    let mut meta = FlowMeta::build_flow(head.first);
    meta.set_preorder(head.before);
    meta.set_postorder(head.after);
    let mut obj = GxlFlow::from(meta);
    multispace0.parse_next(input)?;
    if !starts_with(";", input) {
        let block = gal_block.parse_next(input)?;
        obj.append(block);
    }
    let _ = opt(symbol_semicolon).parse_next(input)?;
    Ok(obj)
}
pub fn gal_stc_flow(input: &mut &str) -> Result<GxlFlow> {
    skip_spaces_block(input)?;
    let ann = opt(gal_ann).parse_next(input)?;
    let mut flow = gal_stc_flow_body.parse_next(input)?;
    flow.set_anns(ann);
    Ok(flow)
}

#[cfg(test)]
mod tests {
    use crate::friendly::New3;
    use orion_error::dev::testing::TestAssert;

    use crate::{
        components::gxl_flow::anno::{FlowAnnFunc, FlowAnnotation},
        parser::{inner::run_gxl, stc_flow::body::gal_stc_flow},
    };

    #[test]
    fn flow_test0() {
        let mut data = r#"
    flow start {
         A = "this is A";
         B = ${A} ;
         gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
    };"#;
        let flow = run_gxl(gal_stc_flow, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }
    #[test]
    fn flow_test1() {
        let mut data = r#"
    flow start {
        gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
        gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
    };"#;
        let flow = run_gxl(gal_stc_flow, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }
    #[test]
    fn flow_test2() {
        let mut data = r#"
        flow start {
             key = "value";
             gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
        };"#;
        let flow = run_gxl(gal_stc_flow, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }

    #[test]
    fn flow_test3() {
        let mut data = r#"
    #[usage(desp="test",color="red"),auto_load(entry)]
    flow start {
         gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
    };"#;
        let flow = run_gxl(gal_stc_flow, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
        assert_eq!(flow.meta().annotations().len(), 2);
        assert_eq!(flow.meta().desp(), Some("test".to_string()));
        assert_eq!(flow.meta().color(), Some("red".to_string()));
    }

    #[test]
    fn flow_test4() {
        let mut data = r#"
    #[use(test)]
    flow start {
         if ${VAL} == "1" {
            gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
         }
         else {
            gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
         }
    };"#;
        let flow = run_gxl(gal_stc_flow, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }

    #[test]
    fn flow_test5() {
        let mut data = r#"
    flow start : one ;"#;
        let flow = run_gxl(gal_stc_flow, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }

    #[test]
    fn flow_ann_test() {
        let mut data = r#"
    #[auto_load(entry)]
    flow test: test_a,test_b : test_c, test_d {
         gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
    };"#;
        let flow = run_gxl(gal_stc_flow, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "test");
        assert_eq!(
            *flow.meta().annotations(),
            vec![FlowAnnotation::new(
                FlowAnnFunc::AutoLoad,
                "auto_load",
                vec![("_1", "entry")]
            ),]
        );
    }
    #[test]
    fn flow_test6() {
        let mut data = r#"
        flow x {
            conf.tpl (
              tpl : "${MAIN_CONF}/tpls/test.sh"  ,
              dst : "${MAIN_CONF}/options/test.sh" ,
              data : "hello" ,
            );
            os.copy (
                src  : "${MAIN_CONF}/options/nginx.conf",
                dst  : "${MAIN_CONF}/used/nginx_ex.conf",
            );
        }
            "#;

        let flow = run_gxl(gal_stc_flow, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "x");
    }
}
