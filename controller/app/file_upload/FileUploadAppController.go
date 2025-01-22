package file_upload

import (
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/File"
	"DairoDFS/extension/String"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/DfsFileUtil"
	"DairoDFS/util/LoginState"
	"net/http"
	"os"
)

// 文件上传Controller
//@Group:/app/file_upload

//@Value("\${data.path}")
//private lateinit var dataPath: String

/**
 * 记录当前正在上传的文件
 * 避免同一个文件同时上传导致文件数据混乱
 * md5 -> 上传时间戳
 */
var uploadingFileMap = map[string]int64{}

// 浏览器文件上传
// @Post:
func Upload(
	writer http.ResponseWriter,
	request *http.Request,
	folder string,
	contentType string,
) error {

	// Parse the multipart form with a maximum upload size of 10 MB
	err := request.ParseMultipartForm(10 << 20) // 10 MB
	if err != nil {
		return exception.Biz("超出了文件大小限制")
	}

	// 获取上传的文件
	header := request.MultipartForm.File["file"][0]

	//文件名
	name := header.Filename
	path := folder + "/" + name

	//检查文件路径是否合法
	pathErr := DfsFileUtil.CheckPath(path)
	if pathErr != nil {
		return pathErr
	}

	//文件MD5
	md5File, err := header.Open()
	if err != nil {
		return err
	}
	defer md5File.Close()
	md5 := File.ToMd5ByReader(md5File)

	// 将 FileHeader 转为 io.Reader
	file, _ := header.Open()
	defer file.Close()

	//将文件存放到指定目录
	localFileDto, saveErr := DfsFileService.SaveToLocalFile(md5, file)
	if saveErr != nil {
		return pathErr
	}
	addErr := addDfsFile(LoginState.LoginId(), localFileDto, path, contentType)
	if addErr != nil {
		return err
	}

	//@TODO:待实现
	////开启生成缩略图线程
	//DfsFileHandleUtil.start()
	return nil
}

//    @Operation(summary = "以流的方式上传文件")
//    @PostMapping("/by_stream/{md5}")
//    @ResponseBody
//    fun byStream(request: HttpServletRequest, @PathVariable md5: String) {
//        synchronized(this.uploadingFileMap) {
//            if (this.uploadingFileMap.containsKey(md5)) {
//                throw ErrorCode.FILE_UPLOADING
//            }
//            this.uploadingFileMap[md5] = System.currentTimeMillis()
//        }
//
//        try {//保存到文件
//            val file = File(this.dataPath + "/temp/" + md5)
//
//            //文件输出流
//            FileOutputStream(file, true).use {
//                request.inputStream.transferTo(it)
////                val stream = request.inputStream
////                val data = ByteArray(64 * 1024)
////                var len: Int
////                while (stream.read(data).also { len = it } != -1) {
////                    sleep(10)
////                    it.write(data, 0, len)
////                }
//            }
//
//            //计算文件的MD5
//            val fileMd5 = file.md5
//            if (md5 != fileMd5) {
//                file.delete()
//                throw BusinessException("文件校验失败")
//            }
//
//            //将文件存放到指定目录
//            this.dfsFileService.saveToLocalFile(md5, file.inputStream())
//            file.delete()
//
//            //开启生成缩略图线程
//            DfsFileHandleUtil.start()
//        } finally {
//            synchronized(this.uploadingFileMap) {
//                this.uploadingFileMap.remove(md5)
//            }
//        }
//    }
//
//    @Operation(summary = "获取文件已经上传大小")
//    @PostMapping("/get_uploaded_size")
//    @ResponseBody
//    fun getUploadedSize(@Parameter(name = "文件的MD5") @RequestParam("md5", required = true) md5: String): Long {
//        if (this.uploadingFileMap.containsKey(md5)) {
//            throw ErrorCode.FILE_UPLOADING
//        }
//        val file = File(dataPath + "/temp/" + md5)
//        if (!file.exists()) {
//            return 0
//        }
//        return file.length()
//    }
//
//    /**
//     * 通过MD5上传
//     * @param md5 文件md5
//     * @param path 文件路径
//     */
//    @PostMapping("/by_md5")
//    @ResponseBody
//    fun byMd5(md5: String, path: String, contentType: String?) {
//
//        val localFileDto = this.localFileDao.selectByFileMd5(md5)
//            ?: throw ErrorCode.NO_EXISTS
//
//        //添加到DFS文件
//        this.addDfsFile(super.loginId, localFileDto, path, contentType)
//
//        //删除上传的临时文件
//        File(dataPath + "/temp/" + md5).delete()
//
//        //开启生成缩略图线程
//        DfsFileHandleUtil.start()
//    }
//

/**
 * 添加到DFS文件
 * @param userId 会员id
 * @param localFileDto 本地文件Dto
 * @param path DFS文件路径
 * @param fileContentType 文件类型
 */
func addDfsFile(userId int64, localFileDto dto.LocalFileDto, path string, fileContentType string) error {

	//文件名
	name := String.FileName(path)

	//上级文件夹名
	folder := String.FileParent(path)

	//获取文件夹ID
	folderId, err := DfsFileService.GetIdByFolder(userId, folder, true)
	if err != nil {
		return err
	}
	contentType := ""
	if fileContentType != "" {
		contentType = fileContentType
	} else {
		ext := String.FileExt(name)
		contentType = DfsFileUtil.DfsContentType(ext)
	}

	fileInfo, _ := os.Stat(localFileDto.Path)
	fileDto := dto.DfsFileDto{
		UserId:      userId,
		LocalId:     localFileDto.Id,
		Name:        name,
		ContentType: contentType,
		Size:        fileInfo.Size(),
		ParentId:    folderId,
	}
	return DfsFileService.AddFile(fileDto, true)
}

//}
