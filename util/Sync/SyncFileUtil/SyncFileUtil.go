package SyncFileUtil

import (
	"DairoDFS/application"
	"DairoDFS/exception"
	"DairoDFS/extension/String"
	"DairoDFS/util/Sync/bean"
	"io"
	"net"
	"net/http"
	"os"
	"time"
)

/**
 * 文件同步工具
 */

///**
// * 实时同步消息的Socket
// */
//private val socket = SyncWebSocketHandler::class.bean
//
///**
// * 数据存储目录
// */
//private val dataPath = Boot::class.bean.dataPath

/**
 * 开始同步
 * @param info 同步主机信息
 * @param md5 文件md5
 * @param retryTimes 记录出错重试次数
 * @return 存储目录
 */
func Download(info bean.SyncServerInfo, md5 string, retryTimes int) (string, error) {

	//得到文件存储目录
	savePath := application.TEMP_PATH + "/" + md5

	//断点下载开始位置
	var downloadStart int64

	saveFileInfo, err := os.Stat(savePath)
	if !os.IsNotExist(err) { //若文件已经存在
		downloadStart = saveFileInfo.Size()
	}
	url := info.Url + "/download/" + md5

	// 创建一个新的HTTP请求
	request, _ := http.NewRequest("GET", url, nil)

	// 设置请求头信息
	request.Header.Set("Range", "bytes="+String.ValueOf(downloadStart)+"-")
	transport := &http.Transport{
		DialContext:           (&net.Dialer{Timeout: 10 * time.Second}).DialContext, //连接超时
		ResponseHeaderTimeout: 10 * time.Second,                                     //读数据超时
	}
	client := &http.Client{Transport: transport}
	res, err := client.Do(request)
	if err != nil { //网络连接失败时可能会报错
		if retryTimes < 5 { //重试次数达到上线之后，直接报错
			time.Sleep(3 * time.Second) //先等待3秒再重试
			return Download(info, md5, retryTimes+1)
		} else {
			return "", err
		}
	}
	defer res.Body.Close()

	//已经下载文件大小
	var downloadedSize = downloadStart

	//返回状态码
	httpStatus := res.StatusCode
	if httpStatus == http.StatusRequestedRangeNotSatisfiable { //文件可能已经下载完成
		return savePath, nil
	}
	if httpStatus != http.StatusOK && httpStatus != http.StatusPartialContent { //请求数据发生错误
		errData, _ := io.ReadAll(res.Body)
		return "", exception.Biz("Status:" + String.ValueOf(httpStatus) + "  Body:" + string(errData))
	}

	//文件总大小
	total := res.ContentLength + downloadedSize

	//设置读物数据缓存
	cache := make([]byte, 64*1024)
	file, err := os.OpenFile(savePath, os.O_APPEND|os.O_WRONLY|os.O_CREATE, 0644)
	defer file.Close()
	for {
		n, readErr := res.Body.Read(cache)
		if readErr == io.EOF { //数据已经读取完毕
			break
		}
		if n > 0 {
			downloadedSize += int64(n)
			file.Write(cache[:n])
			info.Msg = "正在同步文件：" + String.ValueOf(downloadedSize) + "(${downloadedSize.toDataSize})/${total.toDataSize}"
		}
	}
	saveFileInfo, err = os.Stat(savePath)
	if downloadedSize != total || total != saveFileInfo.Size() {
		return "", exception.Biz("文件虽然下载完成,但文件似乎并不完整,请排查问题；Content-Length:" + String.ValueOf(total) + " downloadedSize:" + String.ValueOf(downloadedSize) + " 实际下载到的文件大小:" + String.ValueOf(saveFileInfo.Size()))
	}
	info.Msg = ""
	return savePath, nil
}
