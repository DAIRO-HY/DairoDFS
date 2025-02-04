package SyncFileUtil

import (
	"DairoDFS/application"
	"DairoDFS/extension/String"
	"DairoDFS/util/Sync/bean"
	"fmt"
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
func Download(info bean.SyncServerInfo, md5 string, retryTimes int) string {

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
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		fmt.Println("Error creating request:", err)
		return ""
	}

	// 设置请求头信息
	req.Header.Set("Range", "bytes="+String.ValueOf(downloadStart)+"-")

	transport := &http.Transport{
		DialContext:           (&net.Dialer{Timeout: 10 * time.Second}).DialContext, //连接超时
		ResponseHeaderTimeout: 10 * time.Second,                                     //读数据超时
	}
	client := &http.Client{Transport: transport}
	resp, err := client.Do(req)
	if err != nil {
		fmt.Println(err)
		return ""
	}
	defer resp.Body.Close()

	//已经下载文件大小
	var downloadedSize = downloadStart
	//conn.setRequestProperty("Range", "bytes=${downloadStart}-")

	//返回状态码
	httpStatus := resp.StatusCode
	if httpStatus == http.StatusRequestedRangeNotSatisfiable { //文件可能已经下载完成
		return savePath
	}
	if httpStatus != http.StatusOK && httpStatus != http.StatusPartialContent { //请求数据发生错误
		//TODO:应该返回具体错误信息
		return ""
	}

	//文件总大小
	total := resp.ContentLength + downloadedSize

	//设置读物数据缓存
	cache := make([]byte, 64*1024)
	file, err := os.OpenFile(savePath, os.O_APPEND|os.O_WRONLY|os.O_CREATE, 0644)
	defer file.Close()
	for {
		n, readErr := resp.Body.Read(cache)
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
		//throw RuntimeException("文件虽然下载完成,但文件并不完成,请排查问题;total=${total} downloadedSize=${downloadedSize} fileSize=${saveFile.length()}")
		return ""
	}
	info.Msg = ""
	return savePath
	//} catch (e: Exception) {
	//    if (retryTimes < 10) {//重试10次
	//        info.msg = "文件下载失败,正常第${retryTimes + 1}次尝试重试"
	//        Thread.sleep(15_00)
	//        return this.download(info, md5, retryTimes + 1)
	//    } else {
	//        throw e
	//    }
	//} finally {
	//    conn.disconnect()
	//}
}
