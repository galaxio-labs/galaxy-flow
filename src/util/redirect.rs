use serde::Serialize;

use crate::util::redirect::platform::StdoutRedirect;
use crate::{ExecError, ExecReason};
use orion_error::conversion::{SourceErr, SourceRawErr};
use std::fs::File;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::SystemTime;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod platform {
    use libc::{STDOUT_FILENO, close, dup, dup2, pipe, read, write};
    use std::fs::File;
    use std::io::Write;

    use std::path::Path;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::thread;
    use std::time::Duration;

    pub struct StdoutRedirect {
        thread_handle: Option<thread::JoinHandle<()>>,
        #[allow(dead_code)]
        stop_signal: Arc<AtomicBool>,
        original_stdout_fd: i32,
    }

    impl Drop for StdoutRedirect {
        fn drop(&mut self) {
            self.stop();
        }
    }

    impl StdoutRedirect {
        pub fn start(log_path: &Path) -> Option<Self> {
            let mut pipefd = [0; 2]; // 创建管道文件描述符数组

            if unsafe { pipe(pipefd.as_mut_ptr()) } == -1 {
                error!("pipe() failed");
                return None;
            }
            let read_fd = pipefd[0]; // 读取端
            let write_fd = pipefd[1]; // 写入端

            // 重定向前保存原始标准输出
            let original_stdout_fd = unsafe { dup(STDOUT_FILENO) };
            if original_stdout_fd == -1 {
                // 错误时关闭已打开的文件描述符
                unsafe {
                    close(read_fd);
                    close(write_fd);
                }
                error!("dup(STDOUT_FILENO) failed");
                return None;
            }

            // 将标准输出重定向到管道的写端
            if unsafe { dup2(write_fd, STDOUT_FILENO) } == -1 {
                unsafe {
                    // 错误时关闭已打开的文件描述符
                    close(read_fd);
                    close(write_fd);
                    close(original_stdout_fd);
                }
                error!("dup2() failed");
                return None;
            }
            unsafe { close(write_fd) }; // Close original write end

            // 创建日志文件
            let mut log_file = match File::options()
                .create(true)
                .append(true)
                .open(log_path.as_os_str())
            {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to open stdout.log: {e}");
                    unsafe {
                        // 错误时关闭已打开的文件描述符
                        close(read_fd);
                        close(original_stdout_fd);
                    }
                    return None;
                }
            };

            let stop_signal = Arc::new(AtomicBool::new(false));
            let thread_stop = Arc::clone(&stop_signal);

            // 创建原始标准输出的副本，以便在线程中安全使用
            let thread_stdout_fd = unsafe { dup(original_stdout_fd) };
            if thread_stdout_fd == -1 {
                error!("Failed to duplicate stdout FD for thread");
                unsafe {
                    // 错误时关闭已打开的文件描述符
                    close(read_fd);
                    close(original_stdout_fd);
                }
                return None;
            }

            let thread_handle = thread::spawn(move || {
                let mut buf = [0u8; 1024];
                loop {
                    if thread_stop.load(Ordering::Relaxed) {
                        // 检查是否需要停止线程
                        break;
                    }

                    let n =
                        unsafe { read(read_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };

                    // 通过短暂等待处理EAGAIN/EWOULDBLOCK
                    if n == -1 {
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    }

                    if n <= 0 {
                        // 正常文件结束或错误
                        break;
                    }

                    let n = n as usize;

                    //  写入原始标准输出
                    if unsafe { write(thread_stdout_fd, buf.as_ptr() as *const libc::c_void, n) }
                        == -1
                    {
                        error!("Failed to write to original stdout");
                    }
                    // 写入日志文件
                    if let Err(e) = log_file.write_all(&buf[..n]) {
                        error!("Failed to write to log file: {e}");
                    }
                }

                // 关闭文件描述符
                unsafe {
                    close(read_fd);
                    close(thread_stdout_fd);
                }
            });
            Some(Self {
                thread_handle: Some(thread_handle),
                stop_signal,
                original_stdout_fd,
            })
        }

        pub fn stop(&mut self) {
            // 恢复原始终端 stdout
            unsafe {
                dup2(self.original_stdout_fd, STDOUT_FILENO);
            }
            // 取出 `JoinHandle` 并等待线程完成
            if let Some(handle) = self.thread_handle.take() {
                handle.join().expect("Failed to join logger thread");
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::os::windows::io::AsRawHandle;
    use std::ptr;
    use winapi::shared::minwindef::{BOOL, DWORD};
    use winapi::um::{
        ioapiset::ReadFile,
        memoryapi::WriteFile,
        processthreadsapi::GetCurrentProcess,
        winbase::{CreatePipe, GetStdHandle, STD_OUTPUT_HANDLE, SetStdHandle},
        winnt::{DUPLICATE_SAME_ACCESS, HANDLE},
    };

    type HPIPE = HANDLE;

    /// 重定向 stdout 到管道，并返回读端和后台线程句柄
    pub fn redirect_stdout() -> (HPIPE, thread::JoinHandle<()>) {
        let mut h_read_pipe: HPIPE = ptr::null_mut();
        let mut h_write_pipe: HPIPE = ptr::null_mut();

        // 创建匿名管道
        let sa = std::mem::zeroed(); // 默认安全描述符
        let success = unsafe { CreatePipe(&mut h_read_pipe, &mut h_write_pipe, &sa, 0) };
        if success == 0 {
            panic!("CreatePipe failed: {}", io::Error::last_os_error());
        }

        // 获取原始 stdout 句柄
        let h_original_stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        if h_original_stdout == ptr::null_mut() {
            panic!("GetStdHandle(STD_OUTPUT_HANDLE) failed");
        }

        // 复制管道写端句柄到当前进程（避免被关闭影响）
        let mut h_dup_write_pipe: HPIPE = ptr::null_mut();
        let success = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                h_write_pipe,
                GetCurrentProcess(),
                &mut h_dup_write_pipe,
                0,
                TRUE, // 允许子进程继承句柄（此处不需要，但保留兼容性）
                DUPLICATE_SAME_ACCESS,
            )
        };
        if success == 0 {
            panic!("DuplicateHandle failed: {}", io::Error::last_os_error());
        }
        unsafe { CloseHandle(h_write_pipe) }; // 关闭原始写端

        // 重定向 stdout 到管道写端
        let success = unsafe { SetStdHandle(STD_OUTPUT_HANDLE, h_dup_write_pipe) };
        if success == 0 {
            panic!("SetStdHandle(STD_OUTPUT_HANDLE) failed");
        }

        // 启动后台线程读取管道并写入终端和文件
        let h_read_pipe_clone = h_read_pipe;
        let log_file = File::options()
            .create(true)
            .append(true)
            .open("stdout.log")
            .expect("Failed to open stdout.log");

        let handle = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                let mut bytes_read: DWORD = 0;
                // 从管道读端读取数据（阻塞直到有数据或出错）
                let success = unsafe {
                    ReadFile(
                        h_read_pipe_clone,
                        buf.as_mut_ptr() as *mut u8,
                        buf.len() as DWORD,
                        &mut bytes_read,
                        ptr::null_mut(),
                    )
                };
                if !success || bytes_read == 0 {
                    break; // 管道关闭或出错时退出循环
                }

                // 写入原始 stdout（终端）
                let success = unsafe {
                    WriteFile(
                        h_original_stdout,
                        buf.as_ptr() as *const u8,
                        bytes_read,
                        &mut bytes_read,
                        ptr::null_mut(),
                    )
                };
                if !success {
                    eprintln!(
                        "Failed to write to terminal: {}",
                        io::Error::last_os_error()
                    );
                }

                // 写入日志文件
                if let Err(e) = log_file.write_all(&buf[..bytes_read as usize]) {
                    eprintln!("Failed to write to log file: {}", e);
                }
            }
        });

        (h_read_pipe, handle) // 返回管道读端和线程句柄
    }
}

pub static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// 返回一个临时日志文件，格式为 ".galaxy/templog_<timestamp>.log"
/// 如果文件已存在，则返回该文件路径；如果不存在，则创建新文件
pub fn init_redirect_file() -> Result<PathBuf, ExecError> {
    if let Some(log_file) = LOG_PATH.get() {
        return Ok(log_file.clone());
    }
    let sys_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .source_raw_err(ExecReason::Io, "获取系统时间失败")?
        .as_secs() as i64;
    let log_file = std::env::temp_dir().join(format!("galaxy_templog_{sys_time}.log"));
    File::create(&log_file).source_err(
        ExecReason::Io,
        format!("创建临时日志文件: {}", log_file.display()),
    )?;
    Ok(LOG_PATH.get_or_init(|| log_file).clone())
}

pub fn stop_redirect(redirect: Option<StdoutRedirect>) -> Result<(), ExecError> {
    if let Some(mut redirect) = redirect {
        redirect.stop();
    }
    Ok(())
}

/// 封装日志文件操作：定位到文件末尾并返回位置
pub fn seek_log_file_end(log_file: &Path) -> Result<u64, ExecError> {
    let mut file = File::open(log_file).source_err(
        ExecReason::Io,
        format!("open log file: {}", log_file.display()),
    )?;
    file.seek(SeekFrom::End(0)).source_err(
        ExecReason::Io,
        format!("seek log file: {}", log_file.display()),
    )
}

/// 封装日志内容读取
pub async fn read_log_content(
    log_file: &Path,
    start_pos: u64,
    end_pos: u64,
) -> Result<String, ExecError> {
    let mut file = File::open(log_file).source_err(
        ExecReason::Io,
        format!("open log file: {}", log_file.display()),
    )?;
    file.seek(SeekFrom::Start(start_pos)).source_err(
        ExecReason::Io,
        format!("seek log file: {}", log_file.display()),
    )?;

    let mut buffer = vec![0; (end_pos - start_pos) as usize];
    file.read_exact(&mut buffer).source_err(
        ExecReason::Io,
        format!("read log file: {}", log_file.display()),
    )?;

    Ok(String::from_utf8_lossy(&buffer).into_owned())
}
#[derive(Serialize)]
pub enum ReadSignal {
    Start(u64),
    End(u64),
}
