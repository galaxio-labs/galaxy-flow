use super::prelude::*;

use orion_parse::atom::take_var_name;
use winnow::ascii::multispace0;
use winnow::combinator::preceded;
use winnow::combinator::separated;

use super::atom::spaced_desc;
use super::atom::take_var_ref_fmt;

#[derive(Default, Debug, Clone)]
pub struct STCHeadDto {
    pub keyword: String,
    pub name: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub args: Vec<(String, String)>,
}

#[derive(Default, Debug, Clone, Getters)]
pub struct EnvHeadDto {
    name: String,
    mix: Vec<String>,
}
impl EnvHeadDto {
    pub fn new(name: String, mix: Vec<String>) -> Self {
        Self { name, mix }
    }
}
#[derive(Default, Debug, Clone, Getters)]
pub struct ModDto {
    name: String,
    mix: Vec<String>,
}
impl ModDto {
    pub fn new(name: String, mix: Vec<String>) -> Self {
        Self { name, mix }
    }
}

pub fn gal_env_head(input: &mut &str) -> Result<EnvHeadDto> {
    // Parse the keyword (e.g., "flow")
    spaced_desc("env", "<keyword:env>").parse_next(input)?;
    let first = take_var_name.parse_next(input)?;
    multispace0.parse_next(input)?;

    // Parse the before flows (e.g., "b-flow1,b-flow2")
    if starts_with(":", input) {
        let mix: Vec<String> = preceded(
            (multispace0, ':', multispace0),
            separated(
                0..,
                alt((take_var_path, take_var_ref_fmt)),
                (multispace0, ',', multispace0),
            ),
        )
        .parse_next(input)?;
        Ok(EnvHeadDto::new(first, mix))
    } else {
        Ok(EnvHeadDto::new(first, Vec::new()))
    }
}

pub fn gal_mod_head(input: &mut &str) -> Result<ModDto> {
    // Parse the keyword (e.g., "flow")
    spaced_desc("mod", "<keyword:mod>").parse_next(input)?;
    let first = take_var_name(input)?;
    multispace0.parse_next(input)?;

    if starts_with(":", input) {
        let mix: Vec<String> = preceded(
            (multispace0, ':', multispace0),
            separated(0.., take_var_path, (multispace0, ',', multispace0)),
        )
        .parse_next(input)?;
        Ok(ModDto::new(first, mix))
    } else {
        Ok(ModDto::new(first, Vec::new()))
    }
}

pub fn gal_act_head(act: &'static str, input: &mut &str) -> Result<String> {
    spaced_desc(act, act).parse_next(input)?;

    let key = take_var_name(input)?;
    multispace0.parse_next(input)?;
    Ok(key)
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;

    #[test]
    fn test_take_evn_head() {
        // Set up a parse context and relevant variables to pass in to the function
        let main_key: &str = "dev";
        let mut input: &str = r#"env dev: env1,env2"#;

        // Pass variables into the function and capture the results
        let dto = run_gxl(gal_env_head, &mut input).assert();
        assert_eq!(dto.name(), main_key);
        assert_eq!(
            dto.mix(),
            &["env1".to_string(), "env2".to_string()].to_vec()
        );
        assert_eq!(input, "");

        let mut input: &str = r#"env dev: env1,env2;"#;

        // Pass variables into the function and capture the results
        let dto = run_gxl(gal_env_head, &mut input).assert();
        assert_eq!(dto.name(), main_key);
        assert_eq!(
            dto.mix(),
            &["env1".to_string(), "env2".to_string()].to_vec()
        );
        assert_eq!(input, ";");
        // Expected results
    }
}
