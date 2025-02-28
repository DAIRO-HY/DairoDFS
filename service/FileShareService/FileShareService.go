package FileShareService

import (
	"DairoDFS/controller/app/files/form"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/ShareDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/Number"
	"DairoDFS/service/DfsFileService"
	"strings"
	"time"
)

/**
 * 文件分享操作Service
 */

/**
 * 分享文件
 * @param form 分享表单
 */
func Share(userId int64, inForm form.ShareForm) int64 {
	if len(inForm.Names) == 0 {
		panic(exception.Biz("请选择要分享的路径"))
	}

	//得到缩略图ID
	thumbId := findThumb(userId, inForm.Folder, inForm.Names)

	//判断这是不是只是一个文件夹
	folderFlag := isFolder(userId, inForm.Folder, inForm.Names)

	//获取分享文件的标题
	title := getTitle(inForm.Names)
	shareDto := dto.ShareDto{
		Title:      title,
		UserId:     userId,
		EndDate:    inForm.EndDateTime,
		Pwd:        inForm.Pwd,
		Names:      strings.Join(inForm.Names, "|"),
		Folder:     inForm.Folder,
		FolderFlag: folderFlag,
		Thumb:      thumbId,
		FileCount:  len(inForm.Names),
		Date:       time.Now().UnixMilli(),
		Id:         Number.ID(),
	}
	ShareDao.Add(shareDto)
	return shareDto.Id
}

/**
 * 去查找缩略图
 */
func findThumb(userId int64, folder string, names []string) int64 {

	//得到分享的父文件夹ID
	folderId := DfsFileService.GetIdByFolder(userId, folder, false)
	if folderId == 0 {
		panic(exception.NO_FOLDER())
	}

	//取出当前目录下的所有文件，用来查找缩略图
	subFiles := DfsFileDao.SelectSubFile(userId, folderId)

	//文件名对应的文件信息
	name2file := make(map[string]dto.DfsFileDto)
	for _, it := range subFiles {
		if it.HasThumb {
			name2file[it.Name] = it
		}
	}

	//查找缩略图
	for _, name := range names {
		thumbFile, isExists := name2file[name]
		if isExists { //如果有缩略图
			return thumbFile.Id
		}
	}
	return 0
}

/**
 * 判断这是不是只是一个文件夹
 */
func isFolder(userId int64, folder string, names []string) bool {
	if len(names) > 1 {
		return false
	}

	//得到分享的文件ID
	fileId := DfsFileService.GetIdByFolder(userId, folder+"/"+names[0], false)
	if fileId == 0 {
		panic(exception.NO_FOLDER())
	}
	fileDto, _ := DfsFileDao.SelectOne(fileId)
	if fileDto.StorageId == 0 { //这是一个文件夹
		return true
	}
	return false
}

/**
 * 获取分享文件的标题
 */
func getTitle(names []string) string {
	if len(names) == 1 {
		return names[0]
	}
	return names[0] + "等多个文件"
}
