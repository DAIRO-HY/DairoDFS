package DfsFileUtil

import (
    "embed"
    _ "embed"
)

//go:embed content-type.txt
var contentTypeTxt embed.FS


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
//
//    /**
//     * 获取路径的父级文件夹路径
//     */
//    val String.dfsContentType: String
//        get() {
//            val ext = this.lowercase()
//            return this@DfsFileUtil.extToContentType[ext] ?: "application/octet-stream"//未知文件类型
//        }
//
//    /**
//     * 判断储存路径的磁盘剩余容量,选择合适的目录
//     */
//    private val selectDriverFolder: String
//        get() {
//            val maxSize = SystemConfig.instance.uploadMaxSize
//            val saveFolderList = SystemConfig.instance.saveFolderList
//            if (saveFolderList.isEmpty()) {
//                throw BusinessException("没有配置存储目录")
//            }
//            SystemConfig.instance.saveFolderList.forEach {
//                val localFolder = File(it)
//                if (!localFolder.exists()) {
//                    throw BusinessException("目录${localFolder.absolutePath}不存在")
//                }
//                val freeSpace = localFolder.freeSpace
//                if (freeSpace > maxSize) {
//                    return it
//                }
//            }
//            throw BusinessException("储存目录剩余空间太小不足")
//        }
//
//
//    /**
//     * 获取本地文件存储路径
//     */
//    val localPath: String
//        get() {
//
//            //选择合适的文件夹储存
//            val localSaveFolder = this.selectDriverFolder
//            val dateFormat = SimpleDateFormat("yyyyMM").format(Date())
//
//            //拼接文件名
//            val path = "$localSaveFolder/$dateFormat/${System.currentTimeMillis()}_${StringUtil.getRandomNum(4)}"
//            if (!File(path).parentFile.exists()) {
//                File(path).parentFile.mkdirs()
//            }
//            return path
//        }
//
//    /**
//     * 文件上传限制
//     */
//    val uploadMaxSize: Long
//        get() {
//            return SystemConfig.instance.uploadMaxSize
//        }
//
//    /**
//     * 检查文件路径是否合法
//     * @param path 文件路径
//     */
//    fun checkPath(path: String) {
//        if (Regex("""[>,?,\\,:,|,<,*,"]""").containsMatchIn(path)) {
//            throw BusinessException("文件路径不能包含>,?,\\,:,|,<,*,\"字符")
//        }
//        if (path.contains("//")) {
//            throw BusinessException("文件路径不能包含两个连续的字符/")
//        }
//    }
//
//    /**
//     * 文件下载
//     * @param id 文件ID
//     * @param request 客户端请求
//     * @param response 往客户端返回内容
//     */
//    fun download(id: Long, request: HttpServletRequest, response: HttpServletResponse) {
//        val dfsFile = Boot.service.dfsFileDao.selectOne(id)
//        this.download(dfsFile, request, response)
//    }
//
//    /**
//     * 文件下载
//     * @param id 文件ID
//     * @param request 客户端请求
//     * @param response 往客户端返回内容
//     */
//    fun download(dfsFile: DfsFileDto?, request: HttpServletRequest, response: HttpServletResponse) {
//        response.reset() //清除buffer缓存
//
//        // 此处配置的是允许任意域名跨域请求，可根据需求指定
//        //response.setHeader("Access-Control-Allow-Origin", request.getHeader("origin"))
//        response.setHeader("Access-Control-Allow-Origin", "*")
//        response.setHeader("Access-Control-Allow-Credentials", "true")
//        response.setHeader("Access-Control-Allow-Methods", "GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS")
//        response.setHeader("Access-Control-Allow-Headers", "*")
//
//        // 如果是OPTIONS则结束请求
//        // 跨域请求时用到
//        if (HttpMethod.OPTIONS.toString() == request.method) {
//            response.status = HttpStatus.NO_CONTENT.value()
//            return
//        }
//        if (dfsFile == null) {//文件不存在
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//        val download = request.getParameter("download")
//        if (download != null) {//下载模式
//            val fileName = if (download.isBlank()) {
//                URLEncoder.encode(dfsFile.name, "UTF-8")
//            } else {
//                URLEncoder.encode(download, "UTF-8")
//            }
//            response.setHeader("Content-Disposition", "attachment;filename=$fileName")
//        }
//        response.contentType = dfsFile.contentType
//
//        //本地文件存储信息
//        val localFile = Boot.service.localFileDao.selectOne(dfsFile.localId!!)!!
//        this.download(localFile, request, response)
//    }
//
//    /**
//     * 文件下载
//     * @param localFile 本地文件存储信息
//     * @param request 客户端请求
//     * @param response 往客户端返回内容
//     */
//    fun download(localFile: LocalFileDto?, request: HttpServletRequest, response: HttpServletResponse) {
//        if (localFile == null) {
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//        val file = File(localFile.path!!)
//        if (!file.exists()) {
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//
//        //在头部信息中加入文件MD5
//        response.setHeader("Content-MD5", localFile.md5)
//
//        //文件大小
//        val size = file.length()
//        this.download(file.inputStream(), size, request, response)
//    }
//
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