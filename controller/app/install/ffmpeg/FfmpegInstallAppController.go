package ffmpeg

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install"
	"DairoDFS/util/ShellUtil"
	"net/http"
	"runtime"
)

/**
 * 安装ffmpeg
 */
//@Group:/app/install/ffmpeg

// 下载信息
var downloadInfo *install.LibDownloadInfo

/**
 * 下载地址
 */
func url() string {
	switch runtime.GOOS {
	case "linux":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-linux-amd64.zip"
	case "windows":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-win.zip"
	case "darwin":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-macOS.zip"
	default:
		return ""
	}
}

/**
 * 页面初始化
 */
//@Get:/install_ffmpeg.html
//@Html:app/install/install_ffmpeg.html
func Html() {
	if downloadInfo == nil {
		downloadInfo = &install.LibDownloadInfo{
			Url:      url(),
			SavePath: application.FfmpegPath,
		}
	}
	if validate() != nil {
		downloadInfo.IsInstalled = false
	}
}

// 资源回收
// @Post:/recycle
func Recycle() {
	downloadInfo = nil
}

/**
 * 开始安装
 */
//@Post:/install
func Install() {
	go downloadInfo.DownloadAndUnzip(validate, doInstall)
}

// 当前安装进度
// @Request:/progress
func Progress(writer http.ResponseWriter, request *http.Request) {
	downloadInfo.SendProgress(writer, request)
}

// 文件已经下载完成，获取安装信息
func doInstall() {
	switch runtime.GOOS {
	case "darwin":
	}

	//    } catch (e: Exception) {
	//        form.hasFinish = false
	//        val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
	//        if (osName == "linux") {
	//            if (e.message!!.contains("error=13")) {
	//
	//                //开启可执行权限
	//                ShellUtil.exec(
	//                    """chmod -R +x "${
	//                        File(Constant.FFMPEG_PATH).absolutePath.replace(
	//                            "/./",
	//                            "/"
	//                        )
	//                    }""""
	//                )
	//                val version = ShellUtil.exec("${Constant.FFMPEG_PATH}/ffmpeg -version")
	//                form.info = "安装完成:$version"
	//                form.hasFinish = true
	//                return form
	//            }
	//        } else if (osName.contains("mac")) {//mac系统是
	//            if (e.message!!.contains("error=13")) {
	//                form.error =
	//                    "安装失败:$e\n解决方案:请在弹出的Terminal窗口中验证密码,然后回到当前页面,再次点击安装按钮即可."
	//
	//                // 使用 ProcessBuilder 打开终端并执行指定的命令
	//                val pb = ProcessBuilder(
	//                    "osascript",
	//                    "-e",
	//                    "tell application \"Terminal\" to do script \"sudo chmod -R +x ${
	//                        File(Constant.FFMPEG_PATH).absolutePath.replace(
	//                            "/./",
	//                            "/"
	//                        )
	//                    }\"",
	//                    "-e", "tell application \"Terminal\" to activate"
	//                )
	//                pb.start()
	//            }
	//            return form
	//        }
	//        form.error = "安装失败:$e"
	//    }
}

// 验证安装结果
func validate() error {
	versionResult, cmdErr := ShellUtil.ExecToOkResult(application.FfmpegPath + "/ffmpeg -version")
	if cmdErr == nil {
		downloadInfo.Info = "安装完成：" + versionResult
		downloadInfo.IsInstalled = true
		return nil
	}
	return cmdErr
}
