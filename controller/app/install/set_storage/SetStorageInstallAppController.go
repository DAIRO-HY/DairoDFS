package set_storage

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/controller/app/install/magick"
)

/**
 * 设置存储目录
 */
//@Group:/app/install/set_storage

// @Get:
// @Html:app/install/set_storage.html
func Html() {

	//清除上一步的缓存
	magick.Recycle()
}

/**
 * 设置存储目录
 */
//@Post:/set
func Set(path []string) {
	systemConfig := SystemConfig.Instance()
	systemConfig.SaveFolderList = path
	SystemConfig.Save()
}
