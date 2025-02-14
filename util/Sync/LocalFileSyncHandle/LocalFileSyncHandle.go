package StorageFileSyncHandle

import (
	"DairoDFS/dao/StorageFileDao"
	"DairoDFS/exception"
	"DairoDFS/extension/File"
	"DairoDFS/util/DfsFileUtil"
	"DairoDFS/util/Sync/SyncDownloadUtil"
	"DairoDFS/util/Sync/bean"
	"os"
)

// 表同步时的特殊处理
func ByTable(info *bean.SyncServerInfo, dataMap map[string]any) error {
	id := int64(dataMap["id"].(float64))
	md5 := dataMap["md5"].(string)
	path, err := download(info, md5, id)
	if err != nil {
		return err
	}
	dataMap["path"] = path
	return nil
}

// 日志同步时的特殊处理
func ByLog(info *bean.SyncServerInfo, params []any) error {
	id := int64(params[0].(float64)) //数值类型的Json反序列化之后都是float64类型

	//得到文件的md5
	md5 := params[2].(string)
	path, err := download(info, md5, id)
	if err != nil {
		return err
	}
	params[1] = path
	return nil
}

// 下载文件
// info 主机信息
// md5 文件md5
// masterStorageFileId 主机存储文件Id
func download(info *bean.SyncServerInfo, md5 string, masterStorageFileId int64) (string, error) {

	//从本地数据库查找该文件
	existsStorageFile, isExists := StorageFileDao.SelectByFileMd5(md5)
	if !isExists { //本地不存在该文件,则从主机下载
		tmpFilePath, downloadErr := SyncDownloadUtil.Download(info, md5, 0)
		if downloadErr != nil {
			return "", downloadErr
		}
		tempFileMd5 := File.ToMd5(tmpFilePath)
		if md5 != tempFileMd5 {
			return "", exception.Biz("同步的文件数据不完整，目标文件MD5:" + md5 + "，实际文件MD5:" + tempFileMd5)
		}
		saveLocalPath, saveLocalPathErr := DfsFileUtil.LocalPath()
		if saveLocalPathErr != nil {
			return "", saveLocalPathErr
		}

		//移动文件
		moveErr := os.Rename(tmpFilePath, saveLocalPath)
		if moveErr != nil {
			return "", moveErr
		}

		//将sql语句中的参数路劲修改为本地存储的文件
		return saveLocalPath, nil
	} else { //本机存在同样的文件,直接使用

		//删除本地的数据
		info.DbTx().Exec("delete from storage_file where id = ?", existsStorageFile.Id)

		//更换本机所有本地文件ID为主机上的ID
		info.DbTx().Exec("update dfs_file set storageId = ? where storageId = ?", masterStorageFileId, existsStorageFile.Id)
		return existsStorageFile.Path, nil
	}
}
