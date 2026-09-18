use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use orion_error::conversion::{ErrorWith, SourceErr};

use crate::{ExecReason, ExecResult, traits::Setter, var::VarDict};

use crate::const_val::gxl_const;

pub fn setup_start_vars(vars_dict: &mut VarDict) -> ExecResult<()> {
    vars_dict.set(gxl_const::OS_SYS, format_os_sys().as_str());

    let start_root = current_dir().source_err(ExecReason::system_error(), "get current dir")?;
    vars_dict.set(gxl_const::START_ROOT, start_root.display().to_string());
    let prj_root_opt = find_project_define();
    let prj_root = prj_root_opt.clone().unwrap_or(PathBuf::from("UNDEFIN"));
    vars_dict.set(gxl_const::PRJ_ROOT, format!("{}", prj_root.display()));
    let branch_probe_root = prj_root_opt.as_deref().unwrap_or(start_root.as_path());
    let git_branch = detect_git_branch(branch_probe_root).unwrap_or_else(|| "UNDEFIN".to_string());
    vars_dict.set(gxl_const::GIT_BRANCH, git_branch);
    Ok(())
}

pub fn setup_gxlrun_vars(vars_dict: &mut VarDict) -> ExecResult<()> {
    let start_root = current_dir()
        .source_err(ExecReason::system_error(), "source error")
        .doing("get current dir")?;
    vars_dict.set(gxl_const::CUR_DIR, start_root.display().to_string());
    Ok(())
}

fn get_os_info() -> (String, String, u64) {
    let info = os_info::get();
    let os_type = match info.os_type() {
        os_info::Type::Macos => "macos".to_string(),
        _ => info.os_type().to_string().to_lowercase(),
    };

    let arch = info.architecture().unwrap_or("unknown").to_string();
    let ver_major = match info.version() {
        os_info::Version::Semantic(major, _, _) => *major,
        _ => 0,
    };

    (arch, os_type, ver_major)
}

fn format_os_sys() -> String {
    let (arch, os_type, ver_major) = get_os_info();
    format!("{arch}_{os_type }_{ver_major}",)
}

/*
pub fn load_secfile(vars_dict: &mut VarDict) -> ExecResult<()> {
    let env_path = std::env::var("GAL_SEC_FILE_PATH").map(PathBuf::from);
    let default = sec_value_default_path();
    let path = env_path.unwrap_or(default);
    if path.exists() {
        let dict = ValueDict::from_conf(&path).source_err(UvsReason::logic_error().into(), "source error")?;
        info!(target: "exec","  load {}", path.display());
        for (k, v) in dict.iter() {
            vars_dict.set(format!("SEC_{}", k.to_uppercase()), {
                let value = v.clone();
                match value {
                    ValueType::String(v) => SecValueType::sec_from(v),
                    ValueType::Bool(v) => SecValueType::sec_from(v),
                    ValueType::Number(v) => SecValueType::sec_from(v),
                    ValueType::Float(v) => SecValueType::sec_from(v),
                    ValueType::Ip(v) => SecValueType::sec_from(v),
                    ValueType::Obj(v) => SecValueType::sec_from(v),
                    ValueType::List(v) => SecValueType::sec_from(v),
                }
            });
        }
    } else {
        let mut default = ValueDict::new();
        default.insert("example_key1", ValueType::from("value"));
        let dot_path = galaxy_dot_path();
        if !dot_path.exists() {
            std::fs::create_dir_all(dot_path).source_err(UvsReason::resource_error().into(), "source error")?;
        }
        default.save_conf(&path).source_err(UvsReason::resource_error().into(), "source error")?;
    }
    Ok(())
}
*/

/// 从当前目录开始向上查找 _gal/project.toml 文件
/// 如果找到则返回其绝对路径的PathBuf，未找到则返回None
pub fn find_project_define() -> Option<PathBuf> {
    find_gal_file("project.toml")
}
pub fn find_gal_file(file_name: &str) -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

    loop {
        let project_file = current_dir.join("_gal").join(file_name);
        if project_file.exists() {
            //let project_root = current_dir.clone();
            return Some(current_dir);
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // 已到达根目录
        }
    }

    None
}

pub fn detect_git_branch(start_at: &Path) -> Option<String> {
    let repo = git2::Repository::discover(start_at).ok()?;
    if let Ok(head) = repo.head() {
        if head.is_branch() {
            return head
                .shorthand()
                .ok()
                .and_then(non_empty)
                .map(|v| v.to_string());
        }
        // Detached HEAD is not a branch name by design.
        return None;
    }
    if let Ok(head_ref) = repo.find_reference("HEAD")
        && let Ok(Some(sym)) = head_ref.symbolic_target()
        && let Some(branch) = sym.strip_prefix("refs/heads/")
    {
        return non_empty(branch).map(|v| v.to_string());
    }
    None
}

fn non_empty(input: &str) -> Option<&str> {
    if input.trim().is_empty() {
        None
    } else {
        Some(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        execution::global::{detect_git_branch, find_project_define},
        util::path::WorkDir,
    };
    use tempfile::TempDir;

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_in_current_dir() {
        // 创建临时目录
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let gal_dir = temp_dir.path().join("_gal");
        std::fs::create_dir(&gal_dir).expect("Failed to create _gal dir");
        let project_file = gal_dir.join("project.toml");
        std::fs::write(&project_file, "").expect("Failed to create project.toml");

        // 设置当前工作目录为临时目录
        //env::set_current_dir(temp_dir.path()).expect("Failed to set current dir");
        let _wd = WorkDir::change(temp_dir.path());

        // 调用函数并断言结果
        assert!(find_project_define().is_some())
    }

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_in_parent_dir() {
        // 创建临时目录结构: temp_dir/child/_gal/project.toml
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let child_dir = temp_dir.path().join("child");
        std::fs::create_dir(&child_dir).expect("Failed to create child dir");
        let gal_dir = temp_dir.path().join("_gal");
        std::fs::create_dir(&gal_dir).expect("Failed to create _gal dir");
        let project_file = gal_dir.join("project.toml");
        std::fs::write(&project_file, "").expect("Failed to create project.toml");

        // 设置当前工作目录为child_dir
        let _wd = WorkDir::change(&child_dir);
        //env::set_current_dir(&child_dir).expect("Failed to set current dir");

        // 调用函数应找到父目录中的文件
        assert!(find_project_define().is_some());
    }

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_not_found() {
        // 创建临时目录，不创建_gal/project.toml
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let _wd = WorkDir::change(temp_dir.path());
        // 设置当前工作目录为临时目录
        //env::set_current_dir(temp_dir.path()).expect("Failed to set current dir");

        // 调用函数应返回None
        assert_eq!(find_project_define(), None);
    }

    #[test]
    fn test_detect_git_branch_not_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        assert_eq!(detect_git_branch(temp_dir.path()), None);
    }

    #[test]
    fn test_detect_git_branch_in_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo = git2::Repository::init(temp_dir.path()).expect("init git repo");
        let branch = detect_git_branch(temp_dir.path());
        let expected = repo
            .find_reference("HEAD")
            .expect("HEAD ref")
            .symbolic_target()
            .expect("read symbolic target")
            .expect("HEAD is symbolic")
            .strip_prefix("refs/heads/")
            .expect("heads prefix")
            .to_string();
        assert_eq!(branch, Some(expected));
    }

    #[test]
    fn test_detect_git_branch_detached_head_returns_none() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo = git2::Repository::init(temp_dir.path()).expect("init git repo");
        let sig = git2::Signature::now("tester", "tester@example.com").expect("signature");
        let tree_id = {
            let mut idx = repo.index().expect("index");
            idx.write_tree().expect("write tree")
        };
        let tree = repo.find_tree(tree_id).expect("tree");
        let commit_id = repo
            .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("commit");
        repo.set_head_detached(commit_id)
            .expect("set detached HEAD");

        assert_eq!(detect_git_branch(temp_dir.path()), None);
    }
}
