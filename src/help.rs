use crate::err::{RunReason, RunResult};
use colored::Colorize;
use orion_error::conversion::ToStructError;

struct HelpTopic {
    key: &'static str,
    aliases: &'static [&'static str],
    source: &'static str,
    content: &'static str,
}

const HELP_TOPICS: &[HelpTopic] = &[
    HelpTopic {
        key: "guide",
        aliases: &["index", "intro", "start"],
        source: "docs/guidle/index.md",
        content: include_str!("../docs/guidle/index.md"),
    },
    HelpTopic {
        key: "gx",
        aliases: &["cli.gx"],
        source: "docs/guidle/cli/gx.md",
        content: include_str!("../docs/guidle/cli/gx.md"),
    },
    HelpTopic {
        key: "gxl",
        aliases: &["syntax", "gxl.syntax"],
        source: "docs/gxl/syntax.md",
        content: include_str!("../docs/gxl/syntax.md"),
    },
    HelpTopic {
        key: "gx.assert",
        aliases: &["assert"],
        source: "docs/gxl/inner/assert.md",
        content: include_str!("../docs/gxl/inner/assert.md"),
    },
    HelpTopic {
        key: "gx.cmd",
        aliases: &["cmd"],
        source: "docs/gxl/inner/cmd.md",
        content: include_str!("../docs/gxl/inner/cmd.md"),
    },
    HelpTopic {
        key: "gx.echo",
        aliases: &["echo"],
        source: "docs/gxl/inner/echo.md",
        content: include_str!("../docs/gxl/inner/echo.md"),
    },
    HelpTopic {
        key: "gx.patch_file",
        aliases: &["patch_file", "patch"],
        source: "docs/gxl/inner/patch_file.md",
        content: include_str!("../docs/gxl/inner/patch_file.md"),
    },
    HelpTopic {
        key: "gx.read_file",
        aliases: &[
            "read",
            "read_file",
            "gx.read_cmd",
            "read_cmd",
            "gx.read_stdin",
            "read_stdin",
        ],
        source: "docs/gxl/inner/read.md",
        content: include_str!("../docs/gxl/inner/read.md"),
    },
    HelpTopic {
        key: "gx.run",
        aliases: &["run"],
        source: "docs/gxl/inner/run.md",
        content: include_str!("../docs/gxl/inner/run.md"),
    },
    HelpTopic {
        key: "gx.shell",
        aliases: &["shell"],
        source: "docs/gxl/inner/shell.md",
        content: include_str!("../docs/gxl/inner/shell.md"),
    },
    HelpTopic {
        key: "gx.tpl",
        aliases: &["tpl", "template"],
        source: "docs/gxl/inner/tpl.md",
        content: include_str!("../docs/gxl/inner/tpl.md"),
    },
    HelpTopic {
        key: "gx.vars",
        aliases: &["vars"],
        source: "docs/gxl/inner/vars.md",
        content: include_str!("../docs/gxl/inner/vars.md"),
    },
    HelpTopic {
        key: "gx.ver",
        aliases: &["ver", "version"],
        source: "docs/gxl/inner/ver.md",
        content: include_str!("../docs/gxl/inner/ver.md"),
    },
    HelpTopic {
        key: "gx.sn",
        aliases: &["sn", "serial"],
        source: "docs/gxl/inner/sn.md",
        content: include_str!("../docs/gxl/inner/sn.md"),
    },
    HelpTopic {
        key: "gx.tar",
        aliases: &["tar", "gx.untar", "untar"],
        source: "docs/gxl/inner/tar_untar.md",
        content: include_str!("../docs/gxl/inner/tar_untar.md"),
    },
    HelpTopic {
        key: "gx.download",
        aliases: &["download", "gx.upload", "upload"],
        source: "docs/gxl/inner/download_upload.md",
        content: include_str!("../docs/gxl/inner/download_upload.md"),
    },
    HelpTopic {
        key: "defined",
        aliases: &["gx.defined"],
        source: "docs/gxl/inner/defined.md",
        content: include_str!("../docs/gxl/inner/defined.md"),
    },
];

pub fn print(topic: Option<&str>, markdown: bool) -> RunResult<()> {
    match topic {
        Some(topic) => {
            let doc = resolve(topic)?;
            println!("{}", render_topic(doc, markdown));
        }
        None => {
            println!("{}", "Galaxy Flow doc topics".bold());
            println!();
            println!("usage:");
            println!("  gx doc <topic>");
            println!("  gx doc --markdown <topic>");
            println!();
            println!("topics:");
            for line in topic_lines() {
                println!("  {line}");
            }
        }
    }
    Ok(())
}

fn render_topic(doc: &HelpTopic, markdown: bool) -> String {
    let content = doc.content.trim();
    if markdown {
        return content.to_string();
    }

    format!(
        "topic: {}\nsource: {}\n\n{}",
        doc.key.bright_yellow().bold(),
        doc.source.dimmed(),
        render_markdown(content)
    )
}

fn resolve(topic: &str) -> RunResult<&'static HelpTopic> {
    let normalized = normalize(topic);
    HELP_TOPICS
        .iter()
        .find(|doc| {
            normalize(doc.key) == normalized
                || doc
                    .aliases
                    .iter()
                    .any(|alias| normalize(alias) == normalized)
        })
        .ok_or_else(|| {
            RunReason::Args.to_err().with_detail(format!(
                "unknown doc topic `{topic}`. try one of: {}",
                topic_keys().join(", ")
            ))
        })
}

fn topic_keys() -> Vec<&'static str> {
    HELP_TOPICS.iter().map(|topic| topic.key).collect()
}

fn topic_lines() -> Vec<String> {
    HELP_TOPICS
        .iter()
        .map(|topic| {
            if topic.aliases.is_empty() {
                topic.key.to_string()
            } else {
                format!("{} ({})", topic.key, topic.aliases.join(", "))
            }
        })
        .collect()
}

fn normalize(topic: &str) -> String {
    topic.trim().to_ascii_lowercase()
}

fn render_markdown(input: &str) -> String {
    let mut out = Vec::new();
    let mut in_code_block = false;
    let mut code_fence_len = 0usize;

    for line in input.lines() {
        let trimmed = line.trim();

        if let Some((fence_len, lang)) = parse_fence(trimmed) {
            if in_code_block {
                if fence_len == code_fence_len {
                    out.push(String::new());
                    in_code_block = false;
                    code_fence_len = 0;
                    continue;
                }
            } else {
                if lang.is_empty() {
                    out.push("Code:".bright_magenta().bold().to_string());
                } else {
                    out.push(format!(
                        "{} {}",
                        "Code:".bright_magenta().bold(),
                        lang.dimmed()
                    ));
                }
                in_code_block = true;
                code_fence_len = fence_len;
                continue;
            }
        }

        if in_code_block {
            out.push(format!("    {line}"));
            continue;
        }

        if trimmed.is_empty() {
            out.push(String::new());
            continue;
        }

        if let Some(title) = trimmed.strip_prefix("# ") {
            out.push(title.to_uppercase().bright_yellow().bold().to_string());
            continue;
        }
        if let Some(title) = trimmed.strip_prefix("## ") {
            out.push(title.bright_blue().bold().to_string());
            continue;
        }
        if let Some(title) = trimmed.strip_prefix("### ") {
            out.push(format!(
                "{} {}",
                "-".bright_green().bold(),
                title.bright_green().bold()
            ));
            continue;
        }
        if let Some(title) = trimmed.strip_prefix("#### ") {
            out.push(format!(
                "{} {}",
                "*".bright_cyan().bold(),
                title.bright_cyan()
            ));
            continue;
        }

        if let Some(item) = trimmed.strip_prefix("- ") {
            out.push(format!("  - {}", render_inline(item)));
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("* ") {
            out.push(format!("  - {}", render_inline(item)));
            continue;
        }

        if let Some((num, rest)) = split_ordered_item(trimmed) {
            out.push(format!("  {}. {}", num.cyan(), render_inline(rest)));
            continue;
        }

        if let Some(quote) = trimmed.strip_prefix("> ") {
            out.push(format!(
                "  {} {}",
                "|".dimmed(),
                render_inline(quote).italic()
            ));
            continue;
        }

        out.push(render_inline(trimmed));
    }

    out.join("\n")
}

fn parse_fence(line: &str) -> Option<(usize, &str)> {
    let fence_len = line.chars().take_while(|&ch| ch == '`').count();
    if fence_len < 3 {
        return None;
    }
    Some((fence_len, line[fence_len..].trim()))
}

fn split_ordered_item(line: &str) -> Option<(&str, &str)> {
    let (num, rest) = line.split_once(". ")?;
    if num.chars().all(|ch| ch.is_ascii_digit()) {
        Some((num, rest))
    } else {
        None
    }
}

fn render_inline(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`'
            && let Some(end) = chars[i + 1..].iter().position(|&ch| ch == '`')
        {
            let code: String = chars[i + 1..i + 1 + end].iter().collect();
            out.push_str(&code.bright_cyan().to_string());
            i += end + 2;
            continue;
        }

        if i + 1 < chars.len()
            && chars[i] == '*'
            && chars[i + 1] == '*'
            && let Some(end) = find_double_marker(&chars, i + 2, '*')
        {
            let text: String = chars[i + 2..end].iter().collect();
            out.push_str(&text.bold().to_string());
            i = end + 2;
            continue;
        }

        if chars[i] == '*'
            && let Some(end) = chars[i + 1..].iter().position(|&ch| ch == '*')
        {
            let text: String = chars[i + 1..i + 1 + end].iter().collect();
            out.push_str(&text.italic().to_string());
            i += end + 2;
            continue;
        }

        if chars[i] == '['
            && let Some(close_bracket) = chars[i + 1..].iter().position(|&ch| ch == ']')
        {
            let close_bracket = i + 1 + close_bracket;
            if close_bracket + 1 < chars.len()
                && chars[close_bracket + 1] == '('
                && let Some(close_paren) =
                    chars[close_bracket + 2..].iter().position(|&ch| ch == ')')
            {
                let label: String = chars[i + 1..close_bracket].iter().collect();
                out.push_str(&label.underline().to_string());
                i = close_bracket + 2 + close_paren + 1;
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }
    out
}

fn find_double_marker(chars: &[char], start: usize, marker: char) -> Option<usize> {
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == marker && chars[i + 1] == marker {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{render_markdown, render_topic, resolve, topic_keys};

    #[test]
    fn resolve_by_primary_key() {
        let topic = resolve("gx.cmd").expect("gx.cmd should resolve");
        assert_eq!(topic.key, "gx.cmd");
    }

    #[test]
    fn resolve_by_alias() {
        let topic = resolve("cmd").expect("cmd alias should resolve");
        assert_eq!(topic.key, "gx.cmd");
    }

    #[test]
    fn resolve_gx_cli_topic() {
        let topic = resolve("gx").expect("gx topic should resolve");
        assert_eq!(topic.key, "gx");
    }

    #[test]
    fn topics_include_patch_file() {
        let keys = topic_keys();
        assert!(keys.contains(&"gx.patch_file"));
    }

    #[test]
    fn topics_include_sn() {
        let keys = topic_keys();
        assert!(keys.contains(&"gx.sn"));
    }

    #[test]
    fn render_markdown_formats_code_blocks() {
        let rendered = render_markdown("## 语法\n\n```gxl\ngx.cmd(\"echo hi\");\n```");
        assert!(rendered.contains("语法"));
        assert!(rendered.contains("Code:"));
        assert!(rendered.contains("    gx.cmd(\"echo hi\");"));
    }

    #[test]
    fn render_markdown_formats_lists_and_inline_code() {
        let rendered = render_markdown("- `gx.cmd`：执行命令");
        assert!(rendered.contains("gx.cmd"));
        assert!(rendered.contains("执行命令"));
    }

    #[test]
    fn render_topic_markdown_is_pure_markdown() {
        let topic = resolve("gx.cmd").expect("gx.cmd should resolve");
        let rendered = render_topic(topic, true);

        assert!(rendered.starts_with("# gx.cmd"));
        assert!(!rendered.contains("topic:"));
        assert!(!rendered.contains("source:"));
    }
}
