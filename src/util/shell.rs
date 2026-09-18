use duct_sh;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::evaluator::{EnvExpress, VarParser};
use crate::expect::LogicScope;
use crate::expect::ShellOption;
use crate::var::VarDict;
use crate::{ExecReason, ExecResult};

use colored::*;
use orion_error::conversion::ToStructError;
#[allow(clippy::result_large_err)]
pub fn os_sh(
    scope: LogicScope,
    target: &str,
    cmd: &str,
    opt: &ShellOption,
    exp: &EnvExpress,
    env: &VarDict,
) -> ExecResult<(i32, Vec<u8>, Vec<u8>)> {
    let sec_cmd = exp.sec_eval(cmd)?;
    //let ee = EnvExpress::from_env();
    if !opt.secrecy {
        let lev = opt.log_lev.unwrap_or(log::Level::Debug);
        log!(target: target, lev, "cmd : {sec_cmd}", );
        if !opt.quiet(scope) {
            show_cmd(&sec_cmd);
        }
    }
    let exe_cmd = exp.eval(cmd)?;
    let mut run_env = env.clone();
    run_env.merge_dict(VarDict::from(std::env::vars()));
    let print_output = !opt.quiet(scope);
    let output = if opt.stream {
        stream_sh(&exe_cmd, print_output, run_env.export_str_map())
    } else {
        duct_sh::sh_dangerous(exe_cmd)
            .unchecked()
            .stdout_capture()
            .stderr_capture()
            .full_env(run_env.export_str_map())
            //.full_env(run_env.export())
            .run()
            .map(|out| CmdOutput {
                status: out.status,
                stdout: out.stdout,
                stderr: out.stderr,
            })
            .map_err(|err| StreamRunError::Duct(err.to_string()))
    };
    let fail_msg = opt.err.clone().unwrap_or(sec_cmd.clone());
    let fail_msg = exp.eval(fail_msg.as_str())?;
    match output {
        Err(e) => Err(ExecReason::OsCmd
            .to_err()
            .with_detail(format!("cmd={fail_msg}, code=254, stderr={e}"))),
        Ok(out) => {
            if let Some(code) = out.status.code() {
                let err_desp = "err msg from utf8 failed";
                let out_msg = String::from_utf8(out.stdout.clone()).map_err(|_| {
                    ExecReason::OsCmd
                        .to_err()
                        .with_detail(format!("cmd={sec_cmd}, code=253, stderr={err_desp}"))
                })?;
                let err_msg = String::from_utf8(out.stderr.clone()).map_err(|_| {
                    ExecReason::OsCmd
                        .to_err()
                        .with_detail(format!("cmd={sec_cmd}, code={code}, stderr={err_desp}"))
                })?;
                let is_ok = opt.ok_codes.contains(&code);
                if is_ok && let Some(ref suc_msg) = opt.suc {
                    println!("{suc_msg}");
                }

                let log_level = opt.log_lev.unwrap_or(log::Level::Debug);
                if !opt.stream && !opt.quiet(scope) {
                    if !out_msg.is_empty() {
                        println!("{out_msg}");
                        log!(target: target, log_level, "out:\n{out_msg}", );
                    }
                    if !err_msg.is_empty() {
                        if is_ok {
                            println!("{}", err_msg.yellow());
                            log!(target: target, log_level, "out:\n{err_msg}", );
                        } else {
                            println!("{}", err_msg.clone().red());
                            log!(target: target, log_level, "err:\n{err_msg}", );
                        }
                    }
                }
                return if is_ok {
                    Ok((code, out.stdout, out.stderr))
                } else {
                    Err(ExecReason::OsCmd
                        .to_err()
                        .with_detail(format!("cmd={fail_msg}, code={code}, stderr={err_msg}")))
                };
            }
            Err(ExecReason::OsCmd
                .to_err()
                .with_detail(format!("cmd={fail_msg}, code=252, stderr=no exit code")))
        }
    }
}

enum StreamRunError {
    Duct(String),
    Io(std::io::Error),
    Join(&'static str),
}

impl std::fmt::Display for StreamRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamRunError::Duct(err) => write!(f, "{err}"),
            StreamRunError::Io(err) => write!(f, "{err}"),
            StreamRunError::Join(name) => write!(f, "join {name} stream reader failed"),
        }
    }
}

impl From<std::io::Error> for StreamRunError {
    fn from(value: std::io::Error) -> Self {
        StreamRunError::Io(value)
    }
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn stream_sh(
    exe_cmd: &str,
    print_output: bool,
    env_map: impl IntoIterator<Item = (String, String)>,
) -> Result<CmdOutput, StreamRunError> {
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(exe_cmd)
        .envs(env_map)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("stdout pipe unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::other("stderr pipe unavailable"))?;

    let stdout_buf = Arc::new(Mutex::new(Vec::new()));
    let stderr_buf = Arc::new(Mutex::new(Vec::new()));

    let stdout_handle = spawn_stream_reader(stdout, stdout_buf.clone(), print_output, true);
    let stderr_handle = spawn_stream_reader(stderr, stderr_buf.clone(), print_output, false);

    let status = child.wait()?;

    stdout_handle
        .join()
        .map_err(|_| StreamRunError::Join("stdout"))??;
    stderr_handle
        .join()
        .map_err(|_| StreamRunError::Join("stderr"))??;

    let stdout = Arc::try_unwrap(stdout_buf)
        .map_err(|_| StreamRunError::Join("stdout unwrap"))?
        .into_inner()
        .map_err(|_| StreamRunError::Join("stdout lock"))?;
    let stderr = Arc::try_unwrap(stderr_buf)
        .map_err(|_| StreamRunError::Join("stderr unwrap"))?
        .into_inner()
        .map_err(|_| StreamRunError::Join("stderr lock"))?;

    Ok(CmdOutput {
        status,
        stdout,
        stderr,
    })
}

fn spawn_stream_reader<R: Read + Send + 'static>(
    mut reader: R,
    buffer: Arc<Mutex<Vec<u8>>>,
    print_output: bool,
    is_stdout: bool,
) -> thread::JoinHandle<Result<(), StreamRunError>> {
    thread::spawn(move || {
        let mut chunk = [0u8; 4096];
        loop {
            let size = reader.read(&mut chunk)?;
            if size == 0 {
                break;
            }
            let data = &chunk[..size];
            {
                let mut buf = buffer
                    .lock()
                    .map_err(|_| StreamRunError::Join("stream lock"))?;
                buf.extend_from_slice(data);
            }
            if print_output {
                if is_stdout {
                    let mut out = std::io::stdout().lock();
                    out.write_all(data)?;
                    out.flush()?;
                } else {
                    let mut err = std::io::stderr().lock();
                    err.write_all(data)?;
                    err.flush()?;
                }
            }
        }
        Ok(())
    })
}

fn show_cmd(sec_cmd: &String) {
    let mut cmd_elements = VecDeque::from(sec_cmd.split(' ').collect::<Vec<&str>>());
    let first_element = cmd_elements.pop_front();
    if let Some(first) = first_element {
        let cmd_first = std::path::Path::new(first);
        if cmd_first.exists() {
            let cmd_name = cmd_first.file_name().and_then(|x| x.to_str());
            if let Some(name) = cmd_name {
                let simple_cmd = Vec::from(cmd_elements).join(" ");
                println!(".../{name} {simple_cmd}",);
                return;
            }
        }
        println!("{sec_cmd}",);
    }
}

#[cfg(test)]
mod tests {
    use orion_variate::vars::ValueType;

    use crate::var::VarDict;

    use super::*;
    //use duct_sh::* ;

    #[test]
    fn duct_test() {
        cmd!("echo", "hi").run().unwrap();
        cmd!("/bin/sh", "./src/util/echo.sh").run().unwrap();
        duct_sh::sh("echo hi2").run().unwrap();
        duct_sh::sh("cd ./src ; ls ").run().unwrap();
    }
    #[test]
    fn rg_sh_test() {
        let mut dict = VarDict::global_new();
        dict.sec_set("SEC_KEY", ValueType::from("galaxy".to_string()));
        let exp = EnvExpress::from_env_mix(dict.clone());
        let opt = ShellOption {
            quiet: true,
            ..Default::default()
        };
        let cmd = "echo ${SEC_KEY}".to_string();
        let (_exit_code, stdout, _stderr) =
            os_sh(LogicScope::Outer, "gx.sh", &cmd, &opt, &exp, &dict).unwrap();
        assert_eq!(stdout, b"galaxy\n");
    }
}
