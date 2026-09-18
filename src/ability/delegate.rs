use derive_more::From;
use getset::{Getters, WithSetters};
use orion_error::conversion::ToStructError;
use orion_infra::auto_exit_log;

use crate::ability::prelude::*;

use crate::components::gxl_act::activity::Activity;
use crate::components::gxl_fun::fun::GxlFun;
use crate::components::gxl_spc::GxlSpace;
use crate::execution::runnable::AsyncRunnableArgsTrait;
use crate::model::components::gxl_utls::mod_obj_name;

use crate::primitive::{GxlAParam, GxlAParams, GxlFParams};
use crate::traits::DependTrait;

#[derive(Clone, From)]
pub enum ActTypes {
    Fun(GxlFun),
    Act(Activity),
}
#[derive(Clone, Default, Builder, Getters, WithSetters)]
#[getset(get = "pub")]
pub struct ActCall {
    pub name: String,
    pub sudo: bool,
    pub actual_params: GxlAParams,
    #[getset(set_with = "pub")]
    assembled: bool,
    act: Option<ActTypes>,
}

impl From<String> for ActCall {
    fn from(name: String) -> Self {
        Self {
            name,
            sudo: false,
            actual_params: GxlAParams::new(),
            act: None,
            assembled: false,
        }
    }
}
impl From<(String, Vec<GxlAParam>)> for ActCall {
    fn from(value: (String, Vec<GxlAParam>)) -> Self {
        let mut args = GxlAParams::new();
        value.1.into_iter().for_each(|x| {
            args.insert(x.name().clone(), x);
        });
        Self {
            name: value.0,
            sudo: false,
            actual_params: args,
            assembled: false,
            act: None,
        }
    }
}

impl DependTrait<&GxlSpace> for ActCall {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        if self.assembled {
            return Ok(self);
        }
        let (find_mod, call_name) = mod_obj_name(mod_name, self.name.as_str());
        let mod_log_name = find_mod.clone();
        let mut flag = auto_exit_log!(
            info!(target: "assemble", "assembled  success!:{mod_log_name}.{call_name}"),
            error!(target: "assemble", "assembled failed!:{mod_log_name}.{call_name}" )
        );
        if let Some(found_mod) = src.get(&find_mod) {
            if let Some(act) = found_mod.acts().get(&call_name) {
                let act = act.clone().assemble(mod_name, src)?;
                flag.mark_suc();
                return Ok(ActCall::from((self.clone(), act, find_mod)).with_assembled(true));
            } else if let Some(fun) = found_mod.funs().get(&call_name) {
                self.check_param(fun.meta().params(), find_mod.as_str(), call_name.as_str())?;
                let fun = fun.clone().assemble(mod_name, src)?;
                flag.mark_suc();
                return Ok(ActCall::from((self.clone(), fun, find_mod)).with_assembled(true));
            }
        }
        error!(
            "call not found: {find_mod}.{call_name} origin {mod_name}.{} ",
            self.name,
        );
        Err(AssembleError::from(AssembleReason::Miss(format!(
            "activity: {find_mod}.{call_name}"
        ))))
    }
}

impl ActCall {
    fn check_param(&self, f_params: &GxlFParams, find_mod: &str, call_name: &str) -> AResult<()> {
        for param in f_params {
            let found = if param.is_default() {
                //use default actura name
                self.actual_params
                    .get(param.name())
                    .or(self.actual_params.get("default"))
            } else {
                self.actual_params.get(param.name())
            };

            if found.is_none() && param.default_value().is_none() {
                return AssembleReason::Miss(format!(
                    "{} for call {find_mod}.{call_name}",
                    param.name()
                ))
                .err_result();
            }
        }
        Ok(())
    }
}
impl From<(Self, Activity, String)> for ActCall {
    fn from(value: (Self, Activity, String)) -> Self {
        let mut ins = value.0;
        let cur_act = value.1;
        //cur_act.set_host(value.2);
        //cur_act.merge_prop(ins.args.clone());
        ins.act = Some(ActTypes::from(cur_act));
        ins
    }
}

impl From<(Self, GxlFun, String)> for ActCall {
    fn from(value: (Self, GxlFun, String)) -> Self {
        let mut ins = value.0;
        let cur_fun = value.1;
        //cur_fun.set_host(host);
        //cur_fun.merge_prop(self.props.clone());
        ins.act = Some(ActTypes::from(cur_fun));
        ins
    }
}
#[async_trait]
impl AsyncRunnableTrait for ActCall {
    async fn async_exec(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("@");
        match &self.act {
            Some(act) => act.async_exec(ctx, vars_dict, &self.actual_params).await,
            None => Err(ExecReason::Gxl
                .to_err()
                .with_detail(format!("act call not support :{}", self.name))),
        }
    }
}

#[async_trait]
impl AsyncRunnableArgsTrait for ActTypes {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace, args: &GxlAParams) -> TaskResult {
        match self {
            ActTypes::Fun(o) => o.async_exec(ctx, vars, args).await,
            ActTypes::Act(o) => o.async_exec(ctx, vars, args).await,
        }
    }
}
