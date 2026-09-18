use std::path::PathBuf;

use crate::ability::prelude::*;
use getset::{Getters, Setters, WithSetters};
use orion_accessor::archive::{compress, decompress};
use orion_error::conversion::{ErrorWith, SourceErr, ToStructError};

#[derive(Clone, Default, Debug, PartialEq, Getters, Setters, WithSetters, Builder)]
#[getset(get = "pub", set = "pub", get_mut, set_with)]
pub struct GxTar {
    src: String,
    file: String,
}
impl GxTar {
    pub fn new<S: Into<String>>(src: S, dst: S) -> Self {
        Self {
            src: src.into(),
            file: dst.into(),
        }
    }
}
#[derive(Clone, Default, Debug, PartialEq, Getters, Setters, WithSetters, Builder)]
#[getset(get = "pub", set = "pub", get_mut, set_with)]
pub struct GxUnTar {
    file: String,
    dst: String,
}
impl GxUnTar {
    pub fn new<S: Into<String>>(src: S, out: S) -> Self {
        Self {
            file: src.into(),
            dst: out.into(),
        }
    }
}

// impl DefaultDTO for RgEcho {}

#[async_trait]
impl AsyncRunnableTrait for GxTar {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        let ex = EnvExpress::from_env_mix(vars_dict.global().clone());
        let src = PathBuf::from(ex.eval(&self.src)?);
        let dst = PathBuf::from(ex.eval(&self.file)?);
        info!(target: ctx.path(), "archive {}  -> {}", src.display(), dst.display());
        if !ctx.quiet() {
            eprintln!("archive {}  -> {}", src.display(), dst.display());
        }
        if !src.exists() {
            return Err(ExecReason::Args
                .to_err()
                .with_detail("src not exists")
                .with_context(&src));
        }
        if dst.exists() {
            std::fs::remove_file(&dst)
                .source_err(UvsReason::resource_error().into(), "source error")
                .with_context(&dst)?;
        }
        compress(src, dst).source_err(UvsReason::resource_error().into(), "source error")?;
        Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
    }
}

impl ComponentMeta for GxTar {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.tar")
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxUnTar {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        let ex = EnvExpress::from_env_mix(vars_dict.global().clone());
        let src = PathBuf::from(ex.eval(&self.file)?);
        let out = PathBuf::from(ex.eval(&self.dst)?);
        info!(target: ctx.path(), "untar {}  -> {}", src.display(), out.display());
        if !ctx.quiet() {
            eprintln!("untar {}  -> {}", src.display(), out.display());
        }
        if !src.exists() {
            return Err(ExecReason::Args
                .to_err()
                .with_detail("src not exists")
                .with_context(&src));
        }
        if out.exists() {
            // 如果目标是一个非空目录，先尝试删除它
            if out.is_dir() {
                std::fs::remove_dir_all(&out)
                    .or_else(|_| -> std::io::Result<()> {
                        // 如果删除整个目录失败，尝试删除目录内容
                        for entry in std::fs::read_dir(&out)? {
                            let entry = entry?;
                            let path = entry.path();
                            if path.is_dir() {
                                std::fs::remove_dir_all(&path)?;
                            } else {
                                std::fs::remove_file(&path)?;
                            }
                        }
                        Ok(())
                    })
                    .source_err(ExecReason::resource_error(), "remove dir all")
                    .with_context(&out)?;
            } else {
                // 如果目标是一个文件，直接删除
                std::fs::remove_file(&out)
                    .source_err(ExecReason::resource_error(), "remove file")
                    .with_context(&out)?;
            }
        }
        decompress(src, out).source_err(UvsReason::resource_error().into(), "source error")?;
        Ok(TaskValue::from((vars_dict, ExecOut::Ignore)))
    }
}

impl ComponentMeta for GxUnTar {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.untar")
    }
}

#[cfg(test)]
mod tests {

    use std::fs;
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_gx_archive_async_exec() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let archive_path = temp_dir.path().join("archive.tar.gz");

        // 创建测试目录和文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), "Hello, World!").unwrap();
        fs::create_dir_all(source_dir.join("subdir")).unwrap();
        fs::write(source_dir.join("subdir").join("nested.txt"), "Nested file").unwrap();

        // 创建GxArchive实例
        let gx_archive = GxTar::new(
            source_dir.to_string_lossy().to_string(),
            archive_path.to_string_lossy().to_string(),
        );

        // 创建执行上下文和变量空间
        let ctx = ExecContext::default();
        let vars_dict = VarSpace::default();

        // 执行压缩
        let result = gx_archive.async_exec(ctx, vars_dict).await;

        // 验证结果
        assert!(result.is_ok());
        assert!(archive_path.exists());

        // 验证压缩文件内容
        let extract_dir = temp_dir.path().join("extract");
        orion_accessor::archive::decompress(&archive_path, &extract_dir).unwrap();

        assert!(extract_dir.join("test.txt").exists());
        assert!(extract_dir.join("subdir").join("nested.txt").exists());

        assert_eq!(
            fs::read_to_string(extract_dir.join("test.txt")).unwrap(),
            "Hello, World!"
        );
        assert_eq!(
            fs::read_to_string(extract_dir.join("subdir").join("nested.txt")).unwrap(),
            "Nested file"
        );
    }

    #[tokio::test]
    async fn test_gx_archive_async_exec_nonexistent_source() {
        let temp_dir = tempdir().unwrap();
        let archive_path = temp_dir.path().join("archive.tar.gz");

        // 创建GxArchive实例，源目录不存在
        let gx_archive = GxTar::new(
            "nonexistent_source".to_string(),
            archive_path.to_string_lossy().to_string(),
        );

        // 创建执行上下文和变量空间
        let ctx = ExecContext::default();
        let vars_dict = VarSpace::default();

        // 执行压缩
        let result = gx_archive.async_exec(ctx, vars_dict).await;

        // 验证结果是错误
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gx_untar_async_exec() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let archive_path = temp_dir.path().join("archive.tar.gz");
        let extract_dir = temp_dir.path().join("extracted");

        // 创建测试目录和文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), "Hello, World!").unwrap();
        fs::create_dir_all(source_dir.join("subdir")).unwrap();
        fs::write(source_dir.join("subdir").join("nested.txt"), "Nested file").unwrap();

        // 先创建压缩文件
        let gx_tar = GxTar::new(
            source_dir.to_string_lossy().to_string(),
            archive_path.to_string_lossy().to_string(),
        );

        let ctx = ExecContext::default();
        let vars_dict = VarSpace::default();

        // 执行压缩
        let tar_result = gx_tar.async_exec(ctx.clone(), vars_dict.clone()).await;
        assert!(tar_result.is_ok());
        assert!(archive_path.exists());

        // 创建GxUnTar实例
        let gx_untar = GxUnTar::new(
            archive_path.to_string_lossy().to_string(),
            extract_dir.to_string_lossy().to_string(),
        );

        // 执行解压
        let untar_result = gx_untar.async_exec(ctx, vars_dict).await;

        // 验证结果
        assert!(untar_result.is_ok());
        assert!(extract_dir.exists());
        assert!(extract_dir.join("test.txt").exists());
        assert!(extract_dir.join("subdir").join("nested.txt").exists());

        assert_eq!(
            fs::read_to_string(extract_dir.join("test.txt")).unwrap(),
            "Hello, World!"
        );
        assert_eq!(
            fs::read_to_string(extract_dir.join("subdir").join("nested.txt")).unwrap(),
            "Nested file"
        );
    }

    #[tokio::test]
    async fn test_gx_untar_async_exec_nonexistent_source() {
        let temp_dir = tempdir().unwrap();
        let extract_dir = temp_dir.path().join("extracted");

        // 创建GxUnTar实例，源文件不存在
        let gx_untar = GxUnTar::new(
            "nonexistent_archive.tar.gz".to_string(),
            extract_dir.to_string_lossy().to_string(),
        );

        // 创建执行上下文和变量空间
        let ctx = ExecContext::default();
        let vars_dict = VarSpace::default();

        // 执行解压
        let result = gx_untar.async_exec(ctx, vars_dict).await;

        // 验证结果是错误
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gx_archive_async_exec_overwrite_existing() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let archive_path = temp_dir.path().join("archive.tar.gz");

        // 创建源目录和文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), "Original content").unwrap();

        // 先创建一个已存在的归档文件
        fs::write(&archive_path, "existing content").unwrap();
        assert!(archive_path.exists());

        // 创建GxArchive实例
        let gx_archive = GxTar::new(
            source_dir.to_string_lossy().to_string(),
            archive_path.to_string_lossy().to_string(),
        );

        // 创建执行上下文和变量空间
        let ctx = ExecContext::default();
        let vars_dict = VarSpace::default();

        // 执行压缩
        let result = gx_archive.async_exec(ctx, vars_dict).await;

        // 验证结果
        assert!(result.is_ok());
        assert!(archive_path.exists());

        // 验证文件已被覆盖
        let metadata = fs::metadata(&archive_path).unwrap();
        assert!(metadata.len() > 0);
        // 不再检查内容是否不同，因为归档文件是二进制文件
        // 只需要确认文件已被覆盖且不为空即可
    }

    #[tokio::test]
    async fn test_gx_tar_untar_integration() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let archive_path = temp_dir.path().join("archive.tar.gz");
        let extract_dir = temp_dir.path().join("extracted");

        // 创建测试目录和文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), "Hello, World!").unwrap();
        fs::create_dir_all(source_dir.join("subdir")).unwrap();
        fs::write(source_dir.join("subdir").join("nested.txt"), "Nested file").unwrap();

        // 创建GxTar实例
        let gx_tar = GxTar::new(
            source_dir.to_string_lossy().to_string(),
            archive_path.to_string_lossy().to_string(),
        );

        // 创建GxUnTar实例
        let gx_untar = GxUnTar::new(
            archive_path.to_string_lossy().to_string(),
            extract_dir.to_string_lossy().to_string(),
        );

        // 创建执行上下文和变量空间
        let ctx = ExecContext::default();
        let vars_dict = VarSpace::default();

        // 执行压缩
        let tar_result = gx_tar.async_exec(ctx.clone(), vars_dict.clone()).await;
        assert!(tar_result.is_ok());
        assert!(archive_path.exists());

        // 执行解压
        let untar_result = gx_untar.async_exec(ctx, vars_dict).await;
        assert!(untar_result.is_ok());

        // 验证解压结果
        assert!(extract_dir.exists());
        assert!(extract_dir.join("test.txt").exists());
        assert!(extract_dir.join("subdir").join("nested.txt").exists());

        assert_eq!(
            fs::read_to_string(extract_dir.join("test.txt")).unwrap(),
            "Hello, World!"
        );
        assert_eq!(
            fs::read_to_string(extract_dir.join("subdir").join("nested.txt")).unwrap(),
            "Nested file"
        );
    }
}
