use orion_accessor::addr::Address;
use orion_accessor::addr::GitRepository;
use orion_accessor::types::ResourceDownloader;
use orion_accessor::types::UpdateUnit;
use orion_accessor::update::DownloadOptions;
use orion_error::conversion::SourceErr;
use orion_error::reason::UnifiedReason as UvsReason;
use orion_variate::vars::EnvDict;

use crate::ExecResult;
use crate::evaluator::EnvExpress;
use crate::evaluator::VarParser;
use std::path::PathBuf;

use super::accessor::build_accessor;
const RG_ROOT: &str = "${HOME}/.galaxy";
const VENDOR_ROOT: &str = "${HOME}/.galaxy/vendor";
#[derive(Default, Getters)]
pub struct GitTools {
    gxl_root: String,
    vendor_root: String,
    force: bool,
    exp_engine: EnvExpress,
}
impl GitTools {
    pub fn new(force: bool) -> ExecResult<Self> {
        let ee = EnvExpress::from_env();
        let rg_root = ee.eval(RG_ROOT)?;
        let vendor_root = ee.eval(VENDOR_ROOT)?;
        Ok(GitTools {
            force,
            gxl_root: rg_root,
            vendor_root,
            exp_engine: ee,
        })
    }
    pub async fn update_mod(
        &self,
        addr: GitRepository,
        options: &DownloadOptions,
    ) -> ExecResult<UpdateUnit> {
        build_accessor(&EnvDict::default())
            .download_to_local(
                &Address::from(addr),
                &PathBuf::from(self.vendor_root()),
                options,
            )
            .await
            .source_err(UvsReason::resource_error().into(), "source error")
    }
    pub fn vendor_path(&self, repo: &str, tag: &str) -> String {
        format!("{}/{repo}-{tag}/mods", self.vendor_root())
    }
}
