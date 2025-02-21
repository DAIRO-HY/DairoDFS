package profile

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/controller/app/profile/form"
	"DairoDFS/extension/String"
	"DairoDFS/util/DistributedUtil/SyncByLog"
	"strings"
)

//系统配置
//@Group:/app/profile

/**
 * 页面初始化
 */
//@Html:.html
func Html() {}

/**
 * 页面数据初始化
 */
//@Post:/init
func Init() form.ProfileForm {
	inForm := form.ProfileForm{}
	systemConfig := SystemConfig.Instance()
	inForm.OpenSqlLog = systemConfig.OpenSqlLog
	inForm.HasReadOnly = systemConfig.IsReadOnly
	inForm.UploadMaxSize = systemConfig.UploadMaxSize
	inForm.Folders = strings.Join(systemConfig.SaveFolderList, "\n")
	inForm.SyncDomains = strings.Join(systemConfig.SyncDomains, "\n")
	inForm.Token = systemConfig.Token
	inForm.TrashTimeout = systemConfig.TrashTimeout
	inForm.DeleteStorageTimeout = systemConfig.DeleteStorageTimeout
	return inForm
}

/**
 * 页面初始化
 */
//@Post:/update
func Update(form form.ProfileForm) error {
	folders := strings.Split(form.Folders, "\n")
	systemConfig := SystemConfig.Instance()
	saveFolderList := []string{}

	for _, it := range folders {
		//val folderFile = File(it)
		//if (!folderFile.exists()) {
		//    throw BusinessException.addFieldError("folders", "目录:${it}不存在")
		//}
		//saveFolderList.add(folderFile.absolutePath)
		saveFolderList = append(saveFolderList, it)
	}

	systemConfig.SaveFolderList = saveFolderList
	systemConfig.UploadMaxSize = form.UploadMaxSize
	systemConfig.OpenSqlLog = form.OpenSqlLog
	systemConfig.IsReadOnly = form.HasReadOnly
	systemConfig.TrashTimeout = form.TrashTimeout
	systemConfig.DeleteStorageTimeout = form.DeleteStorageTimeout

	if form.SyncDomains == "" {
		systemConfig.SyncDomains = []string{}
	} else {

		//配置的同步域名
		syncDomains := strings.Split(form.SyncDomains, "\n")
		systemConfig.SyncDomains = syncDomains
	}

	//@TODO:等待完成
	//SyncByLog.init()
	SystemConfig.Save()

	//重新开启监听
	SyncByLog.ListenAll()
	return nil
}

/**
 * 切换token
 */
//@Post:/app/profile/make_token
func MakeToken() {
	systemConfig := SystemConfig.Instance()
	systemConfig.Token = String.MakeRandStr(32)
	SystemConfig.Save()
}
