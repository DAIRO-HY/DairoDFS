package DfsFileService

import (
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/StorageFileDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/File"
	"DairoDFS/extension/Number"
	"DairoDFS/extension/String"
	"DairoDFS/util/DfsFileUtil"
	"bytes"
	"fmt"
	"io"
	"os"
	"strings"
	"time"
)

// 往数据库添加一条数据
func Add(fileDto dto.DfsFileDto) {
	if fileDto.IsFile() { // 获取文件后缀名
		fileDto.Ext = strings.ToLower(String.FileExt(fileDto.Name))
	}
	DfsFileDao.Add(fileDto)
}

// 添加一个文件或文件夹
// fileDto 文件信息
// isOverWrite 是否覆盖，true时，将原来的文件设置为历史文件
func AddFile(fileDto dto.DfsFileDto, isOverWrite bool) {
	if fileDto.StorageId == 0 {
		panic(exception.Biz("本地存储文件ID不能为空"))
	}
	existDto, isExists := DfsFileDao.SelectByParentIdAndName(fileDto.UserId, fileDto.ParentId, fileDto.Name)
	if isExists {
		if existDto.IsFolder() {
			panic(exception.Biz("已存在同名文件夹:" + fileDto.Name))
		}
		if existDto.StorageId == fileDto.StorageId { //同一个文件,直接成功
			return
		}
		if !isOverWrite { //文件已经存在,在不允许覆盖的情况下,直接报错义务错误
			panic(exception.EXISTS_FILE(fileDto.Name))
		}
	}
	fileDto.Date = time.Now().UnixMilli()
	fileDto.Id = Number.ID()
	if isExists && isOverWrite { //将已经存在的文件标记为历史版本
		DfsFileDao.SetHistory(existDto.Id)
	}

	//添加文件
	Add(fileDto)
}

// 添加文件夹
func AddFolder(folderDto dto.DfsFileDto) {
	_, isExists := DfsFileDao.SelectByParentIdAndName(folderDto.UserId, folderDto.ParentId, folderDto.Name)
	if isExists {
		panic(exception.BizCode(1001, "文件或文件夹已经存在"))
	}
	folderDto.StorageId = 0
	folderDto.Date = time.Now().UnixMilli()
	Add(folderDto)
}

/**
 * 通过路径获取文件夹ID
 * @param userId 用户ID
 * @param folder 文件夹路径
 * @param isCreate 文件夹不存在时是否创建
 * @return 文件夹ID
 */
func GetIdByFolder(userId int64, folder string, isCreate bool) int64 {
	var folderId = DfsFileDao.SelectIdByPath(userId, folder)
	if folderId != 0 {
		return folderId
	}
	if isCreate {
		id := Mkdirs(userId, folder)
		folderId = id
	}
	return folderId
}

/**
 * 复制目录
 * @param userId 用户ID
 * @param sourcePaths 要复制的目录数组
 * @param targetFolder 要复制到的目标文件夹目录
 */
func Copy(userId int64, sourcePaths []string, targetFolder string, isOverWrite bool) {
	sourceToTargetMap := map[string]string{}
	for _, it := range sourcePaths {

		//复制的源路径
		sourcePath := it

		//复制的目标路径
		var targetPath = targetFolder + "/" + String.FileName(it)

		if sourcePath == targetPath { //源路径和目标路径一样时,在目标文件名加上编号
			newName := makeNameNo(userId, targetPath)
			targetPath = targetFolder + "/" + newName
		}

		recursionMakeSourceToTargetMap(userId, sourcePath, targetPath, sourceToTargetMap)
	}
	for sourcePath, targetPath := range sourceToTargetMap {
		fileId := DfsFileDao.SelectIdByPath(userId, sourcePath)
		fileDto, _ := DfsFileDao.SelectOne(fileId)
		if fileDto.IsFolder() { //源目录是一个文件夹
			Mkdirs(userId, targetPath)
		} else {
			folderId := GetIdByFolder(userId, String.FileParent(targetPath), true)
			fileName := String.FileName(targetPath)
			createFileDto := dto.DfsFileDto{
				ParentId:    folderId,
				Name:        fileName,
				StorageId:   fileDto.StorageId,
				Size:        fileDto.Size,
				ContentType: fileDto.ContentType,
				UserId:      fileDto.UserId,
				Date:        fileDto.Date,
			}
			AddFile(createFileDto, isOverWrite)
		}
	}
}

/**
 * 同一个文件夹下复制时,为新的文件或文件夹加上编号
 * 例: test.zip  ==>  test(1).zip
 * @param userId 用户id
 * @param targetPath 目标目录
 * @return 新的文件名
 */
func makeNameNo(userId int64, targetPath string) string {

	//得到父级文件夹id
	parentId := GetIdByFolder(userId, String.FileParent(targetPath), false)

	name := String.FileName(targetPath)
	var startName string
	var endNameName string
	lastDotIndex := strings.LastIndex(name, ".")
	if lastDotIndex != -1 { //路径包含点
		existFileDto, _ := DfsFileDao.SelectByParentIdAndName(userId, parentId, name)
		if existFileDto.IsFile() {
			startName = name[:lastDotIndex]
			endNameName = name[lastDotIndex:]
		} else {
			startName = name
			endNameName = ""
		}
	} else {
		startName = name
		endNameName = ""
	}
	for i := 0; i < 10000; i++ {
		newName := fmt.Sprintf("%s(%d)%s", startName, i, endNameName)
		if DfsFileDao.SelectIdByParentIdAndName(userId, parentId, newName) == 0 {
			return newName
		}
	}
	panic(exception.Biz("目录" + targetPath + "已存在"))
}

/**
 * 移动目录
 * @param userId 用户ID
 * @param sourcePaths 要复制的目录数组
 * @param targetFolder 要复制到的目标文件夹目录
 */
func Move(userId int64, sourcePaths []string, targetFolder string, isOverWrite bool) {
	sourceToTargetMap := map[string]string{}
	for _, it := range sourcePaths {

		//复制的源路径
		sourcePath := it

		//复制的目标路径
		targetPath := targetFolder + "/" + String.FileName(it)
		if strings.HasSuffix(targetFolder+"/", it+"/") {
			panic(exception.Biz("不能移动文件夹到子文件夹下"))
		}
		if sourcePath == targetPath { //同一个文件路径无需操作
			continue
		}
		recursionMakeSourceToTargetMap(userId, sourcePath, targetPath, sourceToTargetMap)
	}

	//用来记录移动完成后要删除的文件夹
	afterDeleteFolderList := make([]dto.DfsFileDto, 0)
	for sourcePath, targetPath := range sourceToTargetMap {
		fileId := DfsFileDao.SelectIdByPath(userId, sourcePath)
		sourceFileDto, _ := DfsFileDao.SelectOne(fileId)
		if sourceFileDto.IsFolder() { //源目录是一个文件夹
			Mkdirs(userId, targetPath)
			afterDeleteFolderList = append(afterDeleteFolderList, sourceFileDto)
		} else {
			folderId := GetIdByFolder(userId, String.FileParent(targetPath), true)
			existFileDto, isExists := DfsFileDao.SelectByParentIdAndName(userId, folderId, sourceFileDto.Name)
			if !isExists { //文件不存在时,移动文件包括版本记录
				sourceFileDto.ParentId = folderId

				fileName := String.FileName(targetPath)
				sourceFileDto.Name = fileName
				DfsFileDao.Move(sourceFileDto)
			} else { //目标文件已经存在
				if existFileDto.IsFolder() { //移动的对象一个时文件一个是文件夹,禁止移动
					panic(exception.EXISTS(targetPath))
				}
				if !isOverWrite {
					panic(exception.EXISTS(targetPath))
				}

				fileName := String.FileName(targetPath)
				sourceFileDto.ParentId = folderId
				sourceFileDto.Name = fileName
				DfsFileDao.Move(sourceFileDto)

				//将已经存在的文件标记为历史版本
				DfsFileDao.SetHistory(existFileDto.Id)
			}
		}
	}
	for _, it := range afterDeleteFolderList { //删除源文件夹,不能在移动的途中删除文件夹,否则导致无法找到要移动的文件
		DfsFileDao.Delete(it.Id)
	}
}

/**
 * 文件重命名
 */
func Rename(userId int64, sourcePath string, name string) {

	//获取源目录文件id
	fileId := DfsFileDao.SelectIdByPath(userId, sourcePath)
	if fileId == 0 {
		panic(exception.Biz("移动源目录不存在"))
	}
	fileDto, _ := DfsFileDao.SelectOne(fileId)
	existFileDto, isExists := DfsFileDao.SelectByParentIdAndName(userId, fileDto.ParentId, name)
	if isExists && existFileDto.Id != fileId { //existFileDto.id != fileId判断是否同一个文件,有可能仅仅时将文件名大小写转换的可能
		panic(exception.Biz("路径[" + String.FileParent(sourcePath) + "/" + name + "]已存在"))
	}
	fileDto.Name = name
	DfsFileDao.Move(fileDto)
}

/**
 * 递归整理所有要复制或移动的源路径对应的目标路径(源路径 => 目标路径)
 * @param userId 用户ID
 * @param sourcePath 复制的源目录
 * @param targetPath 复制到的目标目录
 */
func recursionMakeSourceToTargetMap(userId int64, sourcePath string, targetPath string, sourceToTargetMap map[string]string) {
	sourceToTargetMap[sourcePath] = targetPath

	fileId := DfsFileDao.SelectIdByPath(userId, sourcePath)
	if fileId == 0 {
		panic(exception.Biz("文件路径:[" + sourcePath + "]不存在"))
	}
	fileDto, _ := DfsFileDao.SelectOne(fileId)
	if fileDto.IsFolder() { //这是一个文件夹
		for _, it := range DfsFileDao.SelectSubFileIdAndName(userId, fileId) {
			subSourcePath := sourcePath + "/" + it.Name
			subTargetPath := targetPath + "/" + it.Name
			recursionMakeSourceToTargetMap(userId, subSourcePath, subTargetPath, sourceToTargetMap)
		}
	}
}

/**
 * 删除文件夹或者文件
 * @param userId 用户ID
 * @param path 文件夹路径
 */
func SetDelete(userId int64, path string) {
	dfsFileId := DfsFileDao.SelectIdByPath(userId, path)
	if dfsFileId == 0 {
		panic(exception.Biz("找不到指定目录:" + path))
	}
	dsfFileDto, _ := DfsFileDao.SelectOne(dfsFileId)
	DfsFileDao.SetDelete(dsfFileDto.Id, time.Now().UnixMilli())
}

// 通过id删除
// userId 用户ID
// ids id列表
func DeleteById(userId int64, id int64) {
	dsfFileDto, _ := DfsFileDao.SelectOne(id)
	if dsfFileDto.UserId != userId {
		panic(exception.NOT_ALLOW())
	}
	DfsFileDao.SetDelete(dsfFileDto.Id, time.Now().UnixMilli())
}

/**
 * 创建文件夹
 * @param userId 用户ID
 * @param path 文件夹路径
 */
func Mkdirs(userId int64, path string) int64 {
	var lastFolderId int64 = 0

	//用来标记,以后所有文件夹都需要创建
	var isCreatModel = false
	nameList := File.ToSubNames(path)

	//记录当前文件路径
	var curPathSB = ""
	for _, it := range nameList {
		curPathSB += "/" + it
		if !isCreatModel {
			folderDto, isExists := DfsFileDao.SelectByParentIdAndName(userId, lastFolderId, it)
			if isExists { //父级文件夹已经存在
				if folderDto.IsFile() {
					panic(exception.Biz(curPathSB + "是一个文件,不允许创建文件夹"))
				}
				lastFolderId = folderDto.Id
				continue
			}

			//标记往后的文件夹都需要创建
			isCreatModel = true
		}
		createFolderDto := dto.DfsFileDto{
			Id:       Number.ID(),
			UserId:   userId,
			Name:     it,
			ParentId: lastFolderId,
			Size:     0,
		}
		AddFolder(createFolderDto)
		lastFolderId = createFolderDto.Id
	}
	return lastFolderId
}

/**
 * 从垃圾箱还原文件
 * @param userId 用户ID
 * @param ids 要删除的文件ID
 */
func TrashRecover(userId int64, ids []int64) {
	for _, it := range ids {
		fileDto, _ := DfsFileDao.SelectOne(it)
		if fileDto.UserId != userId {
			panic(exception.NOT_ALLOW())
		}
		_, isExists := DfsFileDao.SelectByParentIdAndName(userId, fileDto.ParentId, fileDto.Name)
		if !isExists { //目标文件不存在,直接将文件的删除日期即可
			DfsFileDao.SetNotDelete(it)
		} else { //目标路径已经存在
			path := GetPathById(it)
			panic(exception.EXISTS_FILE(path))
		}
	}
}

/**
 * 通过文件ID获取文件全路径
 * @param id 文件id
 * @return 文件全路径
 */
func GetPathById(id int64) string {
	pathSB := ""
	var tempId = id
	for {
		fileDto, isExists := DfsFileDao.SelectOne(tempId)
		if !isExists {
			panic(exception.NO_EXISTS())
		}
		if fileDto.ParentId == 0 {
			break
		}
		pathSB = "/" + fileDto.Name + pathSB
		tempId = fileDto.ParentId
	}
	return pathSB
}

/**
 * 分享文件转存
 * @param shareUserId 分享用户ID
 * @param userId 用户ID
 * @param sourcePaths 要转存的路径列表
 * @param targetFolder 要复制到的目标文件夹目录
 */
func ShareSaveTo(shareUserId int64, userId int64, sourcePaths []string, targetFolder string, isOverWrite bool) {
	sourceToTargetMap := map[string]string{}
	for _, it := range sourcePaths {

		//复制的目标路径
		targetPath := targetFolder + "/" + String.FileName(it)
		recursionMakeSourceToTargetMap(userId, it, targetPath, sourceToTargetMap)
	}
	for sourcePath, targetPath := range sourceToTargetMap {

		//获取分享文件的ID
		fileId := DfsFileDao.SelectIdByPath(shareUserId, sourcePath)
		fileDto, _ := DfsFileDao.SelectOne(fileId)
		if fileDto.IsFolder() { //源目录是一个文件夹
			Mkdirs(userId, targetPath)
		} else {
			folderId := GetIdByFolder(userId, String.FileParent(targetPath), true)
			fileName := String.FileName(targetPath)
			createFileDto := dto.DfsFileDto{
				ParentId:    folderId,
				Name:        fileName,
				StorageId:   fileDto.StorageId,
				Size:        fileDto.Size,
				ContentType: fileDto.ContentType,
				UserId:      fileDto.UserId,
				Date:        fileDto.Date,
			}
			AddFile(createFileDto, isOverWrite)
		}
	}
}

/**
 * 保存文件到本地磁盘
 * @param data 要保存的数据
 */
func SaveToStorageByData(data []byte) dto.StorageFileDto {
	md5 := File.ToMd5ByBytes(data)
	return SaveToStorageReader(md5, bytes.NewBuffer(data), int64(len(data)))
}

/**
 * 保存文件到本地磁盘
 * @param md5 文件md5
 * @param iStream 文件流
 */
func SaveToStorageByFile(path string, md5 string) dto.StorageFileDto {
	file, err := os.Open(path)
	if err != nil {
		panic(err)
	}
	if md5 == "" { //如果调用的地方没有传递md5
		md5 = File.ToMd5(path)
	}
	stat, _ := os.Stat(path)
	return SaveToStorageReader(md5, file, stat.Size())
}

/**
 * 保存文件到本地磁盘
 * @param md5 文件md5
 * @param iStream 文件流
 * @param size 文件大小
 */
func SaveToStorageReader(md5 string, reader io.Reader, size int64) dto.StorageFileDto {
	existsStorageDto, isExists := StorageFileDao.SelectByFileMd5(md5)
	if isExists { //该文件已经存在
		return existsStorageDto
	}

	//获取本地文件存储路径
	localPath := DfsFileUtil.LocalPath(size)

	// 打开文件（如果不存在则创建）
	file, createErr := os.Create(localPath) // 如果文件已存在，它将被覆盖
	if createErr != nil {
		panic(exception.Biz("创建文件失败：" + createErr.Error()))
	}
	defer file.Close()

	//将文件保存到指定目录
	if _, saveErr := io.Copy(file, reader); saveErr != nil {
		errMsg := saveErr.Error()
		if errMsg == "There is not enough space on the disk." {
			panic(exception.Biz("保存文件失败：磁盘空间不足"))
		}
		panic(exception.Biz("保存文件失败：" + errMsg))
	}

	//确保文件已经写入到了磁盘，避免突然断电导致文件数据丢失
	if err := file.Sync(); err != nil {
		panic(exception.Biz("保存文件失败：" + err.Error()))
	}
	addStorageFileDto := dto.StorageFileDto{
		Path: localPath,
		Md5:  md5,
		Id:   Number.ID(),
	}
	StorageFileDao.Add(addStorageFileDto)
	return addStorageFileDto
}
