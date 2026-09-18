use std::path::PathBuf;
use std::sync::mpsc::Sender;

use orion_error::conversion::{ConvErr, SourceErr};

use crate::ability::prelude::*;

use crate::cmd::GxlCmd;
use crate::execution::runnable::AsyncRunnableWithSenderTrait;
use crate::util::redirect::ReadSignal;
use crate::{runner::GxlRunner, util::path::WorkDir};

#[derive(Clone, Debug, Default, Builder, PartialEq, Getters)]
pub struct GxRun {
    run_path: String,
    gxl_path: String,
    env_conf: String,
    env_isolate: bool,
    flow_cmd: String,
}
impl GxRun {
    pub fn new<S>(run_path: S, gxl_path: S, env_conf: S, flow_cmd: S, env_isolate: bool) -> Self
    where
        S: Into<String> + Clone,
    {
        Self {
            run_path: run_path.into(),
            gxl_path: gxl_path.into(),
            env_conf: env_conf.into(),
            flow_cmd: flow_cmd.into(),
            env_isolate,
        }
    }
}
#[async_trait]
impl AsyncRunnableWithSenderTrait for GxRun {
    async fn async_exec(
        &self,
        mut ctx: ExecContext,
        vars_dict: VarSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult {
        ctx.append("gx.run");
        let mut action = Action::from("gx.run");

        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let cmd = ctx.gxl_cmd().as_ref().clone();
        let cmd = cmd
            .with_env(exp.eval(&self.env_conf)?)
            .with_conf(Some(exp.eval(&self.gxl_path)?));

        let run_path = PathBuf::from(exp.eval(&self.run_path)?);
        let _g = WorkDir::change(run_path.clone())
            .source_err(UvsReason::resource_error().into(), "source error")
            .with_context(&run_path)?;
        do_gxl_run(cmd, &vars_dict, self.env_isolate, sender).await?;
        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
}
impl ComponentMeta for GxRun {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.gxl")
    }
}
pub async fn do_gxl_run(
    cmd: GxlCmd,
    vars_dict: &VarSpace,
    isolate: bool,
    sender: Option<Sender<ReadSignal>>,
) -> ExecResult<TaskValue> {
    let sub_var_space = VarSpace::inherit_init(vars_dict.clone(), isolate)?;
    GxlRunner::run(cmd, sub_var_space, sender).await.conv_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ability::*;

    #[ignore = "will change current run dir"]
    #[tokio::test(flavor = "current_thread")]
    async fn gxl_run_test() {
        let (context, def) = ability_env_init();
        let res = GxRun::new(
            "./examples/assert",
            "_gal/work.gxl",
            "default",
            "assert_main",
            true,
        );
        res.async_exec(context, def, None).await.unwrap();
    }
}
