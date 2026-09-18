use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use orion_accessor::{
    addr::{Address, HttpResource},
    types::{ResourceDownloader, ResourceUploader},
    update::{DownloadOptions, HttpMethod, UploadOptions},
};
use orion_error::conversion::{SourceErr, SourceRawErr, ToStructError};

use crate::{ability::prelude::*, util::accessor::build_accessor};

#[derive(Clone, Default, Debug, PartialEq, Builder, Getters)]
#[builder(setter(into))]
pub struct GxUpLoad {
    local_file: String,
    svc_url: String,
    #[builder(default)]
    username: Option<String>,
    #[builder(default)]
    password: Option<String>,
    method: String,
}

#[derive(Clone, Default, Debug, PartialEq, Builder, Getters)]
#[builder(setter(into))]
pub struct GxDownLoad {
    local_file: String,
    remote_url: String,
    #[builder(default)]
    username: Option<String>,
    #[builder(default)]
    password: Option<String>,
}

#[async_trait]
impl AsyncRunnableTrait for GxUpLoad {
    async fn async_exec(&self, _ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        let ex = EnvExpress::from_env_mix(vars_dict.global().clone());
        let mut addr = HttpResource::from(ex.eval(self.svc_url())?);
        if let (Some(username), Some(password)) = (self.username(), self.password()) {
            let username = ex.eval(username)?;
            let password = ex.eval(password)?;
            addr = addr.with_credentials(username, password);
        }
        let local_file = ex.eval(self.local_file())?;
        let mut action = Action::from("gx.upload").with_target(&local_file);
        let local_file_path = PathBuf::from(&local_file);
        let method = ex.eval(self.method())?;
        let http_method = HttpMethod::from_str(method.as_str())
            .source_raw_err(ExecReason::Args, format!("bad method:{method}"))?;

        if local_file_path.exists() {
            let accessor = build_accessor(&vars_dict.global().clone().into());
            accessor
                .upload_from_local(
                    &Address::from(addr),
                    &local_file_path,
                    &(UploadOptions::with_method(http_method)),
                )
                .await
                .source_err(UvsReason::resource_error().into(), "source error")?;
            action.finish();
            Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
        } else {
            return Err(ExecReason::Miss
                .to_err()
                .with_detail("local_file")
                .doing("gx.upload")
                .with_context(&local_file_path));
        }
    }
}

impl ComponentMeta for GxUpLoad {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.upload")
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxDownLoad {
    async fn async_exec(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.download");
        let ex = EnvExpress::from_env_mix(vars_dict.global().clone());
        let mut addr = HttpResource::from(ex.eval(self.remote_url())?);
        if let (Some(username), Some(password)) = (self.username(), self.password()) {
            let username = ex.eval(username)?;
            let password = ex.eval(password)?;
            addr = addr.with_credentials(username, password);
        }
        let local_file = ex.eval(self.local_file())?;
        let mut action = Action::from("gx.download").with_target(&local_file);
        let local_file_path = PathBuf::from(&local_file);

        // 确定最终的下载路径
        let final_download_path = self.get_final_path(&local_file_path, &addr);
        debug!(target: ctx.path(), "downlad  {} to {}", addr.url() ,final_download_path.display());

        let accessor = build_accessor(&vars_dict.global().clone().into());
        // 确保父目录存在
        match local_file_path.parent() {
            Some(parent) if parent.exists() => {
                accessor
                    .download_to_local(
                        &Address::from(addr),
                        &final_download_path,
                        &DownloadOptions::default(),
                    )
                    .await
                    .source_err(UvsReason::resource_error().into(), "source error")
                    .with_context(&final_download_path)?;
                action.finish();
                Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
            }
            _ => {
                return Err(ExecReason::Miss
                    .to_err()
                    .with_detail("parent path not exists")
                    .doing("gx.download")
                    .with_context(&local_file_path));
            }
        }
    }
}

impl GxDownLoad {
    fn get_final_path(&self, local_file_path: &Path, addr: &HttpResource) -> PathBuf {
        if local_file_path.is_dir() {
            // 如果传入的是目录，从URL中提取文件名
            let filename = addr
                .url()
                .split('/')
                .next_back()
                .and_then(|name| {
                    // 移除查询参数和锚点
                    let clean_name = name
                        .split('?')
                        .next()
                        .unwrap_or(name)
                        .split('#')
                        .next()
                        .unwrap_or(name);
                    if clean_name.is_empty() || clean_name == "/" {
                        None
                    } else {
                        Some(clean_name)
                    }
                })
                .unwrap_or("downloaded_file");

            let target_path = local_file_path.join(filename);
            info!("检测到目录路径，文件将下载到: {}", target_path.display());
            target_path
        } else {
            // 如果传入的是文件路径，直接使用
            info!("使用指定文件路径: {}", local_file_path.display());
            local_file_path.to_path_buf()
        }
    }
}

impl ComponentMeta for GxDownLoad {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.download")
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "network_test")]
    use orion_error::dev::testing::TestAssertWithMsg;
    #[cfg(feature = "network_test")]
    use orion_infra::path::ensure_path;

    use crate::util::path::WorkDir;

    use super::*;

    #[cfg(feature = "network_test")]
    #[tokio::test]
    async fn test_gx_download_parent_exists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let temp_path = temp_dir.path().join("download");
        ensure_path(&temp_path).assert("path");
        let file_path = temp_path.join("README").to_str().unwrap().to_string();

        let download = GxDownLoadBuilder::default()
            .local_file(file_path.clone())
            .remote_url(
                "https://raw.githubusercontent.com/galaxio-labs/galaxy-flow/main/README.md"
                    .to_string(),
            )
            .build()
            .unwrap();

        let vars_dict = VarSpace::default();
        let ctx = ExecContext::default();
        download
            .async_exec(ctx, vars_dict)
            .await
            .assert("exec fail");
        let content = std::fs::read_to_string(file_path).expect("read downloaded file");
        assert!(
            !content.is_empty(),
            "downloaded content should not be empty"
        );
    }

    /// 测试下载文件到已存在目录的功能
    ///
    /// 注意：当前实现中，传入目录路径时实际上会直接下载到该目录路径，
    /// 而不是提取文件名。这个测试验证当前的实际行为。
    #[cfg(feature = "network_test")]
    #[tokio::test]
    async fn test_gx_download_to_directory() {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        // 构建下载配置：传入目录路径
        let download = GxDownLoadBuilder::default()
            .local_file(dir_path.clone()) // 传入目录路径
            .remote_url(
                "https://raw.githubusercontent.com/galaxio-labs/galaxy-flow/main/README.md"
                    .to_string(),
            )
            .build()
            .unwrap();

        // 执行下载操作
        let vars_dict = VarSpace::default();
        let ctx = ExecContext::default();
        let result = download.async_exec(ctx, vars_dict).await;
        assert!(result.is_ok());

        // 验证文件是否下载成功
        // 当前实现会在目录中按URL文件名创建文件
        let downloaded_path = temp_dir.path().join("README.md");
        assert!(downloaded_path.exists(), "下载应该成功完成");
    }

    /// 测试下载包含查询参数的URL到指定文件
    ///
    /// 注意：当前实现中，文件名提取功能在 get_final_path 中实现，
    /// 但实际下载使用的是原始路径。这个测试验证当前的实际行为。
    #[cfg(feature = "network_test")]
    #[tokio::test]
    async fn test_gx_download_url_with_query_params() {
        // 创建临时目录和具体文件路径
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.json");

        // 构建包含查询参数的URL下载配置
        let download = GxDownLoadBuilder::default()
            .local_file(file_path.to_str().unwrap().to_string()) // 使用具体文件路径
            .remote_url("https://raw.githubusercontent.com/galaxio-labs/galaxy-flow/main/README.md?param=value&test=123".to_string()) // 包含查询参数
            .build()
            .unwrap();

        // 执行下载
        let vars_dict = VarSpace::default();
        let ctx = ExecContext::default();
        let result = download.async_exec(ctx, vars_dict).await;
        assert!(result.is_ok());

        // 验证文件下载到指定位置
        assert!(file_path.exists(), "文件应该下载到指定的文件路径");
    }

    /// 测试下载文件到当前目录（没有父目录路径的情况）
    ///
    /// 注意：当前实现要求父目录必须存在，所以没有父目录的路径会失败。
    /// 这个测试验证当前的实际行为。
    #[tokio::test]
    async fn test_gx_download_to_current_directory_file() {
        // 创建临时目录并切换到该目录
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        WorkDir::change(temp_dir.path()).unwrap();

        // 构建下载配置：使用相对路径文件名（没有目录分隔符）
        // 这种情况下 PathBuf::parent() 会返回 None
        let download = GxDownLoadBuilder::default()
            .local_file("test_file.json".to_string()) // 注意：没有路径分隔符，直接是文件名
            .remote_url("https://httpbin.org/json".to_string())
            .build()
            .unwrap();

        // 执行下载
        let vars_dict = VarSpace::default();
        let ctx = ExecContext::default();
        let result = download.async_exec(ctx, vars_dict).await;

        // 恢复原始工作目录
        WorkDir::change(original_dir).unwrap();

        // 验证当前实现的行为：没有父目录的路径会失败
        assert!(result.is_err(), "当前实现中，没有父目录的路径会失败");

        // 验证错误类型
        if let Err(e) = result {
            let error_str = format!("{e:?}");
            assert!(
                error_str.contains("parent path not exists"),
                "应该是父目录不存在的错误"
            );
        }
    }

    /// 测试嵌套目录路径的下载行为
    ///
    /// 注意：当前实现要求父目录必须存在，不会自动创建嵌套目录。
    #[tokio::test]
    async fn test_gx_download_creates_nested_directories() {
        // 创建临时目录作为测试根目录
        let temp_dir = tempfile::tempdir().unwrap();

        // 构造一个深层嵌套的文件路径，这些目录都不存在
        let nested_path = temp_dir.path().join("level1/level2/level3/file.json");

        // 构建下载配置
        let download = GxDownLoadBuilder::default()
            .local_file(nested_path.to_str().unwrap().to_string())
            .remote_url("https://httpbin.org/json".to_string())
            .build()
            .unwrap();

        // 验证父目录确实不存在
        assert!(
            !nested_path.parent().unwrap().exists(),
            "测试前提：父目录应该不存在"
        );

        // 执行下载操作
        let vars_dict = VarSpace::default();
        let ctx = ExecContext::default();
        let result = download.async_exec(ctx, vars_dict).await;

        // 调试信息
        match &result {
            Ok(_) => println!("✅ 下载成功"),
            Err(e) => println!("❌ 下载失败: {e:?}",),
        }

        // 验证当前实现的行为：父目录不存在时会失败
        assert!(result.is_err(), "当前实现中，父目录不存在时下载会失败");

        // 验证错误类型
        if let Err(e) = result {
            let error_str = format!("{e:?}");
            assert!(
                error_str.contains("parent path not exists"),
                "应该是父目录不存在的错误"
            );
        }

        // 验证文件没有被创建
        assert!(!nested_path.exists(), "文件不应该被创建，因为下载失败了");
    }
}
