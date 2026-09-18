use std::fmt::Display;

use crate::components::gxl_spc::GxlSpace;

use super::prelude::*;
use winnow::{
    Parser, Result,
    ascii::multispace0,
    combinator::{alt, fail, opt},
    error::ContextError,
};

use super::stc_mod::gal_stc_mod;

pub struct WinnowErrorEx(ContextError);

impl Display for WinnowErrorEx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut context_vec: Vec<String> = collect_context(&self.0);
        context_vec.reverse();
        writeln!(f, "parse syntax :",)?;
        for context in context_vec {
            write!(f, "{context}::")?;
        }
        Ok(())
    }
}

fn collect_context(err: &ContextError) -> Vec<String> {
    let mut context_vec = Vec::new();
    let current = err;

    for context in current.context() {
        match context {
            winnow::error::StrContext::Label(value) => {
                context_vec.push(value.to_string());
            }
            winnow::error::StrContext::Expected(value) => {
                context_vec.push(value.to_string());
            }
            _ => {}
        }
    }
    context_vec
}
impl From<ContextError> for WinnowErrorEx {
    fn from(err: ContextError) -> Self {
        WinnowErrorEx(err)
    }
}

pub fn gal_stc_spc(input: &mut &str) -> Result<GxlSpace> {
    skip_spaces_block(input)?;
    let mut spc = GxlSpace::default();
    let mut items = Vec::new();
    loop {
        skip_spaces_block.parse_next(input)?;
        if starts_with(alt(((multispace0, "mod"), (multispace0, "#["))), input) {
            let item = gal_stc_mod.context(wn_desc("<mod>")).parse_next(input)?;
            items.push(item);
        } else {
            break;
        }
    }
    let _ = opt(symbol_semicolon).parse_next(input)?;
    info!(target: "parse","mod count: {}", items.len());
    skip_spaces_block(input)?;
    if !input.is_empty() {
        return fail.context(wn_desc("<space-end>")).parse_next(input);
    }

    spc.append(items);
    Ok(spc)
}
#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;

    #[test]
    fn test_spc1() {
        let mut data = r#"
mod main{
  root = "xxx" ;
  name = "xxx" ;
  flow  api {
    gx.echo ( value : "${PRJ_ROOT}" ) ;
  } ;
  flow admin {
    gx.echo ( value : "${PRJ_ROOT}" ) ;
  } ;
  flow test : admin,api  ;
};
"#;
        let spc = run_gxl(gal_stc_spc, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(spc.mods().len(), 1);
    }

    #[test]
    fn test_spc2() {
        let mut data = r#"
mod envs {
    env dev {
        root = "HOME";
    }
}
mod main{
  root = "xxx" ;
  name = "xxx" ;
  flow  api {
    gx.echo ( value : "${PRJ_ROOT}" ) ;
  } ;
  flow admin {
    gx.echo ( value : "${PRJ_ROOT}" ) ;
  } ;
  flow test : admin,api  ;
};
"#;
        let spc = run_gxl(gal_stc_spc, &mut data).assert();
        assert_eq!(data, "");
        assert_eq!(spc.mods().len(), 2);
    }
}
