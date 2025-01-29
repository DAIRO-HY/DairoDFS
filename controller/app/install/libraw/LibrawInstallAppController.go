package libraw

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install"
	"DairoDFS/extension/String"
	"DairoDFS/util/ShellUtil"
	"embed"
	"fmt"
	"io"
	"net/http"
	"os"
	"runtime"
	"strings"
)

/**
 * 安装libraw
 */
//@Group:/app/install/libraw

// 下载信息
var downloadInfo *install.LibDownloadInfo

//go:embed libraw-install.sh
var librawInstallSH embed.FS

/**
 * 下载地址
 */
func url() string {
	switch runtime.GOOS {
	case "linux":
		return "https://github.com/LibRaw/LibRaw/archive/refs/tags/0.21.2.zip"
	case "windows":
		//return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/LibRaw-0.21.2-Win64.zip"
		return "https://github.com/LibRaw/LibRaw/archive/refs/tags/0.21.2.zip"
	case "darwin":
		return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/LibRaw-0.21.2-macOS.zip"
	default:
		return ""
	}
}

/**
 * 页面初始化
 */
//@Get:/install_libraw.html
//@Html:app/install/install_libraw.html
func Html() {
	downloadInfo = &install.LibDownloadInfo{
		Url:      url(),
		SavePath: application.LibrawPath,
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
	_, err := os.Stat(application.LibrawPath)
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
	//versionResult, cmdErr := ShellUtil.ExecToOkResult(application.LibrawPath + "/libraw -version")
	//if cmdErr == nil {
	//	outForm.Info = "安装完成：" + versionResult
	//	outForm.IsInstalled = true
	//	return outForm
	//}
	switch runtime.GOOS {
	case "windows":
		_, cmdErr := ShellUtil.ExecToOkResult(application.DcrawEmuPath + "/dcraw_emu" + " -version")
		if cmdErr != nil && strings.Contains(cmdErr.Error(), "Unknown option \"-version\".") {
			outForm.Info = "安装完成"
			outForm.IsInstalled = true
			return outForm
		}
		outForm.Info = fmt.Sprintf("安装失败：%q", cmdErr)
		return outForm
	case "linux":

		// 读取嵌入的文件内容
		data, _ := librawInstallSH.ReadFile("libraw-install.sh")
		targetFile := application.LibrawPath + "./libraw-install.sh"

		//0755：文件所有者可读、写、执行，同组用户和其他用户可读、执行。
		writeShFileErr := os.WriteFile(targetFile, data, 0755)
		if writeShFileErr != nil {
			outForm.Info = fmt.Sprintf("安装失败：%q", writeShFileErr)
			return outForm
		}

		cache := make([]byte, 8*1024)

		installResultSize := 0
		_, installCmdErr := ShellUtil.ExecToOkReader(application.LibrawPath+"/libraw-install.sh", func(rc io.ReadCloser) {
			n, _ := rc.Read(cache)
			fmt.Println(string(cache[0:n]))
			installResultSize += n
			outForm.Info = String.ToString(installResultSize)
		})
		if installCmdErr != nil {
			outForm.Info = fmt.Sprintf("安装失败：%q", installCmdErr)
			return outForm
		}

		//	if strings.Contains(cmdErr.Error(), "error=13") { //没有赋予可执行权限
		//
		//		//开启可执行权限
		//		_, versionCmdErr := ShellUtil.ExecToOkResult("chmod -R +x " + application.LibrawPath)
		//		if versionCmdErr != nil {
		//			outForm.Info = fmt.Sprintf("安装失败：%q", versionCmdErr)
		//			return outForm
		//		}
		//
		//		//再次获取版本号
		//		versionResult, cmdErr = ShellUtil.ExecToOkResult(application.LibrawPath + "/libraw -version")
		//		if cmdErr == nil {
		//			outForm.Info = "安装完成：" + versionResult
		//			outForm.IsInstalled = true
		//			return outForm
		//		}
		//		outForm.Info = fmt.Sprintf("安装失败：%q", cmdErr)
		//	}
		//case "darwin":
		//}

		//    } catch (e: Exception) {
		//        form.hasFinish = false
		//        val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
		//        if (osName == "linux") {
		//            if (e.message!!.contains("error=13")) {
		//
		//                //开启可执行权限
		//                ShellUtil.exec(
		//                    """chmod -R +x "${
		//                        File(Constant.LIBRAW_PATH).absolutePath.replace(
		//                            "/./",
		//                            "/"
		//                        )
		//                    }""""
		//                )
		//                val version = ShellUtil.exec("${Constant.LIBRAW_PATH}/libraw -version")
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
		//                        File(Constant.LIBRAW_PATH).absolutePath.replace(
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
	}
	return outForm
}
