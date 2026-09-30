use super::prelude::*;

use crate::{
    ability::archive::{GxTar, GxTarBuilder, GxUnTar, GxUnTarBuilder},
    parser::domain::gal_keyword,
};

pub fn gal_tar(input: &mut &str) -> Result<GxTar> {
    let mut builder = GxTarBuilder::default();
    gal_keyword("gx.tar", input)?;
    let props = action_call_args.parse_next(input)?;
    for one in props {
        let key = one.0.to_lowercase();
        if key == "src" {
            builder.src(one.1);
        } else if key == "file" {
            builder.file(one.1);
        }
    }
    match builder.build() {
        Ok(obj) => Ok(obj),
        Err(e) => {
            error!("{e}");
            fail.context(wn_desc("gx.tar-build")).parse_next(input)
        }
    }
}

pub fn gal_untar(input: &mut &str) -> Result<GxUnTar> {
    let mut builder = GxUnTarBuilder::default();
    gal_keyword("gx.untar", input)?;
    let props = action_call_args.parse_next(input)?;
    for one in props {
        let key = one.0.to_lowercase();
        if key == "file" {
            builder.file(one.1);
        } else if key == "dst" {
            builder.dst(one.1);
        }
    }
    match builder.build() {
        Ok(obj) => Ok(obj),
        Err(e) => {
            error!("{e}");
            fail.context(wn_desc("gx.untar-build")).parse_next(input)
        }
    }
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::infra::once_init_log;

    use super::*;

    #[test]
    fn tar_test() {
        once_init_log();
        let mut data = r#"
             gx.tar(  src: "./x" , file: "x.tar.gz" ) ;"#;
        let obj = gal_tar(&mut data).assert();
        assert_eq!(obj.src(), "./x");
        assert_eq!(obj.file(), "x.tar.gz");
    }
    #[test]
    fn untar_test() {
        once_init_log();
        let mut data = r#"
             gx.untar(  dst: "./x" , file: "x.tar.gz" ) ;"#;
        let obj = gal_untar(&mut data).assert();
        assert_eq!(obj.dst(), "./x");
        assert_eq!(obj.file(), "x.tar.gz");
    }
}
