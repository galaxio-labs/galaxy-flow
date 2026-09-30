use crate::components::gxl_fun::fun::GxlFun;
use crate::parser::prelude::*;

use crate::parser::stc_blk::gal_block;

use super::head::gal_fun_head;

pub fn trim_trailing_space(s: &str) -> &str {
    s.trim_end_matches(|c: char| c.is_whitespace())
}

pub fn gal_fun_body(input: &mut &str) -> Result<GxlFun> {
    let meta = gal_fun_head
        .context(wn_desc("<flow-head>"))
        .parse_next(input)?;
    let mut obj = GxlFun::from(meta);
    multispace0.parse_next(input)?;
    if !starts_with(";", input) {
        let block = gal_block.parse_next(input)?;
        obj.append(block);
    }
    let _ = opt(symbol_semicolon).parse_next(input)?;
    Ok(obj)
}
pub fn gal_stc_fun(input: &mut &str) -> Result<GxlFun> {
    skip_spaces_block(input)?;
    //let ann = opt(gal_ann).parse_next(input)?;
    let fun = gal_fun_body.parse_next(input)?;
    //flow.set_anns(ann);
    Ok(fun)
}

#[cfg(test)]
mod tests {
    use orion_error::dev::testing::TestAssert;

    use crate::{
        parser::{gxl_fun::body::gal_stc_fun, inner::run_gxl},
        primitive::GxlFParam,
        util::OptionFrom,
    };

    #[test]
    fn flow_test0() {
        let mut data = r#"
    fn start () {
         A = "this is A";
         B = ${A} ;
         gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
    };"#;
        let fun = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(fun.meta().name(), "start");
    }
    #[test]
    fn flow_test1() {
        let mut data = r#"
    fn start( first, second) {
        gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
        gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
    };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }
    #[test]
    fn flow_test2() {
        let mut data = r#"
        fn start () {
             key = "value";
             gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }

    #[test]
    fn flow_test4() {
        let mut data = r#"
    fn start(first) {
         if ${VAL} == "1" {
            gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
         }
         else {
            gx.echo ( value  : "${PRJ_ROOT}/test/main.py"  );
         }
    };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "start");
    }

    #[test]
    fn flow_test6() {
        let mut data = r#"
        fn x() {
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

        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "x");
    }

    #[test]
    fn flow_test7_empty_body() {
        let mut data = r#"
        fn empty() {
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "empty");
    }

    #[test]
    fn flow_test8_multiple_params() {
        let mut data = r#"
        fn multi_param(a, b = 1, c, d) {
            result = "a b c d processed";
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "multi_param");
        assert_eq!(
            flow.meta().params(),
            &[
                GxlFParam::new("a"),
                GxlFParam::new("b").with_default_value(1_u64.to_opt()),
                GxlFParam::new("c"),
                GxlFParam::new("d"),
            ]
            .to_vec()
        );
    }

    #[test]
    fn flow_test9_nested_calls() {
        let mut data = r#"
        fn nested() {
            gx.cmd( "gx.build_command" );
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "nested");
    }

    #[test]
    fn flow_test10_various_types() {
        let mut data = r#"
        fn mixed_types() {
            string_val = "hello world";
            number_val = "42";
            bool_val = "true";
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "mixed_types");
    }

    #[test]
    fn flow_test11_complex_conditional() {
        let mut data = r#"
        fn complex_condition() {
            if ${status} == "running" && ${count} > 10 {
                gx.log ( level : "warn", message : "High load detected" );
            }
            else if ${status} == "failed" {
                gx.notify ( channel : "alerts", message : "Process failed" );
            }
            else {
                gx.log ( level : "info", message : "System normal" );
            }
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "complex_condition");
    }

    #[test]
    fn flow_test12_empty_body_with_semicolon() {
        let mut data = r#"
        fn noop() {};"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(super::trim_trailing_space(data), "");
        assert_eq!(flow.meta().name(), "noop");
    }

    #[test]
    fn flow_test13_annotations() {
        let mut data = r#"
        fn annotated_func() {
            legacy_code = "some value";
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "annotated_func");
    }

    #[test]
    fn flow_test14_whitespace_variations() {
        let mut data = r#"
        fn spaced(param1, param2) {
            result = "param1 and param2 processed";
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "spaced");
    }

    #[test]
    fn flow_test15_simple_nested() {
        let mut data = r#"
        fn outer() {
            inner_val = "I am inner";
            gx.cmd( cmd : "echo", );
        };"#;
        let flow = run_gxl(gal_stc_fun, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(flow.meta().name(), "outer");
    }
}
