use super::super::prelude::*;
use super::call::action_call_args;

use winnow::combinator::fail;

use crate::ability::echo::*;
use crate::ability::version::*;
use crate::parser::domain::gal_keyword_alt;

pub fn gal_echo(input: &mut &str) -> Result<GxEcho> {
    let mut watcher = GxEcho::default();
    gal_keyword_alt("gx.echo", "rg.echo", input)?;
    let props = action_call_args.parse_next(input)?;
    for (k, v) in props {
        if k == "value" || k == "default" {
            watcher.set(v.as_str());
        }
    }
    Ok(watcher)
}

pub fn gal_version(input: &mut &str) -> Result<GxlVersion> {
    let mut builder = GxlVersionBuilder::default();
    builder.verinc(VerInc::Build);
    builder.export("VERSION".into());
    gal_keyword_alt("gx.ver", "rg.ver", input)?;
    let props = action_call_args.parse_next(input)?;
    for (key, val) in props {
        if key == "file" {
            builder.file(val);
            continue;
        }
        if key == "export" {
            builder.file(val);
            continue;
        }
        if key == "inc" {
            debug!("version inc :{val}",);
            if val == "build" {
                builder.verinc(VerInc::Build);
            }
            if val == "bugfix" {
                builder.verinc(VerInc::Bugfix);
            }
            if val == "feature" {
                builder.verinc(VerInc::Feature);
            }
            if val == "main" {
                builder.verinc(VerInc::Main);
            }
            if val == "null" {
                builder.verinc(VerInc::Null);
            }
        }
    }
    if let Ok(ver) = builder.build() {
        Ok(ver)
    } else {
        fail.parse_next(input)
    }
}

pub fn gal_sn(input: &mut &str) -> Result<GxlSn> {
    let mut builder = GxlSnBuilder::default();
    builder.export("SN".into());
    builder.action(SnAction::Read);
    gal_keyword_alt("gx.sn", "rg.sn", input)?;
    let props = action_call_args.parse_next(input)?;
    for (key, val) in props {
        if key == "file" {
            builder.file(val);
            continue;
        }
        if key == "export" {
            builder.export(val);
            continue;
        }
        if key == "action" {
            if let Ok(action) = parse_sn_action(val.as_str()) {
                builder.action(action);
            } else {
                return fail.parse_next(input);
            }
            continue;
        }
    }
    if let Ok(sn) = builder.build() {
        Ok(sn)
    } else {
        fail.parse_next(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::inner::common::run_gxl;

    use super::*;

    #[test]
    fn echo_test_args() -> Result<()> {
        let mut data = r#"
             gx.echo ( value : "${PRJ_ROOT}/test/main.py" ) ;"#;
        let found = run_gxl(gal_echo, &mut data)?;
        let mut expect = GxEcho::default();
        expect.set(r#"${PRJ_ROOT}/test/main.py"#);
        assert_eq!(found, expect);
        assert_eq!(data, "");
        Ok(())
    }
    #[test]
    fn echo_test_default() -> Result<()> {
        let mut data = r#"
             gx.echo ( "${PRJ_ROOT}/test/main.py" ) ;"#;
        let found = run_gxl(gal_echo, &mut data)?;
        let mut expect = GxEcho::default();
        expect.set(r#"${PRJ_ROOT}/test/main.py"#);
        assert_eq!(found, expect);
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn ver_test() {
        let mut data = r#"
             gx.ver  ( file : "./tests/version.txt",  inc : "build"  ) ;"#;
        let found = gal_version(&mut data).unwrap();
        let expect = GxlVersion::new("./tests/version.txt".into());
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }

    #[test]
    fn sn_test() {
        let mut data = r#"
             gx.sn  ( file : "./tests/sn.txt" ) ;"#;
        let found = gal_sn(&mut data).unwrap();
        let expect = GxlSn::new("./tests/sn.txt".into());
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }

    #[test]
    fn sn_reset_test() {
        let mut data = r#"
             gx.sn  ( file : "./tests/sn.txt", export : "BUILD_SN", action : "reset"  ) ;"#;
        let found = gal_sn(&mut data).unwrap();
        let expect = GxlSnBuilder::default()
            .file("./tests/sn.txt".into())
            .export("BUILD_SN".into())
            .action(SnAction::Reset)
            .build()
            .unwrap();
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }

    #[test]
    fn sn_add_test() {
        let mut data = r#"
             gx.sn  ( file : "./tests/sn.txt", export : "BUILD_SN", action : "add"  ) ;"#;
        let found = gal_sn(&mut data).unwrap();
        let expect = GxlSnBuilder::default()
            .file("./tests/sn.txt".into())
            .export("BUILD_SN".into())
            .action(SnAction::Add)
            .build()
            .unwrap();
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }
}
