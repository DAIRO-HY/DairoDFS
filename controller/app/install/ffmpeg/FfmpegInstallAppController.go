package ffmpeg

import (
	"DairoDFS/application"
	"DairoDFS/controller/app/install/ffmpeg/form"
	"DairoDFS/extension/Number"
	"DairoDFS/util/ShellUtil"
	"archive/zip"
	"bytes"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"time"
)

/**
 * 安装ffmpeg
 */
//@Group:/app/install/ffmpeg

// 下载信息
var downloadInfo *DownloadInfo

/**
 * 页面初始化
 */
//@Get:/install_ffmpeg.html
//@Html:app/install/install_ffmpeg.html
func Html() {}

/**
 * 开始安装
 */
//@Post:/install
func Install() {
	_, err := os.Stat(application.FfmpegPath)
	if !os.IsNotExist(err) { //文件存在
		return
	}
	if downloadInfo != nil { //正在下载中
		return
	}
	go downloadAndUnzip()
}

// 下载并解压
func downloadAndUnzip() {
	defer func() {
		downloadInfo = nil
	}()
	downloadInfo = &DownloadInfo{}
	downloadInfo.info = "正在下载"
	downloadError := downloadFile()
	if downloadError != nil {
		downloadInfo.info = "安装失败"
		return
	}
	downloadInfo.info = "正在解压"
	downloadInfo.info = "安装完成"
}

/**
 * 下载地址
 */
func url() string {
	switch osName := runtime.GOOS; osName {
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

// 下载文件
func downloadFile() error {

	// 创建HTTP GET请求
	resp, err := http.Get(url())
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	// 检查HTTP状态码
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("failed to download file: status code %d", resp.StatusCode)
	}

	//得到文件总大小
	downloadInfo.total = resp.ContentLength

	// 将响应体写入内存
	_, err = io.Copy(&downloadInfo.downloadBuffer, resp.Body)
	if err != nil {
		return err
	}
	return unzip()
}

// unzip 解压 zip 文件到目标目录
func unzip() error {

	// 创建一个 zip.Reader 来读取缓冲区中的 zip 数据
	r, err := zip.NewReader(bytes.NewReader(downloadInfo.downloadBuffer.Bytes()), int64(downloadInfo.downloadBuffer.Len()))
	if err != nil {
		return err
	}

	// 遍历 zip 文件中的每个文件
	for _, f := range r.File {
		filePath := filepath.Join(application.FfmpegPath, f.Name)

		// 检查路径安全，防止 Zip Slip 漏洞
		if !filepath.HasPrefix(filePath, filepath.Clean(application.FfmpegPath)+string(os.PathSeparator)) {
			return fmt.Errorf("非法文件路径: %s", filePath)
		}

		if f.FileInfo().IsDir() {
			// 创建目录
			if err = os.MkdirAll(filePath, os.ModePerm); err != nil {
				return err
			}
		} else {
			// 创建解压文件
			if err = os.MkdirAll(filepath.Dir(filePath), os.ModePerm); err != nil {
				return err
			}

			outFile, err := os.OpenFile(filePath, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, f.Mode())
			if err != nil {
				return err
			}
			defer outFile.Close()

			// 打开 zip 文件内容
			rc, err := f.Open()
			if err != nil {
				return err
			}
			defer rc.Close()

			// 将内容写入解压文件
			_, err = io.Copy(outFile, rc)
			if err != nil {
				return err
			}
		}
	}
	return nil
}

/**
 * 获取下载进度
 */
//@Post:/progress
func Progress() form.FfmpegInstallProgressForm {
	outForm := form.FfmpegInstallProgressForm{}
	_, err := os.Stat(application.FfmpegPath)
	if !os.IsNotExist(err) { //已经安装完成
		versionResult, cmdErr := ShellUtil.ExecToOkResult(application.FfmpegPath + "/ffmpeg -version")
		if cmdErr == nil {
			outForm.Info = "安装完成：" + versionResult
			outForm.HasFinish = true
			return outForm
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
		//    return form
	}
	if downloadInfo == nil { //还没有开始安装
		outForm.HasRuning = false
		return outForm
	}
	outForm.Info = downloadInfo.info
	outForm.HasRuning = true

	currentDownloadedSize := int64(downloadInfo.downloadBuffer.Len())
	now := int64(time.Now().UnixMilli())

	fmt.Println(currentDownloadedSize)

	//计算下载速度
	speed := (currentDownloadedSize - downloadInfo.lastDownloadedSize) / (now - downloadInfo.lastProgressTime)

	downloadInfo.lastProgressTime = now
	downloadInfo.lastDownloadedSize = currentDownloadedSize

	//        form.url = FFMPEGInstallAppController.DOWNLOAD_URL
	outForm.Total = Number.ToDataSize(downloadInfo.total)
	outForm.DownloadedSize = Number.ToDataSize(currentDownloadedSize)
	outForm.Speed = Number.ToDataSize(speed) + "/S"
	prog := 0.0
	if downloadInfo.total > 0 {
		prog = float64(currentDownloadedSize) / float64(downloadInfo.total)
	}
	outForm.Progress = int(prog * 100)
	return outForm
}
