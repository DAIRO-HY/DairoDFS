package folder_selector

import (
	"DairoDFS/controller/app/folder_selector/form"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/LoginState"
)

/**
 * 选择文件夹Controller
 */

// 获取文件夹结构
// @Post:/app/folder_selector/get_list
func GetList(folder string) []form.FolderForm {
	loginId := LoginState.LoginId()
	folderId, folderIdErr := DfsFileService.GetIdByFolder(loginId, folder, false)
	if folderIdErr != nil {
		return []form.FolderForm{}
	}
	list := make([]form.FolderForm, 0)
	for _, it := range DfsFileDao.SelectSubFile(loginId, folderId) {
		if it.StorageId > 0 {
			continue
		}
		outForm := form.FolderForm{
			Name: it.Name,
			//Size : Number.ToDataSize(it.Size),
			//Date : it.Date?.format(),
			//FileFlag : it.isFile,
		}
		list = append(list, outForm)
	}
	return list
}
