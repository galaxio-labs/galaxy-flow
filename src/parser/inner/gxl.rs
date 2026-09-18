use super::prelude::*;

use crate::ability::{GxRun, gxl::GxRunBuilder};

pub fn gal_run(input: &mut &str) -> Result<GxRun> {
    let mut builder = GxRunBuilder::default();
    gal_keyword("gx.run", input)?;
    let props = action_call_args.parse_next(input)?;
    builder.gxl_path("./_gal/work.gxl".into());
    builder.env_isolate(false);
    for one in props {
        let key = one.0.to_lowercase();
        if key == "env" {
            builder.env_conf(one.1);
        } else if key == "flow" {
            let flows = one.1.split(",").map(String::from).collect();
            builder.flow_cmd(flows);
        } else if key == "conf" {
            builder.gxl_path(one.1);
        } else if key == "local" {
            builder.run_path(one.1);
        } else if key == "isolate" && one.1 == "true" {
            builder.env_isolate(true);
        }
    }
    match builder.build() {
        Ok(obj) => Ok(obj),
        Err(e) => {
            error!("{e}");
            fail.context(wn_desc("gx.run-build")).parse_next(input)
        }
    }
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::infra::once_init_log;

    use super::*;

    #[test]
    fn cmd_test() {
        once_init_log();
        let mut data = r#"
             gx.run ( local : "${PRJ_ROOT}", env : "dev" , flow : "conf,test" , isolate : "true" ) ;"#;
        let obj = gal_run(&mut data).assert();
        assert_eq!(obj.env_conf(), "dev");
        assert_eq!(obj.env_conf(), "dev");
    }
}
