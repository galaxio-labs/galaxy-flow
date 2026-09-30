use super::prelude::*;

use crate::ability::patch::{GxPatchFile, GxPatchFileBuilder, PatchAction};
use crate::parser::domain::gal_keyword_alt;

fn parse_patch_action(value: &str) -> Option<PatchAction> {
    match value.to_lowercase().as_str() {
        "set" => Some(PatchAction::Set),
        "comment_line" => Some(PatchAction::CommentLine),
        "uncomment_line" => Some(PatchAction::UncommentLine),
        "comment_block" => Some(PatchAction::CommentBlock),
        "uncomment_block" => Some(PatchAction::UncommentBlock),
        _ => None,
    }
}

fn parse_bool_text(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn gal_patch_file(input: &mut &str) -> Result<GxPatchFile> {
    let mut builder = GxPatchFileBuilder::default();
    builder.strict(true);
    builder.dry_run(false);
    builder.backup(false);
    builder.comment_prefix("#".to_string());

    gal_keyword_alt("gx.patch_file", "rg.patch_file", input)?;
    let props = action_call_args.parse_next(input)?;

    for (k, v) in props {
        let key = k.to_lowercase();
        match key.as_str() {
            "default" | "file" => {
                builder.file(v);
            }
            "action" => {
                if let Some(action) = parse_patch_action(v.as_str()) {
                    builder.action(action);
                } else {
                    return fail
                        .context(wn_desc("gx.patch_file(action)"))
                        .parse_next(input);
                }
            }
            "marker" | "id" => {
                builder.marker(v);
            }
            "value" => {
                builder.value(Some(v));
            }
            "strict" => {
                if let Some(strict) = parse_bool_text(v.as_str()) {
                    builder.strict(strict);
                } else {
                    return fail
                        .context(wn_desc("gx.patch_file(strict)"))
                        .parse_next(input);
                }
            }
            "dry_run" => {
                if let Some(dry_run) = parse_bool_text(v.as_str()) {
                    builder.dry_run(dry_run);
                } else {
                    return fail
                        .context(wn_desc("gx.patch_file(dry_run)"))
                        .parse_next(input);
                }
            }
            "backup" => {
                if let Some(backup) = parse_bool_text(v.as_str()) {
                    builder.backup(backup);
                } else {
                    return fail
                        .context(wn_desc("gx.patch_file(backup)"))
                        .parse_next(input);
                }
            }
            "comment_prefix" | "comment" => {
                if v.is_empty() {
                    return fail
                        .context(wn_desc("gx.patch_file(comment_prefix)"))
                        .parse_next(input);
                }
                builder.comment_prefix(v);
            }
            _ => {
                return fail
                    .context(wn_desc("gx.patch_file(unknown arg)"))
                    .parse_next(input);
            }
        }
    }

    match builder.build() {
        Ok(o) => Ok(o),
        Err(e) => {
            error!(target: "parse", "{e}");
            fail.context(wn_desc("gx.patch_file(build)"))
                .parse_next(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use orion_error::dev::testing::TestAssert;

    use super::*;

    #[test]
    fn parse_patch_set() {
        let mut data = r#"
             gx.patch_file (
             file : "./Cargo.toml",
             action : "set",
             marker : "version",
             value : "v2.0",
             ) ;"#;
        let obj = gal_patch_file(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.file(), "./Cargo.toml");
        assert_eq!(obj.marker(), "version");
        assert_eq!(obj.action(), &PatchAction::Set);
        assert_eq!(obj.value(), &Some("v2.0".to_string()));
    }

    #[test]
    fn parse_patch_comment_block() {
        let mut data = r#"
             gx.patch_file (
             file : "./Cargo.toml",
             action : "comment_block",
             marker : "res_depend_test",
             strict : "false",
             backup : "true",
             ) ;"#;
        let obj = gal_patch_file(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.action(), &PatchAction::CommentBlock);
        assert!(!obj.strict());
        assert!(*obj.backup());
    }

    #[test]
    fn parse_patch_rejects_empty_comment_prefix() {
        let mut data = r#"
             gx.patch_file (
             file : "./Cargo.toml",
             action : "comment_line",
             marker : "m1",
             comment_prefix : "",
             ) ;"#;
        assert!(gal_patch_file(&mut data).is_err());
    }

    #[test]
    fn parse_patch_comment_alias() {
        let mut data = r#"
             gx.patch_file (
             file : "./Cargo.toml",
             action : "comment_line",
             marker : "m1",
             comment : "//",
             ) ;"#;
        let obj = gal_patch_file(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.comment_prefix(), "//");
    }
}
