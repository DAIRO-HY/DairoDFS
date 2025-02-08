package DfsFileUtil

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/LocalFileDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"bufio"
	"embed"
	_ "embed"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"time"
)

//go:embed content-type.txt
var contentTypeTxt embed.FS

// 生成文件时的同步锁，避免并发生成重复文件
var makePathLock sync.Mutex

//    /**
//     * 后缀对应ContentType
//     */
//    private val extToContentType = HashMap<String, String>()
//
//    init {
//        val iStream = DfsFileUtil.javaClass.classLoader.getResourceAsStream("content-type.txt")!!
//        val content = String(iStream.readAllBytes())
//        content.split("\n").forEach {
//            val indexSplit = it.indexOf(':')
//            if (indexSplit == -1) {
//                return@forEach
//            }
//            val key = it.substring(0, indexSplit).lowercase()
//            val value = it.substring(indexSplit + 1)
//            this.extToContentType[key] = value
//        }
//        iStream.close()
//    }

/**
 * 通过文件名获取文件的content-type
 */
func DfsContentType(ext string) string {
	file, err := contentTypeTxt.Open("content-type.txt")
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()
	ext = strings.ToLower(ext)
	scanner := bufio.NewScanner(file)
	for scanner.Scan() { //读取
		line := scanner.Text()
		if strings.HasPrefix(line, ext+":") {
			return line[len(ext)+1:]
		}
	}
	return "application/octet-stream" //未知文件类型
}

/**
 * 判断储存路径的磁盘剩余容量,选择合适的目录
 */
func SelectDriverFolder() (string, error) {
	//maxSize := SystemConfig.Instance().UploadMaxSize
	saveFolderList := SystemConfig.Instance().SaveFolderList
	if len(saveFolderList) == 0 {
		return "", exception.Biz("没有配置存储目录")
	}
	for _, folder := range saveFolderList {
		_, err := os.Stat(folder)
		if os.IsNotExist(err) { //如果文件夹不存在
			continue
		}
		//usage, usageErr := disk.Usage(folder)
		//if usageErr != nil {
		//	return "", usageErr
		//}
		//if usage.Free > uint64(maxSize) { //空间足够
		//	return folder, nil
		//}
		return folder, nil
	}
	return "", exception.Biz("文件夹不存在或没有足够存储空间")
}

/**
 * 获取本地文件存储路径
 */
func LocalPath() (string, error) {

	//选择合适的文件夹储存
	localSaveFolder, err := SelectDriverFolder()
	if err != nil {
		return "", err
	}
	dateFormat := time.Now().Format("2006-01")
	folder := localSaveFolder + "/" + dateFormat
	_, statErr := os.Stat(folder)
	if os.IsNotExist(statErr) { //文件夹不存在时
		mkdirErr := os.MkdirAll(folder, os.ModePerm)
		if mkdirErr != nil {
			return "", mkdirErr
		}
	}
	makePathLock.Lock()
	var path string
	for i := 0; i < 5; i++ {
		time.Sleep(1 * time.Microsecond)

		//拼接文件名
		path = folder + "/" + strconv.FormatInt(time.Now().UnixMicro(), 10)
		_, pathErr := os.Stat(path)
		if os.IsNotExist(pathErr) {
			break
		}
	}
	makePathLock.Unlock()
	_, pathErr := os.Stat(path)
	if os.IsExist(pathErr) { //文件已经存在，则报错（小概率事件）
		return "", exception.Biz("准备创建的文件已经存在")
	}
	return path, nil
}

/**
 * 检查文件路径是否合法
 * @param path 文件路径
 */
func CheckPath(path string) error {
	pattern := `[>,?,\\,:,|,<,*,"]`
	matched, _ := regexp.MatchString(pattern, path)
	if matched {
		return exception.Biz("文件路径不能包含>,?,\\,:,|,<,*,\"字符")
	}
	if strings.Contains(path, "//") {
		return exception.Biz("文件路径不能包含两个连续的字符/")
	}
	return nil
}

/**
 * 文件下载
 * @param id 文件ID
 * @param request 客户端请求
 * @param response 往客户端返回内容
 */
func DownloadDfsId(id int64, writer http.ResponseWriter, request *http.Request) {
	dfsFile, isExists := DfsFileDao.SelectOne(id)
	if !isExists { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	DownloadDfs(dfsFile, writer, request)
}

/**
 * 文件下载
 * @param id 文件ID
 * @param request 客户端请求
 * @param response 往客户端返回内容
 */
func DownloadDfs(dfsFile dto.DfsFileDto, writer http.ResponseWriter, request *http.Request) {

	// 此处配置的是允许任意域名跨域请求，可根据需求指定
	//writer.Header().Set("Access-Control-Allow-Origin", request.getHeader("origin"))
	writer.Header().Set("Access-Control-Allow-Origin", "*")
	writer.Header().Set("Access-Control-Allow-Credentials", "true")
	writer.Header().Set("Access-Control-Allow-Methods", "GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS")
	writer.Header().Set("Access-Control-Allow-Headers", "*")

	// 设置Cache-Control头，配置缓存（1年）
	writer.Header().Set("Cache-Control", "public, max-age=31536000, s-maxage=31536000, immutable")

	// 设置Expires头，配置为1年后的时间
	expiresTime := time.Now().AddDate(1, 0, 0).Format(time.RFC1123)
	writer.Header().Set("Expires", expiresTime)

	// 如果是OPTIONS则结束请求
	// 跨域请求时用到
	if request.Method == http.MethodOptions {
		writer.WriteHeader(http.StatusNoContent)
		return
	}
	if dfsFile.Id == 0 { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	//val download = request.getParameter("download")
	//if (download != null) {//下载模式
	//    val fileName = if (download.isBlank()) {
	//        URLEncoder.encode(dfsFile.name, "UTF-8")
	//    } else {
	//        URLEncoder.encode(download, "UTF-8")
	//    }
	//    response.setHeader("Content-Disposition", "attachment;filename=$fileName")
	//}
	writer.Header().Set("Content-Type", dfsFile.ContentType)

	//本地文件存储信息
	localFile, isExists := LocalFileDao.SelectOne(dfsFile.LocalId)
	if !isExists {
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	Download(localFile.Path, writer, request)
}

/**
* 文件下载
* @param path 文件
* @param writer 往客户端返回内容
* @param request 客户端请求
 */
func Download(path string, writer http.ResponseWriter, request *http.Request) {
	fileInfo, err := os.Stat(path)
	if os.IsNotExist(err) { //文件不存在
		writer.WriteHeader(http.StatusNotFound)
		return
	}

	//文件大小
	size := fileInfo.Size()

	//指定读取部分数据头部标识
	ranges := request.Header.Get("range")
	var start int64
	var end int64
	if len(ranges) == 0 {
		start = 0
		end = size - 1
	} else {
		//range格式：bytes=10-30 或者 bytes=10-30
		rangeArr := strings.Split(strings.ToLower(ranges)[6:], "-")
		start, _ = strconv.ParseInt(rangeArr[0], 10, 64)
		if len(rangeArr[1]) == 0 { //到文件末尾
			end = size - 1
		} else {
			end, _ = strconv.ParseInt(rangeArr[1], 10, 64)
			if end > size-1 { //超出了文件大小范围
				end = size - 1
			}
		}
		if start > end {
			writer.WriteHeader(http.StatusRequestedRangeNotSatisfiable)
			return
		}
		if start >= size {
			writer.WriteHeader(http.StatusRequestedRangeNotSatisfiable)
			return
		}
		writer.Header().Set("Content-Range", fmt.Sprintf("bytes %d-%d/%d", start, end, size))
	}
	//writer.Header().Set("Content-Type", "audio/mp3")
	writer.Header().Set("Content-Length", strconv.FormatInt(end-start+1, 10))

	//告诉客户端,服务器支持请求部分数据
	writer.Header().Set("Accept-Ranges", "bytes")

	file, err := os.Open(path)
	defer file.Close()
	if err != nil {
		writer.WriteHeader(http.StatusInternalServerError)
		return
	}

	//http.ResponseWriter发送状态码之后，再设置头部信息将会不生效，所以发送状态码一定要等所有头部信息设置完成之后再发送
	if len(ranges) == 0 {
		writer.WriteHeader(http.StatusOK)
	} else {
		writer.WriteHeader(http.StatusPartialContent)
	}

	//跳过前面部分数据
	file.Seek(start, io.SeekStart)
	data := make([]byte, 16*1024) // 缓冲字节数组
	var total = start
	for {

		//计算还需要的数据长度
		needReadLen := int(end - total + 1)
		n, readErr := file.Read(data)
		if readErr != nil {
			//if readErr != io.EOF { //如果不是文件读取完成标志,理论上，这里不会发生该异常
			//	writer.WriteHeader(http.StatusInternalServerError)
			//}
			break
		}
		total += int64(n)
		if needReadLen <= n { //还需要的数据长度小于本次读取到的数据长度
			writer.Write(data[:needReadLen])
			break
		} else {
			_, writeErr := writer.Write(data[:n])
			if writeErr != nil { //可能客户端已经关闭停止
				break
			}
		}
	}
}
