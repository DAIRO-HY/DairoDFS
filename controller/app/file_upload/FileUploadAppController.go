package file_upload

import (
	"DairoDFS/application"
	"DairoDFS/dao/StorageFileDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/File"
	"DairoDFS/extension/String"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/DBConnection"
	"DairoDFS/util/DfsFileHandleUtil"
	"DairoDFS/util/DfsFileUtil"
	"DairoDFS/util/LoginState"
	"net/http"
	"os"
	"sync"
)

// 文件上传Controller
//@Group:/app/file_upload

/**
 * 记录当前正在上传的文件
 * 避免同一个文件同时上传导致文件数据混乱
 * md5 -> 上传时间戳
 */
var uploadingFileMap = map[string]int64{}
var uploadingLock sync.Mutex

// 浏览器文件上传
// @Post:
func Upload(request *http.Request, folder string, contentType string) error {

	// 获取上传的文件
	header := request.MultipartForm.File["file"][0]

	//文件名
	name := header.Filename
	path := folder + "/" + name

	//检查文件路径是否合法
	DfsFileUtil.CheckPath(path)

	//文件MD5
	md5File, openErr := header.Open()
	if openErr != nil {
		return openErr
	}
	defer md5File.Close()
	md5 := File.ToMd5ByReader(md5File)

	// 将 FileHeader 转为 io.Reader
	file, _ := header.Open()
	defer file.Close()

	//将文件存放到指定目录
	storageFileDto := DfsFileService.SaveToStorageFile(md5, file)
	addDfsFile(LoginState.LoginId(), storageFileDto, path, contentType)

	//立即提交事务，否则可能导致文件处理任务获取不到数据
	DBConnection.Commit()
	DfsFileHandleUtil.NotifyWorker()
	return nil
}

// ByStream 以流的方式上传文件
// @Post:/by_stream/{md5}
func ByStream(request *http.Request, md5 string) {
	uploadingLock.Lock()
	if _, isExists := uploadingFileMap[md5]; isExists {
		uploadingLock.Unlock()
		panic(exception.FILE_UPLOADING())
	}
	this.uploadingFileMap[md5] = System.currentTimeMillis()
	uploadingLock.Unlock()

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
	//            this.dfsFileService.saveToStorageFile(md5, file.inputStream())
	//            file.delete()
	//
	//            //开启生成缩略图线程
	//            DfsFileHandleUtil.start()
	//        } finally {
	//            synchronized(this.uploadingFileMap) {
	//                this.uploadingFileMap.remove(md5)
	//            }
	//        }
}

// GetUploadedSize 获取文件已经上传大小
// md5 文件的MD5
// @Post:/get_uploaded_size
func GetUploadedSize(md5 string) int64 {
	_, isExists := uploadingFileMap[md5]
	if isExists {
		panic(exception.FILE_UPLOADING())
	}
	stat, err := os.Stat(application.TEMP_PATH + "/" + md5)
	if os.IsNotExist(err) {
		return 0
	}
	return stat.Size()
}

// stat 通过MD5上传
// md5 文件md5
// path 文件路径
// @Post:/by_md5
func ç(md5 string, path string, contentType string) {
	loginId := LoginState.LoginId()
	storageFileDto, isExists := StorageFileDao.SelectByFileMd5(md5)
	if !isExists {
		panic(exception.NO_EXISTS())
	}

	//添加到DFS文件
	addDfsFile(loginId, storageFileDto, path, contentType)

	//删除上传的临时文件
	os.Remove(application.TEMP_PATH + "/" + md5)

	//开启生成缩略图线程
	DfsFileHandleUtil.NotifyWorker()
}

/**
 * 添加到DFS文件
 * @param userId 会员id
 * @param storageFileDto 本地文件Dto
 * @param path DFS文件路径
 * @param fileContentType 文件类型
 */
func addDfsFile(userId int64, storageFileDto dto.StorageFileDto, path string, fileContentType string) {

	//文件名
	name := String.FileName(path)

	//上级文件夹名
	folder := String.FileParent(path)

	//获取文件夹ID
	folderId := DfsFileService.GetIdByFolder(userId, folder, true)
	contentType := ""
	if fileContentType != "" {
		contentType = fileContentType
	} else {
		ext := String.FileExt(name)
		contentType = DfsFileUtil.DfsContentType(ext)
	}

	fileInfo, _ := os.Stat(storageFileDto.Path)
	fileDto := dto.DfsFileDto{
		UserId:      userId,
		StorageId:   storageFileDto.Id,
		Name:        name,
		ContentType: contentType,
		Size:        fileInfo.Size(),
		ParentId:    folderId,
	}
	DfsFileService.AddFile(fileDto, true)
}
