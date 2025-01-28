package ffprobe

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install"
	"DairoDFS/util/ShellUtil"
	"fmt"
	"net/http"
	"os"
	"runtime"
	"strings"
)

/**
 * 安装ffprobe
 */
//@Group:/app/install/ffprobe

// 下载信息
var downloadInfo *install.LibDownloadInfo

/**
 * 下载地址
 */
func url() string {
	switch runtime.GOOS {
	case "linux":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffprobe-7.0.2-linux-amd64.zip"
	case "windows":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffprobe-7.0.2-win.zip"
	case "darwin":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffprobe-7.0.2-macOS.zip"
	default:
		return ""
	}
}

/**
 * 页面初始化
 */
//@Get:/install_ffprobe.html
//@Html:app/install/install_ffprobe.html
func Html() {
	downloadInfo = &install.LibDownloadInfo{
		Url:      url(),
		SavePath: application.FfprobePath,
	}
	runtime.GC()
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
	_, err := os.Stat(application.FfprobePath)
	if !os.IsNotExist(err) { //文件存在
		return
	}
	if downloadInfo.IsRuning { //正在下载中
		return
	}
	go downloadInfo.DownloadAndUnzip()
}

// 当前安装进度
// @Request:/progress
func Progress(writer http.ResponseWriter, request *http.Request) {
	downloadInfo.SendProgress(writer, request, getInstallInfo)
}

// 文件已经下载完成，获取安装信息
func getInstallInfo() install.LibInstallProgressForm {
	outForm := install.LibInstallProgressForm{}
	versionResult, cmdErr := ShellUtil.ExecToOkResult(application.FfprobePath + "/ffprobe -version")
	if cmdErr == nil {
		outForm.Info = "安装完成：" + versionResult
		outForm.IsInstalled = true
		return outForm
	}
	switch runtime.GOOS {
	case "windows":
		outForm.Info = fmt.Sprintf("安装失败：%q", cmdErr)
	case "linux":
		if strings.Contains(cmdErr.Error(), "error=13") { //没有赋予可执行权限

			//开启可执行权限
			_, versionCmdErr := ShellUtil.ExecToOkResult("chmod -R +x " + application.FfprobePath)
			if versionCmdErr != nil {
				outForm.Info = fmt.Sprintf("安装失败：%q", versionCmdErr)
				return outForm
			}

			//再次获取版本号
			versionResult, cmdErr = ShellUtil.ExecToOkResult(application.FfprobePath + "/ffprobe -version")
			if cmdErr == nil {
				outForm.Info = "安装完成：" + versionResult
				outForm.IsInstalled = true
				return outForm
			}
			outForm.Info = fmt.Sprintf("安装失败：%q", cmdErr)
		}
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
	//                        File(Constant.FFPROBE_PATH).absolutePath.replace(
	//                            "/./",
	//                            "/"
	//                        )
	//                    }""""
	//                )
	//                val version = ShellUtil.exec("${Constant.FFPROBE_PATH}/ffprobe -version")
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
	//                        File(Constant.FFPROBE_PATH).absolutePath.replace(
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
	return outForm
}
