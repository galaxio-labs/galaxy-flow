use std::fs;
use std::path::PathBuf;

use orion_error::conversion::{SourceErr, ToStructError};

use crate::ability::prelude::*;

use super::model::{GxPatchFile, PatchAction};
use super::view::PatchView;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PatchApplyResult {
    output: String,
    marker_hits: usize,
    changed_lines: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockScanResult {
    ranges: Vec<(usize, usize)>,
    marker_hits: usize,
}

impl GxPatchFile {
    fn marker_token(action: PatchAction, marker: &str) -> String {
        match action {
            PatchAction::Set => format!("@gxl:set({marker})"),
            PatchAction::CommentLine | PatchAction::UncommentLine => {
                format!("@gxl:line({marker})")
            }
            PatchAction::CommentBlock | PatchAction::UncommentBlock => {
                format!("@gxl:block({marker})")
            }
        }
    }
}

fn apply_patch_text(
    action: PatchAction,
    text: &str,
    marker: &str,
    value: Option<&str>,
    strict: bool,
    comment_prefix: &str,
) -> ExecResult<PatchApplyResult> {
    if matches!(
        action,
        PatchAction::CommentLine
            | PatchAction::UncommentLine
            | PatchAction::CommentBlock
            | PatchAction::UncommentBlock
    ) {
        validate_comment_prefix(comment_prefix)?;
    }

    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let trailing_newline = text.ends_with('\n');
    let mut lines: Vec<String> = text.lines().map(ToOwned::to_owned).collect();

    let (marker_hits, changed_lines) = match action {
        PatchAction::Set => {
            let val = value.ok_or_else(|| {
                ExecReason::Args
                    .to_err()
                    .with_detail("gx.patch_file missing value: action=set requires value")
            })?;
            apply_set(&mut lines, marker, val, strict)?
        }
        PatchAction::CommentLine => apply_line(&mut lines, marker, strict, comment_prefix, true)?,
        PatchAction::UncommentLine => {
            apply_line(&mut lines, marker, strict, comment_prefix, false)?
        }
        PatchAction::CommentBlock => apply_block(&mut lines, marker, strict, comment_prefix, true)?,
        PatchAction::UncommentBlock => {
            apply_block(&mut lines, marker, strict, comment_prefix, false)?
        }
    };

    let mut output = lines.join(newline);
    if trailing_newline {
        output.push_str(newline);
    }

    Ok(PatchApplyResult {
        output,
        marker_hits,
        changed_lines,
    })
}

fn apply_set(
    lines: &mut [String],
    marker: &str,
    value: &str,
    strict: bool,
) -> ExecResult<(usize, usize)> {
    let token = GxPatchFile::marker_token(PatchAction::Set, marker);
    let mut marker_hits = 0usize;
    let mut changed_lines = 0usize;

    for line in lines {
        if !line.contains(&token) {
            continue;
        }
        marker_hits += 1;
        let patched = patch_set_line(line, value, &token)?;
        if patched != *line {
            *line = patched;
            changed_lines += 1;
        }
    }

    if strict {
        validate_marker_hits("set", marker, marker_hits)?;
    }

    Ok((marker_hits, changed_lines))
}

fn apply_line(
    lines: &mut [String],
    marker: &str,
    strict: bool,
    comment_prefix: &str,
    should_comment: bool,
) -> ExecResult<(usize, usize)> {
    let token = GxPatchFile::marker_token(PatchAction::CommentLine, marker);
    let mut marker_hits = 0usize;
    let mut changed_lines = 0usize;

    for line in lines {
        if !line.contains(&token) {
            continue;
        }
        marker_hits += 1;
        let (next, changed) = if should_comment {
            comment_line(line, comment_prefix)
        } else {
            uncomment_line(line, comment_prefix)
        };
        if changed {
            *line = next;
            changed_lines += 1;
        }
    }

    if strict {
        validate_marker_hits("line", marker, marker_hits)?;
    }

    Ok((marker_hits, changed_lines))
}

fn apply_block(
    lines: &mut [String],
    marker: &str,
    strict: bool,
    comment_prefix: &str,
    should_comment: bool,
) -> ExecResult<(usize, usize)> {
    let start_token = GxPatchFile::marker_token(PatchAction::CommentBlock, marker);
    let end_token = format!("@gxl:end({marker})");
    let scan = scan_block_ranges(lines, &start_token, &end_token, marker, strict)?;

    let mut changed_lines = 0usize;
    for (start, end) in scan.ranges {
        if end <= start + 1 {
            continue;
        }
        for line in lines.iter_mut().take(end).skip(start + 1) {
            let (next, changed) = if should_comment {
                comment_line(line, comment_prefix)
            } else {
                uncomment_line(line, comment_prefix)
            };
            if changed {
                *line = next;
                changed_lines += 1;
            }
        }
    }

    Ok((scan.marker_hits, changed_lines))
}

fn scan_block_ranges(
    lines: &[String],
    start_token: &str,
    end_token: &str,
    marker: &str,
    strict: bool,
) -> ExecResult<BlockScanResult> {
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut open_start: Option<usize> = None;

    for (idx, line) in lines.iter().enumerate() {
        if line.contains(start_token) {
            if strict && open_start.is_some() {
                return Err(ExecReason::Args.to_err().with_detail(format!(
                    "invalid block marker nesting: nested @gxl:block({marker})"
                )));
            }
            if open_start.is_none() {
                open_start = Some(idx);
            }
        }

        if line.contains(end_token) {
            if let Some(start) = open_start {
                if start >= idx {
                    return Err(ExecReason::Args.to_err().with_detail(format!(
                        "invalid block marker order: @gxl:end({marker}) before @gxl:block({marker})"
                    )));
                }
                ranges.push((start, idx));
                open_start = None;
            } else if strict {
                return Err(ExecReason::Args
                    .to_err()
                    .with_detail(format!(
                        "unmatched block end marker: line={} has @gxl:end({marker}) without open @gxl:block({marker})",
                        idx + 1
                    )));
            }
        }
    }

    if strict {
        if open_start.is_some() {
            return Err(ExecReason::Args
                .to_err()
                .with_detail(format!("unclosed block marker: missing @gxl:end({marker})")));
        }
        if ranges.len() != 1 {
            return Err(ExecReason::Args.to_err().with_detail(format!(
                "strict mode requires exactly one block: marker={marker}, blocks={}",
                ranges.len()
            )));
        }
    }

    Ok(BlockScanResult {
        marker_hits: ranges.len(),
        ranges,
    })
}

fn validate_marker_hits(scope: &str, marker: &str, marker_hits: usize) -> ExecResult<()> {
    if marker_hits == 1 {
        return Ok(());
    }
    Err(ExecReason::Args.to_err().with_detail(format!(
        "strict mode marker hit mismatch: scope={scope}, marker={marker}, hits={marker_hits}"
    )))
}

fn validate_comment_prefix(comment_prefix: &str) -> ExecResult<()> {
    if !comment_prefix.is_empty() {
        return Ok(());
    }
    Err(ExecReason::Args
        .to_err()
        .with_detail("invalid comment_prefix: comment_prefix cannot be empty"))
}

fn is_colon_assignment_delimiter(head: &str, idx: usize) -> bool {
    let left = head[..idx].trim();
    if left.is_empty() {
        return false;
    }
    let right = &head[idx + 1..];
    if right.starts_with("//") {
        // Avoid treating URI schemes like "http://..." as key/value delimiters.
        return false;
    }
    true
}

fn find_assignment_split_pos(head: &str) -> Option<usize> {
    let mut first_colon = None;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for (idx, ch) in head.char_indices() {
        if in_double {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_double = false,
                _ => {}
            }
            continue;
        }
        if in_single {
            if ch == '\'' {
                in_single = false;
            }
            continue;
        }

        match ch {
            '"' => in_double = true,
            '\'' => in_single = true,
            '=' => return Some(idx),
            ':' if first_colon.is_none() && is_colon_assignment_delimiter(head, idx) => {
                first_colon = Some(idx)
            }
            _ => {}
        }
    }

    first_colon
}

fn patch_set_line(line: &str, value: &str, marker_token: &str) -> ExecResult<String> {
    let marker_idx = line.find(marker_token).ok_or_else(|| {
        ExecReason::Args.to_err().with_detail(format!(
            "missing set marker: marker token {marker_token} not found"
        ))
    })?;

    let mut suffix_start = marker_idx;
    let mut marker_comment_start: Option<usize> = None;

    if let Some(hash_idx) = line[..marker_idx].rfind('#')
        && line[hash_idx + 1..marker_idx].trim().is_empty()
    {
        marker_comment_start = Some(hash_idx);
    }
    if let Some(slash_idx) = line[..marker_idx].rfind("//")
        && line[slash_idx + 2..marker_idx].trim().is_empty()
    {
        marker_comment_start =
            Some(marker_comment_start.map_or(slash_idx, |cur| cur.max(slash_idx)));
    }
    if let Some(idx) = marker_comment_start {
        suffix_start = idx;
    }

    let head = &line[..suffix_start];
    let suffix = &line[suffix_start..];

    let split_pos = find_assignment_split_pos(head).ok_or_else(|| {
        ExecReason::Args.to_err().with_detail(format!(
            "invalid set target line: line has no assignment delimiter: {line}"
        ))
    })?;

    let before = &head[..split_pos + 1];
    let after = &head[split_pos + 1..];

    let leading_len = after
        .find(|c: char| !c.is_whitespace())
        .unwrap_or(after.len());
    let leading_ws = &after[..leading_len];

    let after_trimmed_end = after.trim_end_matches(|c: char| c.is_whitespace());
    let trailing_ws = &after[after_trimmed_end.len()..];

    Ok(format!("{before}{leading_ws}{value}{trailing_ws}{suffix}"))
}

fn comment_line(line: &str, comment_prefix: &str) -> (String, bool) {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return (line.to_string(), false);
    }

    let indent_len = line.len() - trimmed.len();
    if trimmed.starts_with(comment_prefix) {
        return (line.to_string(), false);
    }

    let mut out = String::with_capacity(line.len() + comment_prefix.len());
    out.push_str(&line[..indent_len]);
    out.push_str(comment_prefix);
    out.push_str(&line[indent_len..]);
    (out, true)
}

fn uncomment_line(line: &str, comment_prefix: &str) -> (String, bool) {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    if !trimmed.starts_with(comment_prefix) {
        return (line.to_string(), false);
    }

    let mut rest = &line[indent_len + comment_prefix.len()..];
    if rest.starts_with(' ') {
        rest = &rest[1..];
    }

    let mut out = String::with_capacity(line.len());
    out.push_str(&line[..indent_len]);
    out.push_str(rest);
    (out, true)
}

#[async_trait]
impl AsyncRunnableTrait for GxPatchFile {
    async fn async_exec(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.patch_file");
        let ex = EnvExpress::from_env_mix(vars_dict.global().clone());

        let file = ex.eval(self.file())?;
        let marker = ex.eval(self.marker())?;
        let value = self.value().as_ref().map(|v| ex.eval(v)).transpose()?;

        let file_path = PathBuf::from(file);
        if !file_path.exists() {
            return Err(ExecReason::Miss
                .to_err()
                .with_detail("patch_file target not found")
                .with_context(&file_path));
        }

        let src = fs::read_to_string(&file_path)
            .source_err(UvsReason::resource_error().into(), "source error")
            .with_context(&file_path)?;

        let applied = apply_patch_text(
            *self.action(),
            src.as_str(),
            marker.as_str(),
            value.as_deref(),
            *self.strict(),
            self.comment_prefix(),
        )?;

        if !*self.dry_run() && applied.changed_lines > 0 {
            if *self.backup() {
                let backup_path = PathBuf::from(format!("{}.bak", file_path.display()));
                fs::write(&backup_path, src.as_str())
                    .source_err(UvsReason::resource_error().into(), "source error")
                    .with_context(&backup_path)?;
            }
            fs::write(&file_path, applied.output.as_str())
                .source_err(UvsReason::resource_error().into(), "source error")
                .with_context(&file_path)?;
        }

        let view = PatchView {
            action: *self.action(),
            file: file_path.display().to_string(),
            marker,
            marker_hits: applied.marker_hits,
            changed_lines: applied.changed_lines,
            dry_run: *self.dry_run(),
        };

        let mut action = Action::from("gx.patch_file").with_target(file_path.display().to_string());
        action.stdout = view.render();
        action.finish();

        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
}

impl ComponentMeta for GxPatchFile {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.patch_file")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_value_by_marker() {
        let src = "version = 1.0   # @gxl:set(version)\n";
        let out = apply_patch_text(PatchAction::Set, src, "version", Some("v2.0"), true, "#")
            .expect("set should pass");
        assert_eq!(out.changed_lines, 1);
        assert_eq!(out.output, "version = v2.0   # @gxl:set(version)\n");
    }

    #[test]
    fn set_value_by_marker_hash_no_space() {
        let src = "version = 1.0   #@gxl:set(version)\n";
        let out = apply_patch_text(PatchAction::Set, src, "version", Some("v2.0"), true, "#")
            .expect("set should support #@gxl marker");
        assert_eq!(out.changed_lines, 1);
        assert_eq!(out.output, "version = v2.0   #@gxl:set(version)\n");
    }

    #[test]
    fn set_value_by_marker_slash_no_space() {
        let src = "version = 1.0   //@gxl:set(version)\n";
        let out = apply_patch_text(PatchAction::Set, src, "version", Some("v2.0"), true, "#")
            .expect("set should support //@gxl marker");
        assert_eq!(out.changed_lines, 1);
        assert_eq!(out.output, "version = v2.0   //@gxl:set(version)\n");
    }

    #[test]
    fn comment_block() {
        let src = "# @gxl:block(res_depend_test)\n[features]\nres_depend_test = []\n# @gxl:end(res_depend_test)\n";
        let out = apply_patch_text(
            PatchAction::CommentBlock,
            src,
            "res_depend_test",
            None,
            true,
            "#",
        )
        .expect("comment block should pass");
        assert_eq!(out.changed_lines, 2);
        assert_eq!(
            out.output,
            "# @gxl:block(res_depend_test)\n#[features]\n#res_depend_test = []\n# @gxl:end(res_depend_test)\n"
        );
    }

    #[test]
    fn uncomment_block() {
        let src = "# @gxl:block(res_depend_test)\n#[features]\n#res_depend_test = []\n# @gxl:end(res_depend_test)\n";
        let out = apply_patch_text(
            PatchAction::UncommentBlock,
            src,
            "res_depend_test",
            None,
            true,
            "#",
        )
        .expect("uncomment block should pass");
        assert_eq!(out.changed_lines, 2);
        assert_eq!(
            out.output,
            "# @gxl:block(res_depend_test)\n[features]\nres_depend_test = []\n# @gxl:end(res_depend_test)\n"
        );
    }

    #[test]
    fn strict_rejects_missing_marker() {
        let src = "version = 1.0\n";
        let err = apply_patch_text(PatchAction::Set, src, "version", Some("v2.0"), true, "#")
            .expect_err("strict mode should reject missing marker");
        assert!(matches!(err.reason(), ExecReason::Args));
        assert!(
            err.detail()
                .as_deref()
                .unwrap_or_default()
                .contains("strict mode")
        );
    }

    #[test]
    fn comment_line_by_marker() {
        let src = "res_depend_test = []   # @gxl:line(res_depend_test)\n";
        let out = apply_patch_text(
            PatchAction::CommentLine,
            src,
            "res_depend_test",
            None,
            true,
            "#",
        )
        .expect("comment line should pass");
        assert_eq!(out.changed_lines, 1);
        assert_eq!(
            out.output,
            "#res_depend_test = []   # @gxl:line(res_depend_test)\n"
        );
    }

    #[test]
    fn uncomment_line_by_marker() {
        let src = "#res_depend_test = []   # @gxl:line(res_depend_test)\n";
        let out = apply_patch_text(
            PatchAction::UncommentLine,
            src,
            "res_depend_test",
            None,
            true,
            "#",
        )
        .expect("uncomment line should pass");
        assert_eq!(out.changed_lines, 1);
        assert_eq!(
            out.output,
            "res_depend_test = []   # @gxl:line(res_depend_test)\n"
        );
    }

    #[test]
    fn set_value_with_colon_in_value() {
        let src = "url = \"http://old\"   # @gxl:set(url)\n";
        let out = apply_patch_text(
            PatchAction::Set,
            src,
            "url",
            Some("\"http://new\""),
            true,
            "#",
        )
        .expect("set should keep assignment delimiter");
        assert_eq!(out.changed_lines, 1);
        assert_eq!(out.output, "url = \"http://new\"   # @gxl:set(url)\n");
    }

    #[test]
    fn set_value_with_colon_no_whitespace() {
        let src = "image:app:v1   # @gxl:set(image)\n";
        let out = apply_patch_text(PatchAction::Set, src, "image", Some("app:v2"), true, "#")
            .expect("set should support key:value without spaces");
        assert_eq!(out.changed_lines, 1);
        assert_eq!(out.output, "image:app:v2   # @gxl:set(image)\n");
    }

    #[test]
    fn set_value_with_json_colon() {
        let src = "\"version\":\"1.0\"   # @gxl:set(version)\n";
        let out = apply_patch_text(PatchAction::Set, src, "version", Some("\"2.0\""), true, "#")
            .expect("set should support quoted key with colon delimiter");
        assert_eq!(out.changed_lines, 1);
        assert_eq!(out.output, "\"version\":\"2.0\"   # @gxl:set(version)\n");
    }

    #[test]
    fn set_rejects_non_assignment_colon_line() {
        let src = "http://old   # @gxl:set(url)\n";
        let err = apply_patch_text(PatchAction::Set, src, "url", Some("http://new"), true, "#")
            .expect_err("set should reject non-assignment colon");
        assert!(
            err.detail()
                .as_deref()
                .unwrap_or_default()
                .contains("line has no assignment delimiter")
        );
    }

    #[test]
    fn strict_rejects_unmatched_block_end_marker() {
        let src = "# @gxl:end(res_depend_test)\n# @gxl:block(res_depend_test)\nres_depend_test = []\n# @gxl:end(res_depend_test)\n";
        let err = apply_patch_text(
            PatchAction::CommentBlock,
            src,
            "res_depend_test",
            None,
            true,
            "#",
        )
        .expect_err("strict mode should reject unmatched block end marker");
        assert!(matches!(err.reason(), ExecReason::Args));
        assert!(
            err.detail()
                .as_deref()
                .unwrap_or_default()
                .contains("unmatched block end marker")
        );
    }

    #[test]
    fn strict_rejects_nested_block_markers() {
        let src = "# @gxl:block(res_depend_test)\n# @gxl:block(res_depend_test)\nres_depend_test = []\n# @gxl:end(res_depend_test)\n# @gxl:end(res_depend_test)\n";
        let err = apply_patch_text(
            PatchAction::CommentBlock,
            src,
            "res_depend_test",
            None,
            true,
            "#",
        )
        .expect_err("strict mode should reject nested block marker");
        assert!(matches!(err.reason(), ExecReason::Args));
        assert!(
            err.detail()
                .as_deref()
                .unwrap_or_default()
                .contains("invalid block marker nesting")
        );
    }

    #[test]
    fn reject_empty_comment_prefix() {
        let src = "feature = true   # @gxl:line(flag)\n";
        let err = apply_patch_text(PatchAction::CommentLine, src, "flag", None, true, "")
            .expect_err("line actions should reject empty comment prefix");
        assert_eq!(
            err.detail().as_deref(),
            Some("invalid comment_prefix: comment_prefix cannot be empty")
        );
    }
}
