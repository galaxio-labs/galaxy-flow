use chrono::Local;
use orion_conf::{IniIO, JsonIO, TomlIO, YamlIO};
use rand::RngExt;
use std::path::PathBuf;

use crate::{ability::prelude::*, expect::LogicScope, traits::Setter, var::VarDict};
use getset::{Getters, MutGetters, Setters, WithSetters};
use orion_variate::vars::ValueDict;
#[derive(Clone, Debug, Default, PartialEq, Getters, Setters, WithSetters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut, set_with)]
pub struct GxShell {
    arg_file: Option<PathBuf>,
    out_var: Option<String>,
    shell: String,
    shell_opt: ShellOption,
}
#[async_trait]
impl AsyncRunnableTrait for GxShell {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.execute_impl(ctx, vars_dict)
    }
}
impl ComponentMeta for GxShell {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.cmd")
    }
}

impl GxShell {
    pub fn new<S: Into<String>>(shell: S) -> Self {
        Self {
            shell: shell.into(),
            ..Default::default()
        }
    }
    fn execute_impl(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.shell");
        let mut action = Action::from("gx.shell");
        trace!(target:ctx.path(),"shell:{}", self.shell);
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let ext_cmd = exp.eval(self.shell.as_str())?;
        let mut shell_opt = self.shell_opt.clone();

        shell_opt.quiet = ctx.quiet();
        if let Some(arg_file) = &self.arg_file {
            let dict = match arg_file.extension() {
                Some(ext) if ext == "json" => ValueDict::load_json(arg_file).map_err(|e| {
                    ExecReason::Serde
                        .to_err()
                        .with_detail(format!("JSON解析失败: {e}"))
                })?,
                Some(ext) if ext == "yml" || ext == "yaml" => ValueDict::load_yaml(arg_file)
                    .map_err(|e| {
                        ExecReason::Serde
                            .to_err()
                            .with_detail(format!("YAML解析失败: {e}"))
                    })?,
                Some(ext) if ext == "toml" => ValueDict::load_toml(arg_file).map_err(|e| {
                    ExecReason::Serde
                        .to_err()
                        .with_detail(format!("TOML解析失败: {e}"))
                })?,
                Some(ext) if ext == "ini" => ValueDict::load_ini(arg_file).map_err(|e| {
                    ExecReason::Serde
                        .to_err()
                        .with_detail(format!("INI解析失败: {e}"))
                })?,
                _ => {
                    return Err(ExecReason::from_logic()
                        .to_err()
                        .with_detail(format!("unsupport this format {}", arg_file.display())));
                }
            };
            vars_dict.global_mut().merge_dict(VarDict::from(dict));
        }
        let res = if let Some(out_var) = &self.out_var {
            let out_data_path = PathBuf::from(format!(
                "/tmp/gx_out_{out_var}_{}_{}",
                Local::now().format("%Y%m%d_%H%M%S"),
                rand::rng().random::<u32>()
            ));

            if out_data_path.exists() {
                std::fs::remove_file(&out_data_path).source_err(
                    ExecReason::Io,
                    format!("remove shell output file: {}", out_data_path.display()),
                )?;
            }
            std::fs::create_dir_all(out_data_path.parent().unwrap()).source_err(
                ExecReason::Io,
                format!(
                    "create shell output parent: {}",
                    out_data_path.parent().unwrap().display()
                ),
            )?;
            std::fs::File::create(&out_data_path).source_err(
                ExecReason::Io,
                format!("create shell output file: {}", out_data_path.display()),
            )?;
            // 修改命令以将输出写入 FIFO

            vars_dict
                .global_mut()
                .set(out_var, format!("{}", out_data_path.display()));
            //let exe_cmd = format!("{ext_cmd}");
            let res = gxl_sh!(
                LogicScope::Outer,
                ctx.tag_path("cmd").as_str(),
                &ext_cmd,
                &shell_opt,
                &exp,
                vars_dict.global()
            );
            let file_out = std::fs::read_to_string(&out_data_path).source_err(
                ExecReason::Io,
                format!("read shell output file: {}", out_data_path.display()),
            )?;
            vars_dict
                .global_mut()
                .set(out_var.as_str(), file_out.trim());
            std::fs::remove_file(&out_data_path).source_err(
                ExecReason::Io,
                format!("remove shell output file: {}", out_data_path.display()),
            )?;
            res
        } else {
            gxl_sh!(
                LogicScope::Outer,
                ctx.tag_path("cmd").as_str(),
                &ext_cmd,
                &shell_opt,
                &exp,
                vars_dict.global()
            )
        };

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

    use super::*;
    use crate::{
        ability::*,
        traits::{Getter, Setter},
        util::{OptionFrom, path::WorkDirWithLock},
    };
    use std::path::{Path, PathBuf};

    fn shell_quote(value: &Path) -> String {
        format!("'{}'", value.to_string_lossy().replace('\'', "'\\''"))
    }

    #[tokio::test]
    async fn shell_args_json() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _workdir = WorkDirWithLock::change(&manifest_dir).expect("set manifest dir");
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");
        let demo_sh = manifest_dir.join("tests/material/gx_shell/demo.sh");
        let env_args = manifest_dir.join("tests/material/gx_shell/env_args.json");
        let res = GxShell::new(format!("{} sys app", shell_quote(&demo_sh)))
            .with_out_var("OUT_FILE".to_opt())
            .with_arg_file(env_args.to_string_lossy().into_owned().to_opt());

        let TaskValue { vars, .. } = res.async_exec(context, def).await.assert("dryrun");
        assert_eq!(
            vars.global().get_copy("OUT_FILE").map(|x| x.to_string()),
            Some("DATA\ngalaxy".to_string())
        )
    }

    #[tokio::test]
    async fn shell_args_yml() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _workdir = WorkDirWithLock::change(&manifest_dir).expect("set manifest dir");
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");
        let demo_sh = manifest_dir.join("tests/material/gx_shell/demo.sh");
        let env_args = manifest_dir.join("tests/material/gx_shell/env_args.yml");
        let res = GxShell::new(format!("{} sys app", shell_quote(&demo_sh)))
            .with_out_var("OUT_FILE".to_opt())
            .with_arg_file(env_args.to_string_lossy().into_owned().to_opt());

        let TaskValue { vars, .. } = res.async_exec(context, def).await.assert("dryrun");
        assert_eq!(
            vars.global().get_copy("OUT_FILE").map(|x| x.to_string()),
            Some("DATA\ngalaxy".to_string())
        )
    }
}
