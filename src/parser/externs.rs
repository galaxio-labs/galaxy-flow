use super::prelude::*;
use crate::ExecReason;
use crate::ExecResult;
use crate::components::gxl_extend::ModAddr;
use crate::evaluator::EnvExpress;
use crate::execution::VarSpace;
use crate::parser::abilities::addr::gal_extern_mod;
use crate::parser::abilities::addr::gal_git_path;
use orion_accessor::addr::GitRepository;
use orion_accessor::types::UpdateUnit;
use orion_accessor::update::DownloadOptions;
use orion_error::conversion::{ErrorWith, SourceErr, ToStructError};
use orion_error::runtime::WithContext;
use orion_variate::vars::EnvDict;
use orion_variate::vars::EnvEvalable;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use winnow::ascii::line_ending;
use winnow::ascii::till_line_ending;

use crate::util::GitTools;
use winnow::stream::Stream;

#[derive(Debug)]
pub enum DslStatus {
    Extern,
    Code,
    Data,
    End,
}

#[derive(Default, Builder, Getters)]
struct ExternLocal {
    path: PathBuf,
}

#[derive(Default, Builder)]
pub struct ExternGit {}

impl ExternGit {
    pub async fn pull(addr: GitRepository, up_options: &DownloadOptions) -> ExecResult<UpdateUnit> {
        GitTools::new(false)?.update_mod(addr, up_options).await
    }
}

impl ExternLocal {
    pub fn fetch_code(&self, name: &str) -> ExecResult<String> {
        let mut ctx = WithContext::doing("load code");
        let ee = EnvExpress::from_env();
        let gxl_full_path = format!("{}/{}.gxl", self.path.display(), name);
        let gxl_full_path = crate::evaluator::VarParser::eval(&ee, &gxl_full_path)?;
        ctx.record("gxl", gxl_full_path.as_str());
        let code = read_to_string(gxl_full_path.as_str())
            .source_err(ExecReason::Gxl, "read mod file fail!")
            .with_context(&ctx)?;
        Ok(code)
    }
}

pub struct ExternParser {}
impl Default for ExternParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternParser {
    pub fn new() -> Self {
        ExternParser {}
    }
    pub fn parse_code(input: &mut &str) -> Result<(String, DslStatus)> {
        let mut out = String::new();

        loop {
            if input.is_empty() {
                break;
            }
            let ck = input.checkpoint();
            if starts_with((multispace0, "extern"), input) {
                input.reset(&ck);
                return Ok((out, DslStatus::Extern));
            }

            // 解析当前行（包括空字符）
            let line = till_line_ending.parse_next(input)?;
            out.push_str(line);

            // 如果输入未结束，跳过换行符
            if !input.is_empty() {
                let end = line_ending(input)?;
                out.push_str(end);
            }
        }

        Ok((out, DslStatus::End))
    }

    pub async fn parse_extend_mod(
        cur: &mut &str,
        options: &DownloadOptions,
        vars_space: &VarSpace,
        file_exist_path: Option<&Path>,
    ) -> ExecResult<(String, DslStatus)> {
        use crate::evaluator::VarParser;
        let extern_mods = gal_extern_mod
            .context(wn_desc("<extern-mod>"))
            .parse_next(cur)
            .map_err(|err| {
                ExecReason::Gxl
                    .to_err()
                    .with_detail(format!("parse extern mod fail!: {err}"))
            })?;
        let exp = EnvExpress::from_env_mix(vars_space.global().clone());
        let local = match extern_mods.addr() {
            ModAddr::Git(git_addr) => {
                let git_url = exp.eval(git_addr.remote())?;
                let cl_git_url = git_url.clone();
                let (_host, repo_name) = gal_git_path(&mut git_url.as_str()).map_err(|err| {
                    ExecReason::Gxl
                        .to_err()
                        .with_detail(format!("parse git repo fail!: {err}"))
                })?;

                debug!("git url: {cl_git_url}");
                debug!("git repo : {repo_name}",);

                let addr = GitRepository::from(git_addr.remote())
                    .with_opt_branch(git_addr.branch().clone())
                    .with_opt_tag(git_addr.tag().clone());
                let addr = addr.env_eval(&EnvDict::from(vars_space.global().export().clone()));
                let local_path = ExternGit::pull(addr, options).await?;
                ExternLocalBuilder::default()
                    .path(local_path.position().join("mods"))
                    .build()
                    .unwrap()
            }
            ModAddr::Loc(loc_addr) => {
                let local_path = if let Some(file_exist_path) = file_exist_path {
                    loc_addr
                        .path()
                        .replace("@{PATH}", file_exist_path.display().to_string().as_str())
                } else {
                    loc_addr.path().clone()
                };
                ExternLocalBuilder::default()
                    .path(PathBuf::from(exp.eval(local_path.as_str())?))
                    .build()
                    .unwrap()
            }
        };
        debug!("mod-local @PATH: {}", local.path().display());
        let mut out = String::new();
        for mod_name in extern_mods.mods() {
            let mut code = local.fetch_code(mod_name)?;
            code = code.replace("@{PATH}", local.path().display().to_string().as_str());
            code = code.replace("@PATH", local.path().display().to_string().as_str());
            out += code.as_str();
        }
        Ok((out, DslStatus::Code))
    }
    pub async fn extern_parse(
        &self,
        options: &DownloadOptions,
        input: &mut &str,
        vars_space: &VarSpace,
        file_exist_path: Option<&Path>,
    ) -> ExecResult<(String, bool)> {
        let mut status = DslStatus::Code;
        let mut out = String::new();
        let mut have_extern = false;
        loop {
            if input.is_empty() {
                break;
            }
            match status {
                DslStatus::Code => {
                    let (code, cur_status) = Self::parse_code(input).map_err(|err| {
                        ExecReason::data_error()
                            .to_err()
                            .with_detail(err.to_string())
                    })?;
                    out += code.as_str();
                    status = cur_status;
                    continue;
                }
                DslStatus::Extern => {
                    let (code, cur_status) =
                        Self::parse_extend_mod(input, options, vars_space, file_exist_path).await?;
                    out += code.as_str();
                    status = cur_status;
                    have_extern = true;
                }
                DslStatus::Data => {
                    // DslStatus::Data 目前没有任何生产者，正常不会走到这里；
                    // 保留为错误而不是 panic，避免内部状态不一致直接崩掉进程。
                    return Err(ExecReason::Gxl
                        .to_err()
                        .with_detail("unexpected DslStatus::Data in extern_parse"));
                }
                DslStatus::End => break,
            }
        }
        Ok((out, have_extern))
    }
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::GxLoader;

    use super::*;
    #[tokio::test]
    async fn test_extern_one() {
        let up_opt = DownloadOptions::for_test();
        let parser = ExternParser::new();
        let vars = VarSpace::sys_init().assert();
        let mut data = r#"extern mod ssh { path = "./examples/moduse/mods";}"#;
        let (codes, _have_ext) = parser
            .extern_parse(&up_opt, &mut data, &vars, None)
            .await
            .assert();

        let mut expect = read_to_string("./examples/moduse/mods/ssh.gxl").unwrap();
        expect = expect.replace("@PATH", "./examples/moduse/mods");
        assert_eq!(codes, expect);
    }
    #[tokio::test]
    async fn test_extern_muti() {
        let vars = VarSpace::sys_init().assert();
        let dw_opt = DownloadOptions::for_test();
        let parser = ExternParser::new();
        let mut data = r#"extern mod os,ssh { path = "./examples/moduse/mods";}"#;
        let (codes, _have_ext) = parser
            .extern_parse(&dw_opt, &mut data, &vars, None)
            .await
            .assert();
        let mut expect = read_to_string("./examples/moduse/expect/_all.gxl").unwrap();
        expect = expect.replace("@PATH", "./examples/moduse/mods");
        println!("{codes}",);
        assert_eq!(codes, expect);
    }

    #[tokio::test]
    async fn test_extern_levels() {
        let loader = GxLoader::default();
        let vars = VarSpace::sys_init().assert();
        let codes = loader
            .parse_file("./src/parser/code/main.gxl", false, &vars)
            .await
            .assert();
        assert!(codes.get("mod_a").is_some());
        assert!(codes.get("mod_b").is_some());
        assert_eq!(codes.get("mod_c").unwrap().flows().len(), 2);
        assert!(codes.get("mod_d").is_none());
    }
}
