use super::{gxl_flow::meta::FlowMeta, prelude::*};
use crate::{
    ability::prelude::TaskValue,
    cmd::GxlCmd,
    execution::{
        sequence::{ExecSequence, SequAppender, SequLoader},
        unit::{RunUnitGuard, RunUnitLable},
    },
    menu::*,
    meta::MetaInfo,
    util::{redirect::ReadSignal, task_report::task_local_report},
};
use colored::Colorize;
use contracts::requires;
use indexmap::IndexMap;
use orion_error::conversion::{ConvErr, ToStructError};
use std::{fmt::Display, sync::mpsc::Sender};

use super::GxlMod;

const MAIN_MOD: &str = "main";
const ENV_MOD: &str = "env";
const ENVS_MOD: &str = "envs";

#[derive(Clone, Default, Getters)]
pub struct GxlSpace {
    mods: IndexMap<String, GxlMod>,
    assembled: bool,
}

impl GxlSpace {
    pub fn get(&self, key: &str) -> Option<&GxlMod> {
        self.mods.get(key)
    }

    pub fn len(&self) -> usize {
        self.mods.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mods.is_empty()
    }

    pub fn main(&self) -> ExecResult<&GxlMod> {
        self.get(MAIN_MOD).ok_or_else(|| {
            ExecReason::Args
                .to_err()
                .with_detail(format!("'{MAIN_MOD}' mod not found"))
        })
    }

    pub fn env(&self) -> ExecResult<&GxlMod> {
        self.get(ENV_MOD)
            .or_else(|| self.get(ENVS_MOD))
            .or_else(|| self.get(MAIN_MOD))
            .ok_or_else(|| {
                ExecReason::Args
                    .to_err()
                    .with_detail("Neither 'envs' 'env' nor 'main' mod found")
            })
    }

    pub fn show(&self) -> ExecResult<()> {
        let menu = self.menu()?;
        eprintln!(
            "{}",
            "---------------prj work menu-------------".cyan().bold()
        );

        eprintln!("{}", "envs:".yellow());
        for choice in menu.envs() {
            show_item(choice);
        }

        eprintln!("\n{}", "flow:".yellow());
        for choice in menu.flows() {
            show_item(choice);
        }

        Ok(())
    }

    pub fn assemble(self) -> AResult<Self> {
        let mut spc_mix = self.clone();
        for (_, module) in self.mods {
            let ass_mod = module.assemble_mix(&spc_mix)?;
            spc_mix.replace(ass_mod);
        }
        let mut spc_ass = spc_mix.clone();

        for (mod_name, module) in spc_mix.mods {
            let updated = module.assemble(mod_name.as_str(), &spc_ass)?;
            debug_assert!(updated.assembled());
            spc_ass.replace(updated);
        }
        spc_ass.assembled = true;
        Ok(spc_ass)
    }

    fn replace(&mut self, updated: GxlMod) {
        self.mods.insert(updated.of_name(), updated);
    }
}

impl SequLoader for GxlSpace {
    fn find_flow(&self, meta: &FlowMeta, sequ: &mut impl SequAppender) -> ExecResult<()> {
        self.guard_load_flow(meta, &RunUnitGuard::from_flow(), sequ)
    }
}

impl AppendAble<GxlMod> for GxlSpace {
    fn append(&mut self, module: GxlMod) {
        let name = module.of_name();
        self.mods.insert(name, module);
    }
}
impl AppendAble<Vec<GxlMod>> for GxlSpace {
    fn append(&mut self, mod_vec: Vec<GxlMod>) {
        for item in mod_vec {
            self.append(item)
        }
    }
}

impl ExecLoadTrait for GxlSpace {
    #[requires(self.assembled)]
    fn load_env(
        &self,
        ctx: ExecContext,
        sequ: &mut ExecSequence,
        obj_path: &str,
    ) -> ExecResult<()> {
        let (mod_name, item_name) = parse_obj_path(obj_path)?;

        self.mods
            .get(mod_name)
            .ok_or_else(|| ExecReason::Miss.to_err().with_detail(mod_name.to_string()))?
            .load_env(ctx, sequ, item_name)
    }

    #[requires(self.assembled)]
    fn load_flow(
        &self,
        _ctx: ExecContext,
        sequ: &mut ExecSequence,
        obj_path: &str,
    ) -> ExecResult<()> {
        let (mod_name, item_name) = parse_obj_path(obj_path)?;

        let mox = self
            .mods
            .get(mod_name)
            .ok_or_else(|| ExecReason::Miss.to_err().with_detail(mod_name.to_string()))?;
        self.mod_load_flow(mox, item_name, &RunUnitGuard::from_flow(), sequ)
    }

    fn of_name(&self) -> String {
        "space".to_string()
    }

    fn menu(&self) -> ExecResult<GxMenu> {
        let mut main_menu = self.main()?.menu()?;
        let mut env_menu = self.env()?.menu()?;

        // Merge menus
        env_menu.flows.clear();
        main_menu.envs.clear();
        main_menu.envs.extend(env_menu.envs);

        Ok(main_menu)
    }
}

// Helper function to parse object paths
fn parse_obj_path(obj_path: &str) -> ExecResult<(&str, &str)> {
    let mut parts = obj_path.splitn(2, '.');

    match (parts.next(), parts.next()) {
        (Some(mod_name), Some(item_name)) => Ok((mod_name, item_name)),
        _ => Err(ExecReason::Gxl.to_err().with_detail(obj_path.to_string())),
    }
}

#[derive(Default, Debug, Clone)]
pub struct ExecOptions {
    dryrun: bool,
    out: bool,
}
impl ExecOptions {
    pub fn with_dryrun(mut self, dryrun: bool) -> Self {
        self.dryrun = dryrun;
        self
    }
    pub fn is_dryrun(&self) -> bool {
        self.dryrun
    }
    pub fn is_out(&self) -> bool {
        self.out
    }
}

impl GxlSpace {
    #[requires(self.assembled)]
    pub async fn exec(
        &self,
        cmd: GxlCmd,
        var_space: VarSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> RunResult<TaskValue> {
        info!(
            target: "execution",
            "Starting execution stack with output: {:?}", cmd.quiet,
        );
        let envs: Vec<String> = cmd.get_env_list();
        warn!(target : "exec","Executing with envs: {}, flow:{}", envs.join(",") , cmd.flows() );
        warn!(target : "exec","inherted vars :\n{}", var_space.inherited());
        info!(target : "exec","inherted vars :\n{}", var_space.global());

        let main_ctx = ExecContext::new(cmd.clone());
        let flow = cmd.flows();
        self.execute_flow(&main_ctx, &var_space, &envs, flow, sender.clone())
            .await
    }

    #[requires(self.assembled)]
    async fn execute_flow(
        &self,
        main_ctx: &ExecContext,
        var_space: &VarSpace,
        envs: &[String],
        flow_name: &str,
        sender: Option<Sender<ReadSignal>>,
    ) -> RunResult<TaskValue> {
        let flow_name = self.normalize_flow_name(flow_name);
        if !main_ctx.quiet() {
            eprintln!("execute flow: {}", flow_name);
        }

        let mut exec_sequ = ExecSequence::from("flow");
        let mut ctx = main_ctx.clone();

        self.load_envs(&mut ctx, envs, &mut exec_sequ)?;

        let flow_ctx = main_ctx.clone();
        self.load_flow(flow_ctx, &mut exec_sequ, &flow_name)
            .conv_err()?;

        let exec_ctx = main_ctx.clone().with_subcontext("exec");

        match exec_sequ
            .execute(exec_ctx, var_space.clone(), self, sender)
            .await
            .conv_err()
        {
            Ok(task) => {
                task_local_report(task.rec().clone());
                Ok(task)
            }
            Err(do_err) => Err(do_err),
        }
    }

    fn normalize_flow_name(&self, name: &str) -> String {
        if name.contains('.') {
            name.to_string()
        } else {
            format!("{MAIN_MOD}.{name}",)
        }
    }

    fn load_envs(
        &self,
        ctx: &mut ExecContext,
        envs: &[String],
        exec_sequ: &mut ExecSequence,
    ) -> RunResult<()> {
        ctx.append("env");

        for env in envs {
            let env_paths = [
                env.as_str(),
                &format!("{MAIN_MOD}.{env}",),
                &format!("{ENV_MOD}.{env}",),
                &format!("{ENVS_MOD}.{env}",),
            ];

            if let Some(found) = env_paths
                .iter()
                .find(|path| self.load_env(ctx.clone(), exec_sequ, path).is_ok())
            {
                info!("Loaded environment: {found}",);
                continue;
            }

            return Err(RunReason::Args
                .to_err()
                .with_detail(format!("Environment '{env}' not found")));
        }

        Ok(())
    }

    fn guard_load_flow(
        &self,
        meta: &FlowMeta,
        guard: &RunUnitGuard,
        sequ: &mut impl SequAppender,
    ) -> ExecResult<()> {
        if let Some(mod_meta) = meta.host()
            && let Some(mox) = self.mods.get(mod_meta.name())
        {
            return self.mod_load_flow(mox, meta.name(), guard, sequ);
        }
        Err(ExecReason::Miss.to_err().with_detail(meta.long_name()))
    }
    fn mod_load_flow(
        &self,
        mox: &GxlMod,
        name: &str,
        guard: &RunUnitGuard,
        sequ: &mut impl SequAppender,
    ) -> ExecResult<()> {
        match mox.flows().get(name) {
            Some(found) => {
                debug_assert!(found.assembled());
                debug_assert!(mox.assembled());
                sequ.append_hold(&RunUnitGuard::from_mod(mox.meta()), mox.props().clone());
                if let RunUnitLable::Flow = guard.lable() {
                    for x in mox.entrys().iter() {
                        let guard = RunUnitGuard::from_entry(x);
                        self.guard_load_flow(x, &guard, sequ)?;
                    }
                }
                let pre_flows = found.meta().pre_metas();
                let post_flows = found.meta().pos_metas();
                for flow in pre_flows.iter() {
                    self.guard_load_flow(flow, guard, sequ)?;
                }
                sequ.append_hold(guard, found.clone());
                for flow in post_flows.iter() {
                    self.guard_load_flow(flow, guard, sequ)?;
                }
                if let RunUnitLable::Flow = guard.lable() {
                    for x in mox.exits().iter() {
                        let guard = RunUnitGuard::from_exit(x);
                        self.guard_load_flow(x, &guard, sequ)?;
                    }
                }
                Ok(())
            }
            None => Err(ExecReason::Miss.to_err().with_detail(name)),
        }
    }
}

// UI helper functions
pub fn show_item(item: &MenuItem) {
    if item.key.starts_with('_') {
        return;
    }

    let display_text = match &item.desp {
        Some(desp) => format!("    * {:<20} -- {}", item.key, desp),
        None => format!("    * {:<20}", item.key),
    };

    color_show(display_text, item.color.as_deref());
}

pub fn color_show<S: AsRef<str> + Display>(text: S, color: Option<&str>) {
    let colored_text = match color {
        Some("red") => text.as_ref().red(),
        Some("green") => text.as_ref().green(),
        Some("blue") => text.as_ref().blue(),
        Some("yellow") => text.as_ref().yellow(),
        Some("cyan") => text.as_ref().cyan(),
        Some("magenta") => text.as_ref().magenta(),
        Some("black") => text.as_ref().black(),
        Some("white") => text.as_ref().white(),
        Some("purple") => text.as_ref().purple(),
        _ => return eprintln!("{text}"),
    };

    eprintln!("{colored_text}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::friendly::New2;
    use crate::{
        ability::prelude::GxlVar,
        components::{GxlEnv, GxlFlow, GxlMod, GxlProps, gxl_mod::meta::ModMeta},
        execution::exec_init_env,
        types::AnyResult,
    };
    use orion_error::dev::testing::TestAssert;

    #[tokio::test]
    async fn execute_forward() -> AnyResult<()> {
        let (ctx, def) = exec_init_env();

        // Create main module
        let mut main_mod = GxlMod::from(ModMeta::build_mod(MAIN_MOD));
        main_mod.append(GxlVar::new("key1", "val1"));

        let flow = GxlFlow::load_ins("flow1");
        main_mod.append(flow);

        // Create environment module
        let mut env_mod = GxlMod::from(ModMeta::build_mod(ENV_MOD));

        let mut env = GxlEnv::from("env1");
        env.append(GxlVar::new("key1", "val1"));

        let mut rg_vars = GxlProps::default();
        rg_vars.append(GxlVar::new("key1", "val1"));
        env.append(rg_vars);

        env_mod.append(env);

        // Build code space
        let mut code_space = GxlSpace::default();
        code_space.append(env_mod);
        code_space.append(main_mod);

        // Execute
        let mut flow = ExecSequence::from("test");
        let work_space = code_space.assemble().assert();

        work_space
            .load_env(ctx.clone(), &mut flow, "env.env1")
            .assert();
        work_space
            .load_flow(ctx.clone(), &mut flow, "main.flow1")
            .assert();

        let task_v = flow
            .test_execute(ctx, def, &work_space, None)
            .await
            .unwrap();
        debug!("Job result: {task_v:#?}",);

        work_space.show().unwrap();
        Ok(())
    }
}
