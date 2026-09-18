use std::fs;
use std::path::Path;

use orion_error::conversion::ToStructError;
use orion_error::conversion::{ErrorWith, SourceErr};
use orion_error::reason::UnifiedReason as UvsReason;

use crate::err::{RunReason, RunResult};

const SELF_UPDATE_BINARIES: &[&str] = &["gx"];

pub fn rollback(install_dir: &Path, backup_dir: &Path) -> RunResult<()> {
    for bin in SELF_UPDATE_BINARIES {
        let live_path = install_dir.join(bin_name(bin));
        let backup_path = backup_dir.join(bin_name(bin));
        if !backup_path.exists() {
            return Err(RunReason::Args.to_err().with_detail(format!(
                "backup is incomplete: backup_dir={}, missing={}",
                backup_dir.display(),
                backup_path.display()
            )));
        }
        copy_file(&backup_path, &live_path)?;
    }
    Ok(())
}

pub fn health_check(install_dir: &Path) -> RunResult<()> {
    for bin in SELF_UPDATE_BINARIES {
        exec_version(&install_dir.join(bin_name(bin)))?;
    }
    Ok(())
}

fn exec_version(bin: &Path) -> RunResult<()> {
    let status = std::process::Command::new(bin)
        .arg("--version")
        .status()
        .source_err(UvsReason::resource_error().into(), "source error")
        .doing("run binary version command")
        .with_context(("bin", bin))?;
    if status.success() {
        return Ok(());
    }
    Err(RunReason::Exec.to_err().with_detail(format!(
        "health check failed: {} --version exit={status}",
        bin.display()
    )))
}

fn copy_file(src: &Path, dst: &Path) -> RunResult<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .source_err(UvsReason::resource_error().into(), "source error")
            .doing("create copy destination parent")
            .with_context(("path", parent))?;
    }
    fs::copy(src, dst)
        .source_err(UvsReason::resource_error().into(), "source error")
        .doing("copy file")
        .with_context(("src", src))
        .with_context(("dst", dst))?;
    let perm = fs::metadata(src)
        .source_err(UvsReason::resource_error().into(), "source error")
        .doing("read source file permissions")
        .with_context(("src", src))?
        .permissions();
    fs::set_permissions(dst, perm)
        .source_err(UvsReason::resource_error().into(), "source error")
        .doing("set destination file permissions")
        .with_context(("src", src))
        .with_context(("dst", dst))?;
    Ok(())
}

fn bin_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
