package DfsFileUtil

import (
	"DairoDFS/appication/SystemConfig"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/LocalFileDao"
	"DairoDFS/dao/dto"
	controller "DairoDFS/exception"
	"bufio"
	"embed"
	_ "embed"
	"fmt"
	"github.com/shirou/gopsutil/disk"
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
	maxSize := SystemConfig.Instance().UploadMaxSize
	saveFolderList := SystemConfig.Instance().SaveFolderList
	if len(saveFolderList) == 0 {
		return "", &controller.BusinessException{
			Message: "没有配置存储目录",
		}
	}
	for _, folder := range saveFolderList {
		_, err := os.Stat(folder)
		if os.IsNotExist(err) { //如果文件夹不存在
			continue
		}
		usage, usageErr := disk.Usage(folder)
		if usageErr != nil {
			return "", usageErr
		}
		if usage.Free > maxSize { //空间足够
			return folder, nil
		}
	}
	return "", &controller.BusinessException{
		Message: "文件夹不存在或没有足够存储空间",
	}
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
		return "", &controller.BusinessException{
			Message: "准备创建的文件已经存在",
		}
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
		return &controller.BusinessException{
			Message: "文件路径不能包含>,?,\\,:,|,<,*,\"字符",
		}
	}
	if strings.Contains(path, "//") {
		return &controller.BusinessException{
			Message: "文件路径不能包含两个连续的字符/",
		}
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
	dfsFile := DfsFileDao.SelectOne(id)
	DownloadDfs(dfsFile, writer, request)
}

/**
 * 文件下载
 * @param id 文件ID
 * @param request 客户端请求
 * @param response 往客户端返回内容
 */
func DownloadDfs(dfsFile *dto.DfsFileDto, writer http.ResponseWriter, request *http.Request) {

	// 此处配置的是允许任意域名跨域请求，可根据需求指定
	//writer.Header().Set("Access-Control-Allow-Origin", request.getHeader("origin"))
	writer.Header().Set("Access-Control-Allow-Origin", "*")
	writer.Header().Set("Access-Control-Allow-Credentials", "true")
	writer.Header().Set("Access-Control-Allow-Methods", "GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS")
	writer.Header().Set("Access-Control-Allow-Headers", "*")

	// 如果是OPTIONS则结束请求
	// 跨域请求时用到
	if request.Method == http.MethodOptions {
		writer.WriteHeader(http.StatusNoContent)
		return
	}
	if dfsFile == nil { //文件不存在
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
	writer.Header().Set("Content-Type", *dfsFile.ContentType)

	//本地文件存储信息
	localFile := LocalFileDao.SelectOne(*dfsFile.LocalId)
	DownloadLocal(localFile, writer, request)
}

/**
 * 文件下载
 * @param localFile 本地文件存储信息
 * @param request 客户端请求
 * @param response 往客户端返回内容
 */
func DownloadLocal(localFile *dto.LocalFileDto, writer http.ResponseWriter, request *http.Request) {
	if localFile == nil {
		writer.WriteHeader(http.StatusNotFound)
		return
	}

	//在头部信息中加入文件MD5
	writer.Header().Set("Content-MD5", *localFile.Md5)
	download(*localFile.Path, writer, request)
}

//    /**
//     * 文件下载
//     * @param iStream 输入流
//     * @param size 数据大小
//     * @param request 客户端请求
//     * @param response 往客户端返回内容
//     */
//    fun download(iStream: InputStream, size: Long, request: HttpServletRequest, response: HttpServletResponse) {
//
//        //指定读取部分数据头部标识
//        val range = request.getHeader("range")
//        val start: Long
//        var end: Long
//        if (range == null) {
//            start = 0
//            end = size - 1
//            response.status = HttpStatus.OK.value()
//        } else {
//            val ranges = range.lowercase().replace("bytes=", "").split("-")
//            start = ranges[0].toLong()
//            if (ranges[1].isBlank()) {
//                end = size - 1
//            } else {
//                end = ranges[1].toLong()
//                if (end > size - 1) {
//                    end = size - 1
//                }
//            }
//            if (start > end) {//开始位置大于结束位置
//                response.status = HttpStatus.REQUESTED_RANGE_NOT_SATISFIABLE.value()
//                return
//            }
//            if (start >= size) {//开始位置大于文件大小
//                response.status = HttpStatus.REQUESTED_RANGE_NOT_SATISFIABLE.value()
//                return
//            }
//            response.setHeader("Content-Range", "bytes $start-$end/${size}")
//
//            //部分数据的状态值
//            response.status = HttpStatus.PARTIAL_CONTENT.value()
//        }
//        response.setContentLengthLong(end - start + 1)
//
//        //告诉客户端,服务器支持请求部分数据
//        response.setHeader("Accept-Ranges", "bytes")
//
//        // 允许客户端缓存
//        response.setHeader("Cache-Control", "public, max-age=31536000, s-maxage=31536000, immutable")
//        if (request.method == HttpMethod.HEAD.name()) {//只返回头部信息,不返回具体数据
//            return
//        }
//
//        //每次读取数据间隔时间(测试用)
//        val wait = request.getParameter("wait")?.toLong()
//        val oStream = response.outputStream
//        if (wait != null) {//测试用
//            iStream.use {
//                iStream.skip(start)
//                val data = ByteArray(1024) // 缓冲字节数组
//                var total = start
//                println("-->wait:$wait")
//                while (true) {
//                    Thread.sleep(wait)
//                    val len = iStream.read(data)
//                    if (len == -1) {//原则上读取的数据不可能为-1
//                        break
//                    }
//                    total += len
//                    println("-->total:${DownloadInterceptor.count}  range:$range  ${total.toDataSize}")
//                    oStream.write(data, 0, len)
//                }
//            }
//            return
//        }
//        iStream.use {
//            it.skip(start)
//            it.transferTo(oStream)
////            val data = ByteArray(32 * 1024) // 缓冲字节数组
////            var total = start
////            var readLen = data.size.toLong()
////
////            var isEnd = false
////            while (true) {
////
////                //还需要的数据长度
////                val needReadLen = (end - total + 1).toLong()
////                if (needReadLen <= data.size) {//还需要的数据长度小于或者等于设置的缓存数
////                    readLen = needReadLen
////                    isEnd = true
////                }
////                val len = iStream.read(data, 0, readLen.toInt())
////                if (len == -1) {//原则上读取的数据不可能为-1
////                    break
////                }
////                total += len
////                oStream.write(data, 0, len)
////                if (isEnd) {
////                    break
////                }
////            }
//        }
//        return
//    }
//}

/**
* 文件下载
* @param path 文件
* @param writer 往客户端返回内容
* @param request 客户端请求
 */
func download(path string, writer http.ResponseWriter, request *http.Request) {
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
