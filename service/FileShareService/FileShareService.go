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
func Share(userId int64, inForm form.ShareForm) (int64, error) {
	if len(inForm.Names) == 0 {
		return 0, exception.Biz("请选择要分享的路径")
	}

	//得到缩略图ID
	thumbId, err := findThumb(userId, inForm.Folder, inForm.Names)
	if err != nil {
		return 0, err
	}

	//判断这是不是只是一个文件夹
	folderFlag, err := isFolder(userId, inForm.Folder, inForm.Names)
	if err != nil {
		return 0, err
	}

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
	return shareDto.Id, nil
}

/**
 * 去查找缩略图
 */
func findThumb(userId int64, folder string, names []string) (int64, error) {

	//得到分享的父文件夹ID
	folderId, err := DfsFileService.GetIdByFolder(userId, folder, false)
	if err != nil {
		return 0, err
	}
	if folderId == 0 {
		return 0, exception.NO_FOLDER()
	}

	//取出当前目录下的所有文件，用来查找缩略图
	subFiles := DfsFileDao.SelectSubFile(userId, folderId)

	//文件名对应的文件信息
	name2file := make(map[string]dto.DfsFileThumbDto)
	for _, it := range subFiles {
		if it.HasThumb {
			name2file[it.Name] = it
		}
	}

	//查找缩略图
	for _, name := range names {
		thumbFile, isExists := name2file[name]
		if isExists { //如果有缩略图
			return thumbFile.Id, nil
		}
	}
	return 0, nil
}

/**
 * 判断这是不是只是一个文件夹
 */
func isFolder(userId int64, folder string, names []string) (bool, error) {
	if len(names) > 1 {
		return false, nil
	}

	//得到分享的文件ID
	fileId, err := DfsFileService.GetIdByFolder(userId, folder+"/"+names[0], false)
	if err != nil {
		return false, err
	}
	if fileId == 0 {
		return false, exception.NO_FOLDER()
	}
	fileDto, _ := DfsFileDao.SelectOne(fileId)
	if fileDto.LocalId == 0 { //这是一个文件夹
		return true, nil
	}
	return false, nil
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
