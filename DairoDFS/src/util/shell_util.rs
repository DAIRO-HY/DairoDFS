use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// 执行外部程序,返回标准输出的数据
/// 执行失败时返回错误流的内容
/// - program 程序路径
/// - args 参数(不经过shell解析,路径里有空格也没问题)
pub async fn exec<I, S>(program: &Path, args: I) -> Result<Vec<u8>, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    exec_with_input(program, args, None).await
}

/// 执行外部程序,并把数据写入它的标准输入
pub async fn exec_with_input<I, S>(program: &Path, args: I, input: Option<Vec<u8>>) -> Result<Vec<u8>, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new(program)
        .args(args)
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("执行{}失败: {}", program.display(), e))?;
    if let Some(input) = input {
        let mut stdin = child.stdin.take().unwrap();

        //在另一个任务里写入,避免输出缓冲区满了之后互相等待
        tokio::spawn(async move {
            let _ = stdin.write_all(&input).await;
        });
    }
    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("执行{}失败: {}", program.display(), e))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if err.is_empty() {
            return Err(format!("执行{}失败: {}", program.display(), output.status));
        }
        return Err(err);
    }
    Ok(output.stdout)
}

/// 获取工具的执行路径:安装目录下存在时使用安装目录下的程序,否则使用系统PATH里的程序
/// - folder 安装目录
/// - name 程序名
pub fn tool_path(folder: &Path, name: &str) -> PathBuf {
    let file_name = if cfg!(windows) { format!("{}.exe", name) } else { name.to_string() };
    let path = folder.join(&file_name);
    if path.is_file() {
        return path;
    }
    PathBuf::from(name)
}

/// 获取通过brew安装的工具路径(macOS)
/// 程序在后台运行时PATH里可能没有brew的安装目录,所以PATH里找不到时再去brew的安装目录找
pub fn brew_tool_path(name: &str) -> PathBuf {
    if !cfg!(target_os = "macos") || find_in_path(name).is_some() {
        return PathBuf::from(name);
    }
    for folder in ["/opt/homebrew/bin", "/usr/local/bin"] {
        let path = Path::new(folder).join(name);
        if path.is_file() {
            return path;
        }
    }
    PathBuf::from(name)
}

/// 在PATH环境变量的目录里查找程序
fn find_in_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|it| it.join(name))
        .find(|it| it.is_file())
}
