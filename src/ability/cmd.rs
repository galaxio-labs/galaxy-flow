use crate::{ability::prelude::*, expect::LogicScope};

#[derive(Clone, Debug, Default, Builder, PartialEq, Getters)]
pub struct GxCmd {
    dto: GxCmdDto,
}
#[derive(Clone, Debug, Builder, PartialEq, Default)]
pub struct GxCmdDto {
    pub cmd: String,
    pub shell_opt: ShellOption,
}
impl GxCmdDto {
    pub fn update(&mut self, vars_dict: &VarSpace) -> ExecResult<()> {
        let ee = EnvExpress::from_env_mix(vars_dict.global().clone());
        self.cmd = ee.eval(&self.cmd)?;
        Ok(())
    }
}
#[async_trait]
impl AsyncRunnableTrait for GxCmd {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.execute_impl(&self.dto.cmd, ctx, vars_dict)
    }
}
impl ComponentMeta for GxCmd {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.cmd")
    }
}

impl GxCmd {
    pub fn new(forword: String) -> Self {
        let dto = GxCmdDto {
            cmd: forword,
            ..Default::default()
        };
        Self::dto_new(dto)
    }
    pub fn dto_new(dto: GxCmdDto) -> Self {
        GxCmd { dto }
    }
    fn execute_impl(&self, cmd: &String, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.cmd");
        let mut action = Action::from("gx.cmd");
        trace!(target:ctx.path(),"cmd:{cmd}", );
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let exe_cmd = exp.eval(cmd)?;

        let mut shell_opt = self.dto.shell_opt.clone();
        shell_opt.quiet = ctx.quiet();

        let res = gxl_sh!(
            LogicScope::Outer,
            ctx.tag_path("cmd").as_str(),
            &exe_cmd,
            &shell_opt,
            &exp,
            vars_dict.global()
        );
        match res {
            Ok((exit_code, stdout, stderr)) => {
                let out = String::from_utf8(stdout)
                    .source_raw_err(ExecReason::data_error(), "decode command stdout as utf-8")?;
                let err = String::from_utf8(stderr)
                    .source_raw_err(ExecReason::data_error(), "decode command stderr as utf-8")?;
                action.set_command_output(exit_code, out, err);
            }
            Err(error) => {
                action.set_stderr(error.to_string());
                return Err(error);
            }
        }
        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
}

#[cfg(test)]
mod tests {
    use orion_error::dev::testing::TestAssertWithMsg;
    use std::path::PathBuf;

    use super::*;
    use crate::{ability::*, traits::Setter, util::path::WorkDirWithLock};

    #[tokio::test]
    async fn cmd_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");
        let res = GxCmd::new(
          "if test ! -L  ${CONF_ROOT}/ability.bak; then ln -s ${CONF_ROOT}/ability.gxl ${CONF_ROOT}/ability.bak;  fi ".into()
          ) ;
        let _ = res.async_exec(context, def).await.assert("dryrun");
    }

    #[tokio::test]
    async fn cmd_test_err() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/example/conf");
        //syntax error;
        let res = GxCmd::new(
          "if test ! -L  ${CONF_ROOT}/used/link2.txt ; then ln -s ${CONF_ROOT}/options/link.txt  ${CONF_ROOT}/used/link2.txt ; i ".into()
          ) ;
        let result = res.async_exec(context, def).await;
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn cmd_test_keeps_exit_code_stdout_and_stderr() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _workdir = WorkDirWithLock::change(&manifest_dir).expect("set manifest dir");
        let (context, def) = ability_env_init();
        let dto = GxCmdDto {
            cmd: "printf out && printf err 1>&2 && exit 2".into(),
            shell_opt: ShellOption {
                quiet: true,
                ok_codes: vec![0, 2],
                ..Default::default()
            },
        };
        let result = GxCmd::dto_new(dto)
            .async_exec(context, def)
            .await
            .assert("cmd success");
        let ExecOut::Action(action) = result.rec else {
            panic!("expected action output");
        };
        assert_eq!(action.exit_code, Some(2));
        assert_eq!(action.stdout, "out");
        assert_eq!(action.stderr, "err");
    }

    #[tokio::test]
    async fn cmd_test_stream_keeps_exit_code_stdout_and_stderr() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _workdir = WorkDirWithLock::change(&manifest_dir).expect("set manifest dir");
        let (context, def) = ability_env_init();
        let dto = GxCmdDto {
            cmd: "printf out && printf err 1>&2 && exit 2".into(),
            shell_opt: ShellOption {
                stream: true,
                quiet: true,
                ok_codes: vec![0, 2],
                ..Default::default()
            },
        };
        let result = GxCmd::dto_new(dto)
            .async_exec(context, def)
            .await
            .assert("cmd success");
        let ExecOut::Action(action) = result.rec else {
            panic!("expected action output");
        };
        assert_eq!(action.exit_code, Some(2));
        assert_eq!(action.stdout, "out");
        assert_eq!(action.stderr, "err");
    }
}
