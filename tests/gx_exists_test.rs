//! 集成测试：流程存在性判断（`gx run --exists` / GXL `gx.exists`）与 `gx.run(flow:)` 转发。
//!
//! 这些用真实 `gx` 二进制跑，断言**退出码**与 stdout，覆盖 `process::exit` 无法在进程内单测的路径。

use std::path::Path;
use std::process::{Command, Output};

use tempfile::tempdir;

fn gx_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gx")
}

/// 在 `dir` 下以 hermetic 的 `HOME` 运行 `gx`。
fn run(home: &Path, dir: &Path, args: &[&str]) -> Output {
    Command::new(gx_bin())
        .current_dir(dir)
        .env("HOME", home)
        .args(args)
        .output()
        .expect("gx should run")
}

fn write(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).expect("conf should be written");
    path
}

const SIMPLE_CONF: &str = r#"
mod envs { env default {} }
mod main {
  flow a {}
  flow b {}
}
"#;

#[test]
fn exists_probe_exit_codes() {
    let home = tempdir().unwrap();
    let dir = tempdir().unwrap();
    write(dir.path(), "work.gxl", SIMPLE_CONF);
    let (home, dir) = (home.path(), dir.path());

    // 存在 → 0，且不写 stdout
    let out = run(home, dir, &["run", "-c", "work.gxl", "a", "--exists"]);
    assert!(out.status.success(), "existing flow should exit 0");
    assert!(out.stdout.is_empty(), "probe should keep stdout clean");

    // 不存在 → 非 0 + stderr 说明
    let out = run(home, dir, &["run", "-c", "work.gxl", "nope", "--exists"]);
    assert!(!out.status.success(), "missing flow should exit non-zero");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("flow not found: nope"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // 多流程：全存在 → 0
    let out = run(home, dir, &["run", "-c", "work.gxl", "a", "b", "--exists"]);
    assert!(out.status.success(), "all-present should exit 0");

    // 多流程：任一不存在 → 非 0
    let out = run(
        home,
        dir,
        &["run", "-c", "work.gxl", "a", "nope", "--exists"],
    );
    assert!(
        !out.status.success(),
        "partially-missing should exit non-zero"
    );

    // 不给流程名 → 非 0 + 说明
    let out = run(home, dir, &["run", "-c", "work.gxl", "--exists"]);
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("no flow name given"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // conf 缺失 → 非 0 + 说明
    let out = run(
        home,
        dir,
        &["run", "-c", "does-not-exist.gxl", "a", "--exists"],
    );
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("conf not exists"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn exists_does_not_execute_the_flow() {
    let home = tempdir().unwrap();
    let dir = tempdir().unwrap();
    // `boom` 被执行会以退出码 3 失败；`--exists` 只探测不应执行它。
    write(
        dir.path(),
        "work.gxl",
        "mod envs { env default {} }\nmod main { flow boom { gx.cmd ( \"exit 3\" ) ; } }\n",
    );

    let out = run(
        home.path(),
        dir.path(),
        &["run", "-c", "work.gxl", "boom", "--exists"],
    );
    assert!(
        out.status.success(),
        "--exists must not execute the flow; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn adm_exists_probe_works_too() {
    let home = tempdir().unwrap();
    let dir = tempdir().unwrap();
    write(dir.path(), "adm.gxl", SIMPLE_CONF);

    let present = run(
        home.path(),
        dir.path(),
        &["adm", "-c", "adm.gxl", "a", "--exists"],
    );
    assert!(
        present.status.success(),
        "adm --exists should exit 0 for a present flow"
    );

    let missing = run(
        home.path(),
        dir.path(),
        &["adm", "-c", "adm.gxl", "nope", "--exists"],
    );
    assert!(
        !missing.status.success(),
        "adm --exists should exit non-zero for a missing flow"
    );
}

#[test]
fn gx_exists_condition_selects_branch() {
    let home = tempdir().unwrap();
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "work.gxl",
        r#"mod envs { env default {} }
mod main {
  flow a { gx.echo ( value : "A" ); }
  flow check {
    if gx.exists(flow: "a") { gx.echo ( value : "HAS_A" ); }
    if gx.exists(flow: "zzz") { gx.echo ( value : "HAS_ZZZ" ); }
    if !gx.exists(flow: "zzz") { gx.echo ( value : "NO_ZZZ" ); }
  }
}
"#,
    );

    let out = run(home.path(), dir.path(), &["run", "-c", "work.gxl", "check"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("HAS_A"), "stdout: {stdout}");
    assert!(stdout.contains("NO_ZZZ"), "stdout: {stdout}");
    assert!(
        !stdout.contains("HAS_ZZZ"),
        "missing flow must not take the true branch; stdout: {stdout}"
    );
}

#[test]
fn gx_run_forwards_multiple_flows() {
    let home = tempdir().unwrap();
    let dir = tempdir().unwrap();
    let conf_path = dir.path().join("work.gxl");
    let conf = format!(
        r#"mod envs {{ env default {{}} }}
mod main {{
  flow b {{ gx.echo ( value : "BBBB" ); }}
  flow c {{ gx.echo ( value : "CCCC" ); }}
  flow fwd {{ gx.run ( local : ".", env : "default", conf : "{}", flow : "b,c" ) ; }}
}}
"#,
        conf_path.display()
    );
    std::fs::write(&conf_path, conf).unwrap();

    let out = run(home.path(), dir.path(), &["run", "-c", "work.gxl", "fwd"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("BBBB") && stdout.contains("CCCC"),
        "both forwarded flows should run; stdout: {stdout}"
    );
}

#[test]
fn gx_exists_sees_only_the_sub_run_conf() {
    // 嵌套运行隔离：子运行只应看到子 conf 的流程，看不到外层 conf 的流程。
    let home = tempdir().unwrap();
    let dir = tempdir().unwrap();
    let sub_path = write(
        dir.path(),
        "sub.gxl",
        r#"mod envs { env default {} }
mod main {
  flow subonly {
    if gx.exists(flow: "probe") { gx.echo ( value : "SUB_SEES_OUTER" ); }
    if gx.exists(flow: "subonly") { gx.echo ( value : "SUB_SEES_SELF" ); }
  }
}
"#,
    );
    let outer = format!(
        r#"mod envs {{ env default {{}} }}
mod main {{
  flow probe {{
    if gx.exists(flow: "probe") {{ gx.echo ( value : "OUTER_SEES_SELF" ); }}
    gx.run ( local : ".", env : "default", conf : "{}", flow : "subonly" ) ;
  }}
}}
"#,
        sub_path.display()
    );
    write(dir.path(), "work.gxl", &outer);

    let out = run(home.path(), dir.path(), &["run", "-c", "work.gxl", "probe"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("OUTER_SEES_SELF"), "stdout: {stdout}");
    assert!(stdout.contains("SUB_SEES_SELF"), "stdout: {stdout}");
    assert!(
        !stdout.contains("SUB_SEES_OUTER"),
        "sub run must not see the outer conf's flows; stdout: {stdout}"
    );
}
