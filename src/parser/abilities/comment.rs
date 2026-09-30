use super::prelude::*;

#[derive(Debug)]
enum DslStatus {
    Comment,
    MultiComment,
    Code,
    Data,
    RawData,
    RawBlock,
}
fn ignore_comment_line(status: &mut DslStatus, input: &mut &str) -> Result<String> {
    //let mut status = DslStatus::Code;
    let mut out = String::new();
    loop {
        if input.is_empty() {
            break;
        }
        match status {
            DslStatus::Code => {
                let code = take_while(0.., |c| c != '"' && c != '/' && c != '^' && c != '`')
                    .parse_next(input)?;
                out += code;
                if input.is_empty() {
                    break;
                }
                let rst = opt("^\"").parse_next(input)?;
                if let Some(code) = rst {
                    out += code;
                    *status = DslStatus::RawData;
                    continue;
                }
                let rst = opt("```").parse_next(input)?;
                if let Some(code) = rst {
                    out += code;
                    *status = DslStatus::RawBlock;
                    continue;
                }

                let rst = opt("\"").parse_next(input)?;
                if let Some(code) = rst {
                    out += code;
                    *status = DslStatus::Data;
                    continue;
                }
                if opt("//").parse_next(input)?.is_some() {
                    *status = DslStatus::Comment;
                    continue;
                }
                if opt("/*").parse_next(input)?.is_some() {
                    *status = DslStatus::MultiComment;
                    continue;
                }
                return fail.context(wn_desc("end-code")).parse_next(input);
            }
            DslStatus::RawData => match opt(take_until(0.., "\"^")).parse_next(input)? {
                Some(data) => {
                    out += data;
                    let data = "\"^".parse_next(input)?;
                    out += data;
                    *status = DslStatus::Code;
                }
                None => {
                    let data = till_line_ending.parse_next(input)?;
                    out += data;
                }
            },
            DslStatus::RawBlock => match opt(take_until(0.., "```")).parse_next(input)? {
                Some(data) => {
                    out += data;
                    let data = "```".parse_next(input)?;
                    out += data;
                    *status = DslStatus::Code;
                }
                None => {
                    let data = till_line_ending.parse_next(input)?;
                    out += data;
                }
            },

            DslStatus::Data => {
                //dbg_in(in_ctx.path(), cur)?;
                let data = take_till(0.., |c| c == '"').parse_next(input)?;
                out += data;
                let data = "\"".parse_next(input)?;
                out += data;
                *status = DslStatus::Code;
            }
            DslStatus::Comment => {
                //TODO: 或到字符串结束
                let _ = till_line_ending.parse_next(input)?;
                *status = DslStatus::Code;
            }
            DslStatus::MultiComment => match opt(take_until(0.., "*/")).parse_next(input)? {
                Some(_) => {
                    let _ = "*/".parse_next(input)?;
                    *status = DslStatus::Code;
                }
                None => {
                    let _ = till_line_ending.parse_next(input)?;
                }
            },
        }
    }
    Ok(out)
}

pub fn ignore_comment(input: &mut &str) -> Result<String> {
    let mut status = DslStatus::Code;
    let mut out = String::new();
    loop {
        if input.is_empty() {
            break;
        }
        let mut line = till_line_ending.parse_next(input)?;
        let code = ignore_comment_line(&mut status, &mut line)?;
        out += code.as_str();
        if opt(line_ending).parse_next(input)?.is_some() {
            match status {
                DslStatus::MultiComment => {}
                DslStatus::RawData => {}
                _ => {
                    out += "\n";
                }
            }
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;
    #[test]
    fn test_comment() {
        let mut data = "hello //xxx\nboy";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "hello \nboy");

        let mut data = "	// need galaxy 0.4.1";
        let codes = run_gxl(ignore_comment, &mut data).unwrap();
        assert_eq!(codes, "\t");

        let mut data = "\"hello //\"\nboy";
        let expect = data;
        let codes = run_gxl(ignore_comment, &mut data).unwrap();
        assert_eq!(codes, expect);

        let mut data = "\"hello //\"//xxx\nboy";
        let expect = "\"hello //\"\nboy";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, expect);

        let mut data = "hello^\"//you\"^";
        let expect = "hello^\"//you\"^";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, expect);
    }

    #[test]
    fn test_multi_line_comment() {
        let mut data = "hello /* multi-line \n comment */ world";
        let codes = run_gxl(ignore_comment, &mut data).unwrap();
        assert_eq!(codes, "hello  world");

        let mut data = "hello /* multi-line \n comment */ world\n// single-line comment";
        let codes = run_gxl(ignore_comment, &mut data).unwrap();
        assert_eq!(codes, "hello  world\n");

        let mut data =
            "hello /* multi-line \n comment */ world\n/* another multi-line \n comment */";
        let codes = run_gxl(ignore_comment, &mut data).unwrap();
        assert_eq!(codes, "hello  world\n");
    }

    #[test]
    fn test_comment_in_string() {
        let mut data = "\"hello /* not a comment */ world\"";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "\"hello /* not a comment */ world\"");

        let mut data = "\"hello // not a comment\"\nworld";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "\"hello // not a comment\"\nworld");

        let mut data = "\"hello /* not a comment */ world\"\n// single-line comment";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "\"hello /* not a comment */ world\"\n");
    }

    #[test]
    fn test_mixed_comments_and_code() {
        let mut data = "code /* comment */ more code // another comment\nfinal code";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "code  more code \nfinal code");

        let mut data = "code /* comment */ more code /* another comment */ final code";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "code  more code  final code");
    }

    #[test]
    fn test_empty_string() {
        let mut data = "";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "");
    }

    #[test]
    fn test_only_comments() {
        let mut data = "// single-line comment";
        let codes = run_gxl(ignore_comment, &mut data).unwrap();
        assert_eq!(codes, "");

        let mut data = "/* multi-line comment */";
        let codes = run_gxl(ignore_comment, &mut data).unwrap();
        assert_eq!(codes, "");

        let mut data = "// single-line comment\n/* multi-line comment */";
        let codes = run_gxl(ignore_comment, &mut data).unwrap();
        assert_eq!(codes, "\n");
    }

    #[test]
    fn test_only_code() {
        let mut data = "code";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "code");

        let mut data = "code\nmore code";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "code\nmore code");
    }

    #[test]
    fn test_only_data() {
        let mut data = "\"data\"";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "\"data\"");

        let mut data = "\"data\"\n\"more data\"";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "\"data\"\n\"more data\"");
    }

    #[test]
    fn test_only_raw_data() {
        let mut data = "^\"raw data\"^";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "^\"raw data\"^");

        let mut data = "^\"raw data\"^\n^\"more raw data\"^";
        let codes = ignore_comment(&mut data).unwrap();
        assert_eq!(codes, "^\"raw data\"^\n^\"more raw data\"^");
    }

    #[test]
    fn test_complex_mixed_case1() {
        let mut data = r#"
        code /* multi-line comment */ "string with // comment"
        // single-line comment
        ^"raw data with /* comment */"^
        /* another multi-line comment */
        more code
        "#;
        let _ = ignore_comment(&mut data).assert();
    }

    #[test]
    fn test_complex_mixed_case2() {
        let mut data = r#"
        code /* multi-line comment */ "string with // comment"
        // single-line comment
        ```
        cp  ./a.txt  ./b.txt
        /*

        ```
        /* another multi-line comment */
        more code
        "#;
        let _ = ignore_comment(&mut data).assert();
    }
}
