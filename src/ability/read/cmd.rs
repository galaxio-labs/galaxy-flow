use crate::ability::prelude::*;
use crate::components::GxlProps;
use crate::expect::{LogicScope, ShellOption};

use crate::friendly::New2;

#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct CmdDTO {
    pub name: String,
    pub cmd: String,
    pub shell_opt: ShellOption,
}

impl CmdDTO {
    pub fn execute(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.read_cmd");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let cmd = self.cmd.clone();
        let name = self.name.clone();
        let cmd = exp.eval(&cmd)?;
        let (_exit_code, data, _) = gxl_sh!(
            LogicScope::Outer,
            ctx.path(),
            &cmd,
            &self.shell_opt,
            &exp,
            vars_dict.global()
        )?;
        let data_str = String::from_utf8(data)
            .source_raw_err(ExecReason::from_data(), "bad command result")?;
        let mut vars = GxlProps::new("cmd");
        vars.append(GxlVar::new(name, data_str.trim().to_string()));
        vars.export_props(ctx, vars_dict.global_mut(), "")?;
        Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ability::{read::integra::ReadMode, *};

    #[tokio::test]
    async fn read_cmd_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/example/conf");
        let dto = CmdDTO {
            name: "RG".to_string(),
            cmd: "echo galaxy-1.0".to_string(),
            shell_opt: ShellOption::default(),
        };
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
}
