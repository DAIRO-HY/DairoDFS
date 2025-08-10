package file_upload

import (
	"DairoDFS/application"
	"DairoDFS/dao/DfsFileDao"
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
	"io"
	"net/http"
	"os"
	"sync"
	"time"
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
func Upload(request *http.Request, folder string, contentType string) {

	// 获取上传的文件
	header := request.MultipartForm.File["file"][0]

	//文件名
	name := header.Filename
	path := folder + "/" + name

	//检查文件路径是否合法
	File.CheckPath(path)

	//文件MD5
	md5File, openErr := header.Open()
	if openErr != nil {
		panic(openErr)
	}
	defer md5File.Close()
	md5 := File.ToMd5ByReader(md5File)

	// 将 FileHeader 转为 io.Reader
	file, _ := header.Open()
	defer file.Close()

	//将文件存放到指定目录
	storageFileDto := DfsFileService.SaveToStorageReader(md5, file, header.Size)
	addDfsFile(LoginState.LoginId(), storageFileDto, path, contentType)

	//立即提交事务，否则可能导致文件处理任务获取不到数据
	DBConnection.Commit()
	DfsFileHandleUtil.NotifyWorker()
}

// ByStream 以流的方式上传文件
// @Post:/by_stream/{md5}
func ByStream(request *http.Request, md5 string) {
	defer func() {
		uploadingLock.Lock()

		//执行结束之后移除正在上传标记
		delete(uploadingFileMap, md5)
		uploadingLock.Unlock()
	}()
	uploadingLock.Lock()
	if _, isExists := uploadingFileMap[md5]; isExists {
		uploadingLock.Unlock()
		panic(exception.FILE_UPLOADING())
	}
	uploadingFileMap[md5] = time.Now().UnixMilli()
	uploadingLock.Unlock()

	//保存文件
	tempPath := application.TEMP_PATH + "/" + md5

	//以追加的方式打开文件
	writeFile, openFileErr := os.OpenFile(tempPath, os.O_APPEND|os.O_WRONLY|os.O_CREATE, 0644)
	if openFileErr != nil {
		panic(openFileErr)
	}
	defer func() { //最终关闭流并删除临时文件
		writeFile.Close()
	}()

	//保存文件
	_, copyErr := io.Copy(writeFile, request.Body)
	if copyErr != nil { //可能与客户端中断的情况
		panic(copyErr)
		return
	}

	//计算文件的MD5
	fileMd5 := File.ToMd5(tempPath)
	if md5 != fileMd5 {
		writeFile.Close()
		os.Remove(tempPath)
		panic(exception.Biz("文件校验失败"))
	}

	//将文件存放到指定目录
	DfsFileService.SaveToStorageByFile(tempPath, md5)

	//文件上传成功，删除临时文件
	writeFile.Close()
	os.Remove(tempPath)
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
func ByMd5(md5 string, path string, contentType string) {
	loginId := LoginState.LoginId()
	storageFileDto, isExists := StorageFileDao.SelectByFileMd5(md5)
	if !isExists {

		//文件上传中发生了以外,删除临时文件重新上传
		os.Remove(application.TEMP_PATH + "/" + md5)
		panic(exception.NO_EXISTS())
	}

	//添加到DFS文件
	addDfsFile(loginId, storageFileDto, path, contentType)

	//删除上传的临时文件
	os.Remove(application.TEMP_PATH + "/" + md5)

	//开启生成缩略图线程
	DfsFileHandleUtil.NotifyWorker()
}

// 检查文件是否已经存在
// - md5 文件的md5,多个以逗号分隔
// @Post:/check_exists_by_md5
func CheckExistsByMd5(md5 string) bool {
	return DfsFileDao.CheckExistsByMd5(LoginState.LoginId(), md5)
}

// 添加到DFS文件
// userId 会员id
// storageFileDto 本地文件Dto
// path DFS文件路径
// fileContentType 文件类型
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
