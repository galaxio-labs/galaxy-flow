use crate::components::gxl_spc::GxlSpace;
use crate::execution::VarSpace;
use crate::parser::abilities::ignore_comment;
use crate::parser::externs::ExternParser;
use crate::parser::stc_spc::WinnowErrorEx;
use crate::parser::stc_spc::gal_stc_spc;
use crate::util::accessor::build_accessor;

use std::fs;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

use crate::err::*;

use crate::ability::version::Version;
use once_cell::sync::OnceCell;
use orion_accessor::addr::Address;
use orion_accessor::addr::GitRepository;
use orion_accessor::addr::LocalPath;
use orion_accessor::types::ResourceDownloader;
use orion_accessor::update::DownloadOptions;
use orion_accessor::update::UpdateScope;
use orion_error::conversion::ErrorWith;
use orion_error::conversion::ToStructError;
use orion_error::conversion::{ConvErr, SourceErr};
use orion_error::reason::UnifiedReason as UvsReason;
use orion_error::runtime::WithContext;
use orion_variate::vars::EnvDict;
use orion_variate::vars::ValueDict;

static CODE_INSTANCE: OnceCell<String> = OnceCell::new();
pub fn get_parse_code() -> &'static str {
    if let Some(code) = CODE_INSTANCE.get() {
        return code;
    }
    ""
}

#[derive(Getters)]
pub struct GxLoader {
    gal_ver: Version,
}
impl Default for GxLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl GxLoader {
    pub fn new() -> GxLoader {
        GxLoader {
            gal_ver: Version::new(2, 0, 0, None),
        }
    }
    pub async fn parse_file(
        &self,
        conf: &str,
        update: bool,
        vars_space: &VarSpace,
    ) -> RunResult<GxlSpace> {
        info!(target:"parse", "parse file: {conf}" );
        let mut wc = WithContext::doing("parse gxl file");
        wc.record("conf", conf);
        let code = read_to_string(conf)
            .source_err(UvsReason::core_conf().into(), "source error")
            .with_context(&wc)?;
        let file_path = Path::new(conf);
        let file_exist_path = file_path.parent();
        self.parse_code(&code, update, vars_space, file_exist_path)
            .await
    }
    pub async fn parse_code(
        &self,
        code: &str,
        update: bool,
        vars_space: &VarSpace,
        file_exist_path: Option<&Path>,
    ) -> RunResult<GxlSpace> {
        let e_parser = ExternParser::new();

        let up_options = if update {
            DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default())
        } else {
            DownloadOptions::new(UpdateScope::None, ValueDict::default())
        };
        let mut target_code = code.to_string();

        loop {
            let mut target_code_str = target_code.as_str();
            let (code, have) = e_parser
                .extern_parse(
                    &up_options,
                    &mut target_code_str,
                    vars_space,
                    file_exist_path,
                )
                .await
                .with_context(("code", err_code_prompt(target_code_str)))
                .conv_err()?;

            target_code_str = code.as_str();
            target_code = ignore_comment(&mut target_code_str).map_err(|err| {
                RunReason::Gxl
                    .to_err()
                    .with_detail(format!("comment parse: {err}"))
                    .with_context(err_code_prompt(target_code_str))
            })?;
            if !have {
                break;
            }
        }
        info!(target: "parse","code len: {}", target_code.len());
        fs::write("./.run.gxl", target_code.as_str())
            .source_err(UvsReason::resource_error().into(), "source error")?;
        let mut code = target_code.as_str();
        let gxl_space = gal_stc_spc(&mut code)
            .map_err(WinnowErrorEx::from)
            .map_err(|err| {
                RunReason::Gxl
                    .to_err()
                    .with_detail(err.to_string())
                    .position(err_code_prompt(code))
                    .doing("parse ./.run.gxl file")
            })?;
        Ok(gxl_space)
    }

    pub async fn init_from_git(&self, addr: GitRepository) -> RunResult<()> {
        let init_path = PathBuf::from("./_gal");
        if init_path.exists() {
            return Err(RunReason::Args.to_err().with_detail(format!(
                "_gal already exists: path: {}",
                init_path.display()
            )));
        }

        let up_options = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());
        // Create _gal only after source validation
        std::fs::create_dir(&init_path)
            .source_err(UvsReason::resource_error().into(), "source error")?;

        let accessor = build_accessor(&EnvDict::default());
        let result = accessor
            .download_to_local(&Address::from(addr), &init_path, &up_options)
            .await;

        match result {
            Ok(downloaded) => {
                finalize_init_target(&init_path, downloaded.position())?;
                eprintln!("project initialized from git to ./_gal/");
                Ok(())
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&init_path);
                Err(RunReason::Exec
                    .to_err()
                    .with_detail(format!("copy to _gal failed: {}", e)))
            }
        }
    }

    pub async fn init_from_local(&self, src: &str) -> RunResult<()> {
        let src_path = PathBuf::from(src);
        if !src_path.exists() {
            return Err(RunReason::Args.to_err().with_detail(format!(
                "template path not found: path: {}",
                src_path.display()
            )));
        }

        let init_path = PathBuf::from("./_gal");
        if init_path.exists() {
            return Err(RunReason::Args.to_err().with_detail(format!(
                "_gal already exists: path: {}",
                init_path.display()
            )));
        }

        // Create _gal only after source validation
        std::fs::create_dir(&init_path)
            .source_err(UvsReason::resource_error().into(), "source error")?;

        let up_options = DownloadOptions::new(UpdateScope::None, ValueDict::default());
        let accessor = build_accessor(&EnvDict::default());
        let result = accessor
            .download_to_local(
                &Address::from(LocalPath::from(src)),
                &init_path,
                &up_options,
            )
            .await;

        match result {
            Ok(downloaded) => {
                finalize_init_target(&init_path, downloaded.position())?;
                eprintln!("project initialized from {} to ./_gal/", src);
                Ok(())
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&init_path);
                Err(RunReason::Exec
                    .to_err()
                    .with_detail(format!("copy to _gal failed: {}", e)))
            }
        }
    }
}

fn finalize_init_target(init_path: &Path, downloaded_path: &Path) -> RunResult<()> {
    if downloaded_path == init_path {
        return Ok(());
    }

    let init_canonical = std::fs::canonicalize(init_path)
        .source_err(UvsReason::resource_error().into(), "source error")?;
    let downloaded_canonical = std::fs::canonicalize(downloaded_path)
        .source_err(UvsReason::resource_error().into(), "source error")?;
    if !downloaded_canonical.starts_with(&init_canonical) {
        return Err(RunReason::Exec.to_err().with_detail(format!(
            "copy to _gal failed: downloaded path escaped init dir: init: {}, downloaded: {}",
            init_canonical.display(),
            downloaded_canonical.display()
        )));
    }

    if downloaded_path.is_file() {
        let name = downloaded_path.file_name().ok_or_else(|| {
            RunReason::Exec
                .to_err()
                .with_detail("copy to _gal failed: bad file name")
        })?;
        std::fs::rename(downloaded_path, init_path.join(name))
            .source_err(UvsReason::resource_error().into(), "source error")?;
        return Ok(());
    }

    for entry in std::fs::read_dir(downloaded_path)
        .source_err(UvsReason::resource_error().into(), "source error")?
    {
        let entry = entry.source_err(UvsReason::resource_error().into(), "source error")?;
        std::fs::rename(entry.path(), init_path.join(entry.file_name()))
            .source_err(UvsReason::resource_error().into(), "source error")?;
    }
    std::fs::remove_dir_all(downloaded_path)
        .source_err(UvsReason::resource_error().into(), "source error")?;
    Ok(())
}

pub fn err_code_prompt(code: &str) -> String {
    let take_len = if code.len() > 200 { 200 } else { code.len() };
    if let Some((left, _right)) = code.split_at_checked(take_len) {
        return format!("{left}...");
    }
    "".to_string()
}

#[cfg(test)]
mod tests {

    use crate::{
        cmd::GxlCmd, execution::VarSpace, infra::once_init_log, types::AnyResult,
        util::path::WorkDirWithLock,
    };
    use orion_error::dev::testing::TestAssert;

    use super::GxLoader;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_parse_file() -> AnyResult<()> {
        //log_init(&LogConf::alpha()).assert();
        once_init_log();
        let loader = GxLoader::default();
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _workdir = WorkDirWithLock::change(&manifest_dir)?;
        let conf = manifest_dir.join("_gal/work.gxl");
        let vars = VarSpace::sys_init().assert();
        let spc = loader
            .parse_file(conf.to_string_lossy().as_ref(), false, &vars)
            .await
            .assert()
            .assemble()
            .assert();
        info!("test begin");
        spc.show().assert();
        println!("mods:{}", spc.len());
        assert!(spc.len() > 1);
        spc.exec(
            GxlCmd::default()
                .with_env("default".into())
                .with_flows("conf".into()),
            VarSpace::sys_init().assert(),
            None,
        )
        .await
        .assert();
        Ok(())
    }

    #[tokio::test]
    async fn init_from_local_flattens_template_directory() -> AnyResult<()> {
        once_init_log();

        let loader = GxLoader::default();
        let temp_workdir = tempdir()?;
        let template_root = tempdir()?;
        let rust_tpl = template_root.path().join("rust");
        std::fs::create_dir_all(&rust_tpl)?;
        std::fs::write(rust_tpl.join("work.gxl"), "mod base {}")?;
        std::fs::write(rust_tpl.join("adm.gxl"), "mod base {}")?;
        std::fs::write(
            rust_tpl.join("project.toml"),
            "[project]\nname = \"demo\"\n",
        )?;

        let _workdir = WorkDirWithLock::change(temp_workdir.path())?;
        loader
            .init_from_local(rust_tpl.to_string_lossy().as_ref())
            .await
            .assert();

        assert!(temp_workdir.path().join("_gal/work.gxl").exists());
        assert!(temp_workdir.path().join("_gal/adm.gxl").exists());
        assert!(temp_workdir.path().join("_gal/project.toml").exists());
        assert!(!temp_workdir.path().join("_gal/rust").exists());
        Ok(())
    }

    #[test]
    fn finalize_init_target_rejects_paths_outside_gal_dir() -> AnyResult<()> {
        let temp_workdir = tempdir()?;
        let _workdir = WorkDirWithLock::change(temp_workdir.path())?;
        let init_path = temp_workdir.path().join("_gal");
        let external_path = temp_workdir.path().join("rust");

        std::fs::create_dir_all(&init_path)?;
        std::fs::create_dir_all(&external_path)?;
        std::fs::write(external_path.join("work.gxl"), "mod base {}")?;

        let err = super::finalize_init_target(&init_path, &external_path)
            .expect_err("external downloaded path should be rejected");
        let err_text = format!("{err:?}");

        assert!(err_text.contains("downloaded path escaped init dir"));
        assert!(external_path.exists());
        assert!(external_path.join("work.gxl").exists());
        assert!(!init_path.join("work.gxl").exists());
        Ok(())
    }
}
