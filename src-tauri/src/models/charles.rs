use core::fmt;
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::time::sleep;
#[derive(Debug)]
pub enum CharlesConvertError {
    FileNotFound(PathBuf),
    ConversionFailed(String),
    IOError(std::io::Error),
}

pub struct CharlesConverter {
    charles_path: PathBuf,
}

impl fmt::Display for CharlesConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharlesConvertError::FileNotFound(path) => {
                write!(f, "文件未找到: {:?}", path)
            }
            CharlesConvertError::ConversionFailed(message) => {
                write!(f, "转换失败: {}", message)
            }
            CharlesConvertError::IOError(err) => {
                write!(f, "IO 错误: {}", err)
            }
        }
    }
}

// 实现 std::error::Error trait
impl std::error::Error for CharlesConvertError {}

impl CharlesConverter {
    /// 创建转换器实例  
    pub fn new(charles_path: &str) -> Result<Self, CharlesConvertError> {
        let path = PathBuf::from(charles_path);

        // 验证路径是否存在
        if !path.exists() {
            return Err(CharlesConvertError::FileNotFound(path));
        }

        Ok(Self { charles_path: path })
    }
    /// 转换单个 .chls 文件到 .har  
    pub fn convert_to_har(
        &self,
        input_path: &str,
        output_path: Option<&str>,
    ) -> Result<PathBuf, CharlesConvertError> {
        // 验证输入文件是否存在
        let input = Path::new(input_path);
        if !input.exists() {
            return Err(CharlesConvertError::FileNotFound(input.to_path_buf()));
        }

        // 如果没有提供输出路径，使用输入文件的目录
        let output = match output_path {
            Some(path) => PathBuf::from(path),
            None => {
                let mut output = input.to_path_buf();
                output.set_extension("har");
                output
            }
        };

        // 执行转换命令
        let output_result = Command::new(&self.charles_path)
            .args(&["convert", input_path, output.to_str().unwrap()])
            .output()
            .map_err(CharlesConvertError::IOError)?;

        // 检查转换是否成功
        if output_result.status.success() {
            debug!("成功转换: {} -> {}", input_path, output.to_string_lossy());
            Ok(output)
        } else {
            Err(CharlesConvertError::ConversionFailed(
                String::from_utf8_lossy(&output_result.stderr).to_string(),
            ))
        }
    }

    /// 批量转换 .chls 文件到 .har  
    pub fn batch_convert(
        &self,
        input_dir: &str,
        output_dir: Option<&str>,
    ) -> Result<Vec<PathBuf>, CharlesConvertError> {
        // 验证输入目录
        let input_path = Path::new(input_dir);
        if !input_path.is_dir() {
            return Err(CharlesConvertError::FileNotFound(input_path.to_path_buf()));
        }

        // 确定输出目录
        let output_path = match output_dir {
            Some(dir) => PathBuf::from(dir),
            None => input_path.to_path_buf(),
        };

        // 创建输出目录
        std::fs::create_dir_all(&output_path).map_err(CharlesConvertError::IOError)?;

        // 批量转换
        let converted_files: Vec<PathBuf> = std::fs::read_dir(input_path)
            .map_err(CharlesConvertError::IOError)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension().map_or(false, |ext| ext == "chls") {
                    let file_name = path.file_name()?;
                    let output_file = output_path.join(file_name).with_extension("har");

                    match self.convert_to_har(path.to_str()?, Some(output_file.to_str()?)) {
                        Ok(converted_path) => Some(converted_path),
                        Err(e) => {
                            eprintln!("转换失败 {:?}: {:?}", path, e);
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(converted_files)
    }
}

const PROCESS_NAMES: [&str; 3] = ["Charles", "Charles.exe", "charles"];

pub fn is_charles_running() -> bool {
    let mut system = System::new_all();
    system.refresh_processes();

    for process in system.processes_by_name("") {
        let name = process.name().to_lowercase();
        if PROCESS_NAMES
            .iter()
            .any(|&p| name.contains(&p.to_lowercase()))
        {
            return true;
        }
    }

    false
}

pub async fn kill_charles_async() -> Result<bool, String> {
    // 先执行终止操作
    let killed_result = {
        let mut system = System::new_all();
        system.refresh_processes();

        let mut found_charles = false;
        let mut killed_any = false;
        let mut error_message = String::new();

        for (pid, process) in system.processes() {
            let name = process.name().to_lowercase();
            if PROCESS_NAMES
                .iter()
                .any(|&p| name.contains(&p.to_lowercase()))
            {
                found_charles = true;
                debug!("找到Charles进程: {} (PID: {})", process.name(), pid);

                if process.kill() {
                    killed_any = true;
                } else {
                    // 记录错误信息
                    error_message = format!(
                        "无法终止进程 {} (PID: {})，可能需要管理员权限",
                        process.name(),
                        pid
                    );
                }
            }
        }

        if found_charles && !killed_any {
            Err(error_message)
        } else {
            Ok(killed_any)
        }
    };

    // 检查终止操作的结果
    match killed_result {
        Err(e) => Err(e),
        Ok(false) => Ok(false), // 没有终止任何进程
        Ok(true) => {
            // 设置超时时间
            let start = Instant::now();
            let timeout = Duration::from_secs(2);

            loop {
                // 先检查进程状态
                if !is_charles_running() {
                    return Ok(true);
                }

                // 再检查是否超时
                if start.elapsed() >= timeout {
                    return Err(String::from("尝试终止Charles进程超时，进程可能仍在运行"));
                }

                // 等待下一个检查周期
                sleep(Duration::from_millis(50)).await;
            }
        }
    }
}
