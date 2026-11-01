use crate::application;
use crate::extension::number::NumberExtension;
use crate::util::{media, shell_util};
use serde::Serialize;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

/// 需要安装的媒体处理工具,按安装向导的顺序排列
pub const TOOLS: [&str; 5] = ["ffmpeg", "ffprobe", "libraw", "magick", "exiftool"];

/// 各工具的安装状态
static STATES: LazyLock<Mutex<HashMap<&'static str, InstallState>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 安装状态
#[derive(Debug, Default, Clone)]
struct InstallState {

    // 是否正在安装中
    is_running: bool,

    // 是否已经安装完成
    is_installed: bool,

    // 安装信息
    info: String,

    // 安装包总大小
    total: u64,

    // 已经下载的大小
    downloaded: u64,

    // 下载速度(字节/秒)
    speed: u64,
}

/// 安装进度(返回给前端)
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {

    // 是否正在安装中
    pub is_running: bool,

    // 是否已经安装完成
    pub is_installed: bool,

    // 安装包总大小
    pub total: String,

    // 已经下载的大小
    pub downloaded_size: String,

    // 下载速度
    pub speed: String,

    // 下载进度(百分比)
    pub progress: u64,

    // 安装信息
    pub info: String,
}

/// 获取工具名(只允许预定义的工具)
pub fn tool_name(name: &str) -> Option<&'static str> {
    TOOLS.into_iter().find(|it| *it == name)
}

/// 下载地址(与老版本一致)
fn url(tool: &str) -> &'static str {
    match (tool, std::env::consts::OS) {
        ("ffmpeg", "linux") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-linux-amd64.zip",
        ("ffmpeg", "windows") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-win.zip",
        ("ffmpeg", "macos") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-macOS.zip",
        ("ffprobe", "linux") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffprobe-7.0.2-linux-amd64.zip",
        ("ffprobe", "windows") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffprobe-7.0.2-win.zip",
        ("ffprobe", "macos") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffprobe-7.0.2-macOS.zip",
        ("libraw", "linux") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/refs/heads/main/LibRaw-0.21.2-source.zip",
        ("libraw", "windows") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/refs/heads/main/LibRaw-0.21.2-Win64.zip",
        ("libraw", "macos") => "https://github.com/DAIRO-HY/DairoDfsLib/raw/refs/heads/main/LibRaw-0.21.2-macOS.zip",
        _ => "",
    }
}

/// 获取下载地址,官网只保留最新版本的工具,下载地址需要在安装时获取
async fn download_url(tool: &str) -> Result<String, String> {
    match (tool, std::env::consts::OS) {
        ("magick", "windows") => {
            //从GitHub获取最新版本的安装程序
            let text = http_client()?
                .get("https://api.github.com/repos/ImageMagick/ImageMagick/releases/latest")
                .send()
                .await
                .map_err(|e| format!("获取ImageMagick最新版本失败：{}", e))?
                .text()
                .await
                .map_err(|e| format!("获取ImageMagick最新版本失败：{}", e))?;
            let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("获取ImageMagick最新版本失败：{}", e))?;
            json["assets"]
                .as_array()
                .and_then(|it| {
                    it.iter()
                        .filter_map(|asset| asset["browser_download_url"].as_str())
                        .find(|url| url.ends_with("-Q16-HDRI-x64-dll.exe"))
                })
                .map(|it| it.to_string())
                .ok_or_else(|| "没有找到ImageMagick的Windows安装程序".to_string())
        }
        ("exiftool", "windows") => {
            let version = exiftool_version().await?;
            Ok(format!("https://downloads.sourceforge.net/project/exiftool/exiftool-{}_64.zip", version))
        }
        _ => Ok(url(tool).to_string()),
    }
}

/// 获取exiftool的最新版本号
async fn exiftool_version() -> Result<String, String> {
    let version = http_client()?
        .get("https://exiftool.org/ver.txt")
        .send()
        .await
        .map_err(|e| format!("获取exiftool最新版本失败：{}", e))?
        .text()
        .await
        .map_err(|e| format!("获取exiftool最新版本失败：{}", e))?;
    let version = version.trim().to_string();
    if version.parse::<f64>().is_err() {
        return Err(format!("获取exiftool最新版本失败：{}", version));
    }
    Ok(version)
}

/// GitHub的接口要求必须有User-Agent
fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(concat!("DairoDFS/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())
}

/// 安装目录
fn save_path(tool: &str) -> PathBuf {
    match tool {
        "ffmpeg" => application::FFMPEG_PATH.clone(),
        "ffprobe" => application::FFPROBE_PATH.clone(),
        "libraw" => application::LIBRAW_PATH.clone(),
        "magick" => application::IMAGEMAGICK_PATH.clone(),
        _ => application::EXIFTOOL_PATH.clone(),
    }
}

/// 修改安装状态
fn update(tool: &'static str, f: impl FnOnce(&mut InstallState)) {
    let mut states = STATES.lock().unwrap();
    f(states.entry(tool).or_default());
}

/// 获取安装进度,没有在安装时先检查是否已经安装
pub async fn progress(tool: &'static str) -> InstallProgress {
    let is_running = STATES.lock().unwrap().get(tool).is_some_and(|it| it.is_running);
    if !is_running {
        let result = validate(tool).await;
        update(tool, |it| match result {
            Ok(info) => {
                it.is_installed = true;
                it.info = info;
            }
            Err(_) => {
                it.is_installed = false;
                if it.info.is_empty() || it.info.starts_with("安装完成") {
                    it.info = "点击“安装”按钮开始安装。".to_string();
                }
            }
        });
    }
    current_progress(tool)
}

/// 获取当前的安装进度(不检查是否已经安装)
pub fn current_progress(tool: &'static str) -> InstallProgress {
    let state = STATES.lock().unwrap().get(tool).cloned().unwrap_or_default();
    let total = if state.total > 0 { state.total } else { state.downloaded };
    InstallProgress {
        is_running: state.is_running,
        is_installed: state.is_installed,
        total: total.data_size(),
        downloaded_size: state.downloaded.data_size(),
        speed: format!("{}/S", state.speed.data_size()),
        progress: if total > 0 { state.downloaded * 100 / total } else { 0 },
        info: state.info,
    }
}

/// 开始安装(在后台执行),已经在安装中时不做任何处理
pub fn install(tool: &'static str) {
    {
        let mut states = STATES.lock().unwrap();
        let state = states.entry(tool).or_default();
        if state.is_running {
            return;
        }
        *state = InstallState {
            is_running: true,
            info: "准备安装".to_string(),
            ..Default::default()
        };
    }
    tokio::spawn(async move {
        let result = match do_install(tool).await {
            Ok(_) => validate(tool).await,
            Err(e) => Err(e),
        };
        update(tool, |it| {
            it.is_running = false;
            match result {
                Ok(info) => {
                    it.is_installed = true;
                    it.info = info;
                }
                Err(e) => it.info = format!("安装失败：{}", e),
            }
        });
    });
}

/// 下载、解压并执行安装
async fn do_install(tool: &'static str) -> Result<(), String> {
    let save_path = save_path(tool);
    update(tool, |it| it.info = "获取下载地址".to_string());
    let url = download_url(tool).await?;
    let url = url.as_str();
    if !url.is_empty() {
        //删除之前的安装目录
        let _ = tokio::fs::remove_dir_all(&save_path).await;
        let file_name = url.rsplit('/').next().unwrap_or(tool).to_string();
        let download_path = application::TEMP_PATH.join(format!("{}.download", file_name));
        let result = download(tool, url, &download_path).await;
        let result = match result {
            Ok(_) if file_name.ends_with(".zip") => {
                update(tool, |it| it.info = "正在解压".to_string());
                let (zip_path, target) = (download_path.clone(), save_path.clone());
                media::blocking(move || unzip(&zip_path, &target)).await
            }
            Ok(_) => {
                //不是压缩文件,直接保存到安装目录
                tokio::fs::create_dir_all(&save_path).await.map_err(|e| e.to_string())?;
                tokio::fs::rename(&download_path, save_path.join(&file_name)).await.map_err(|e| e.to_string())
            }
            Err(e) => Err(e),
        };
        let _ = tokio::fs::remove_file(&download_path).await;
        result?;
    }
    match (tool, std::env::consts::OS) {
        ("libraw", "linux") => run_script(tool, include_str!("../../assets/install-script/libraw-install.sh"), "libraw-install.sh", 250820).await,
        ("magick", "linux") => run_script(tool, include_str!("../../assets/install-script/imagemagick-install.sh"), "imagemagick-install.sh", 320820).await,
        ("exiftool", "linux") => run_script(tool, include_str!("../../assets/install-script/exiftool-install.sh"), "exiftool-install.sh", 97943).await,
        ("magick", "macos") => run_command(tool, &brew_install("imagemagick"), &save_path, 302021).await,
        ("exiftool", "macos") => run_command(tool, &brew_install("exiftool"), &save_path, 846).await,
        ("magick", "windows") => {
            update(tool, |it| it.info = "正在安装：请按照弹出的安装界面提示安装软件".to_string());
            let exe = save_path.join(url.rsplit('/').next().unwrap_or_default());
            let status = Command::new(exe).status().await.map_err(|e| e.to_string())?;
            if !status.success() {
                return Err(status.to_string());
            }
            Ok(())
        }
        ("exiftool", "windows") => {
            //压缩包里的目录名带版本号,如 exiftool-13.59_64
            let version = url.rsplit('/').next().unwrap_or_default().trim_end_matches(".zip");
            let folder = save_path.join(version);
            tokio::fs::rename(folder.join("exiftool(-k).exe"), folder.join("exiftool.exe")).await.map_err(|e| e.to_string())
        }
        _ => Ok(()),
    }
}

/// brew安装命令,程序在后台运行时PATH里可能没有brew,所以使用完整路径
fn brew_install(formula: &str) -> String {
    format!("\"{}\" install {}", shell_util::brew_tool_path("brew").to_string_lossy(), formula)
}

/// 下载安装包
async fn download(tool: &'static str, url: &str, path: &Path) -> Result<(), String> {
    update(tool, |it| it.info = "准备下载".to_string());
    let mut resp = http_client()?.get(url).send().await.map_err(|e| format!("下载失败：{}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HttpStatus:{}", resp.status().as_u16()));
    }
    let total = resp.content_length().unwrap_or(0);
    update(tool, |it| {
        it.info = "下载中".to_string();
        it.total = total;
    });
    let mut file = tokio::fs::File::create(path).await.map_err(|e| e.to_string())?;
    let mut downloaded = 0u64;

    //用来计算下载速度
    let mut last_time = Instant::now();
    let mut last_downloaded = 0u64;
    while let Some(chunk) = resp.chunk().await.map_err(|e| format!("下载失败：{}", e))? {
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        let elapsed = last_time.elapsed().as_millis() as u64;
        let speed = if elapsed >= 500 {
            let speed = (downloaded - last_downloaded) * 1000 / elapsed;
            last_time = Instant::now();
            last_downloaded = downloaded;
            Some(speed)
        } else {
            None
        };
        update(tool, |it| {
            it.downloaded = downloaded;
            if let Some(speed) = speed {
                it.speed = speed;
            }
        });
    }
    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 解压到临时目录,完成之后再改名为安装目录
fn unzip(zip_path: &Path, target: &Path) -> Result<(), String> {
    let temp = target.with_extension("temp");
    let _ = std::fs::remove_dir_all(&temp);
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;

        //检查路径安全，防止 Zip Slip 漏洞
        let Some(name) = entry.enclosed_name() else {
            return Err(format!("非法文件路径: {}", entry.name().unwrap_or_default()));
        };
        let out_path = temp.join(name);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
        std::fs::write(&out_path, data).map_err(|e| e.to_string())?;

        //解压后的文件直接赋予可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
        }
    }
    std::fs::rename(&temp, target).map_err(|e| e.to_string())
}

/// 执行安装脚本(Linux)
async fn run_script(tool: &'static str, script: &str, file_name: &str, total_size: usize) -> Result<(), String> {
    let folder = save_path(tool);
    tokio::fs::create_dir_all(&folder).await.map_err(|e| e.to_string())?;
    let script_path = folder.join(file_name);
    tokio::fs::write(&script_path, script).await.map_err(|e| e.to_string())?;
    let command = format!("bash \"{}\"", script_path.to_string_lossy());
    run_command(tool, &command, &folder, total_size).await
}

/// 执行安装命令,并把输出的数据量作为安装进度
/// - total_size 预估的输出数据量
async fn run_command(tool: &'static str, command: &str, folder: &Path, total_size: usize) -> Result<(), String> {
    tokio::fs::create_dir_all(folder).await.map_err(|e| e.to_string())?;
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(format!("{} 2>&1", command))
        .current_dir(folder)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdout = child.stdout.take().unwrap();
    let mut buf = vec![0u8; 4096];
    let mut size = 0;

    //保留最后的输出,安装失败时显示
    let mut last_output = Vec::new();
    loop {
        let n = stdout.read(&mut buf).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        size += n;
        last_output.extend_from_slice(&buf[..n]);
        if last_output.len() > 2000 {
            last_output.drain(..last_output.len() - 2000);
        }
        update(tool, |it| it.info = format!("正在安装：{}/{}", size, total_size));
    }
    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(String::from_utf8_lossy(&last_output).to_string());
    }
    Ok(())
}

/// 验证是否已经安装
/// - return 安装完成时的信息
async fn validate(tool: &str) -> Result<String, String> {
    let (program, arg) = match tool {
        "ffmpeg" => (media::ffmpeg(), "-version"),
        "ffprobe" => (media::ffprobe(), "-version"),
        "libraw" => (media::dcraw_emu(), "-version"),
        "magick" => (media::magick(), "--version"),
        _ => (media::exiftool(), "-ver"),
    };
    let output = Command::new(&program)
        .arg(arg)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("{}: {}", program.display(), e))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = text.trim();
    let ok = match tool {
        //dcraw_emu不支持-version参数,能执行就说明已经安装
        "libraw" => true,
        "magick" => text.starts_with("Version: ImageMagick"),
        "exiftool" => output.status.success() && text.parse::<f64>().is_ok(),
        _ => output.status.success(),
    };
    if !ok {
        return Err(text.to_string());
    }
    if tool == "libraw" {
        return Ok("安装完成".to_string());
    }
    Ok(format!("安装完成：{}", text.lines().next().unwrap_or_default()))
}
