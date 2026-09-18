use std::io;

use crate::ability::prelude::*;
use crate::components::GxlProps;

use crate::friendly::New2;
use derive_more::From;
use getset::{Getters, MutGetters, Setters, WithSetters};

pub trait InputReader {
    fn read_line(&self, buffer: &mut String) -> io::Result<usize>;
}
#[derive(Debug, PartialEq, Clone)]
pub struct StdinReader;

#[derive(From, Debug, PartialEq, Clone)]
pub enum InReaderTypes {
    Stdin(StdinReader),
    Mock(MockReader),
}

impl InputReader for StdinReader {
    fn read_line(&self, buffer: &mut String) -> io::Result<usize> {
        io::stdin().read_line(buffer)
    }
}

impl InputReader for InReaderTypes {
    fn read_line(&self, buffer: &mut String) -> io::Result<usize> {
        match self {
            InReaderTypes::Stdin(o) => o.read_line(buffer),
            InReaderTypes::Mock(o) => o.read_line(buffer),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MockReader {
    input: String,
}

impl InputReader for MockReader {
    fn read_line(&self, buffer: &mut String) -> io::Result<usize> {
        buffer.push_str(&self.input);
        Ok(self.input.len())
    }
}
#[derive(Getters, Setters, WithSetters, MutGetters, Debug, PartialEq, Clone)]
pub struct StdinDTO {
    #[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
    name: String,
    #[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
    prompt: String,
    input: InReaderTypes,
}
impl Default for StdinDTO {
    fn default() -> Self {
        Self {
            name: String::new(),
            prompt: String::new(),
            input: InReaderTypes::Stdin(StdinReader),
        }
    }
}
impl StdinDTO {
    pub fn new(input: InReaderTypes) -> Self {
        Self {
            name: String::new(),
            prompt: String::new(),
            input,
        }
    }
}

impl StdinDTO {
    pub fn execute(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.read_ini");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let msg = self.prompt.clone();
        let name = self.name.clone();
        let msg = exp.eval(&msg)?;
        println!("{msg}");
        let mut buffer = String::new();
        let stdin = io::stdin(); // We get `Stdin` here.
        stdin
            .read_line(&mut buffer)
            .source_err(UvsReason::data_error().into(), "source error")?;
        let mut vars = GxlProps::new("stdio");
        vars.append(GxlVar::new(name, buffer.trim().to_string()));
        vars.export_props(ctx, vars_dict.global_mut(), "")?;
        Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ability::{read::integra::ReadMode, *};

    #[ignore]
    #[tokio::test]
    async fn read_stdin_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/example/conf");
        let dto = StdinDTO::new(InReaderTypes::from(MockReader {
            input: "system".to_string(),
        }))
        .with_name(String::from("name"))
        .with_prompt(String::from("please input you name"));
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
}
