use super::prelude::*;

use crate::ability::{GxDownLoad, GxDownLoadBuilder};
use crate::ability::{GxUpLoad, GxUpLoadBuilder};
use crate::parser::domain::gal_keyword;
pub fn gal_download(input: &mut &str) -> Result<GxDownLoad> {
    let mut down = GxDownLoadBuilder::default();
    gal_keyword("gx.download", input)?;
    let props = action_call_args.parse_next(input)?;
    for (k, v) in &props {
        if k == "url" {
            down.remote_url(v.clone());
        } else if k == "local_file" {
            down.local_file(v.clone());
        } else if k == "username" {
            down.username(v.clone());
        } else if k == "password" {
            down.password(v.clone());
        } else if k == "force" {
            down.force(parse_flag(v));
        }
    }
    match down.build() {
        Ok(o) => Ok(o),
        Err(e) => {
            error!("{e}",);
            fail.context(wn_desc("gx.download")).parse_next(input)
        }
    }
}

/// GXL 调用里的布尔字面量（如 `force: "true"` / `force: true`）。
fn parse_flag(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "on" | "1"
    )
}

pub fn gal_upload(input: &mut &str) -> Result<GxUpLoad> {
    let mut down = GxUpLoadBuilder::default();
    gal_keyword("gx.upload", input)?;
    let props = action_call_args.parse_next(input)?;
    for (k, v) in &props {
        if k == "url" {
            down.svc_url(v.clone());
        } else if k == "local_file" {
            down.local_file(v.clone());
        } else if k == "method" {
            down.method(v.clone());
        } else if k == "username" {
            down.username(v.clone());
        } else if k == "password" {
            down.password(v.clone());
        }
    }
    match down.build() {
        Ok(o) => Ok(o),
        Err(e) => {
            error!("{e}",);
            fail.context(wn_desc("gx.upload")).parse_next(input)
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::infra::once_init_log;

    use super::*;
    use orion_error::dev::testing::TestAssert;

    #[test]
    fn parse_gx_download() {
        once_init_log();
        let mut data = r#"
             gx.download (
             url : "https://github/galaxy",
             local_file : "gsys" ,
             ) ;"#;
        let obj = gal_download(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.remote_url(), "https://github/galaxy");
        // 未写 force：默认 false（沿用 reuse_cache）
        assert!(!obj.force());
    }

    #[test]
    fn parse_gx_download_with_force() {
        once_init_log();
        let mut data = r#"
             gx.download (
             url : "https://github/galaxy",
             local_file : "gsys",
             force : "true",
             ) ;"#;
        let obj = gal_download(&mut data).assert();
        assert_eq!(data, "");
        assert!(obj.force());
    }
    #[test]
    fn parse_gx_upload() {
        once_init_log();
        let mut data = r#"
             gx.upload (
             url : "https://github/galaxy",
             method : "put",
             local_file : "gsys" ,
             ) ;"#;
        let obj = gal_upload(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.svc_url(), "https://github/galaxy");
    }
}
