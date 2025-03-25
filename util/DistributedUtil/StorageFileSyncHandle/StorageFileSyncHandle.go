package StorageFileSyncHandle

import (
	"DairoDFS/dao/StorageFileDao"
	"DairoDFS/extension/File"
	"DairoDFS/util/DfsFileUtil"
	"DairoDFS/util/DistributedUtil"
	"DairoDFS/util/DistributedUtil/SyncDownloadUtil"
	"fmt"
	"io"
	"os"
	"reflect"
	"strings"
)

// 表同步时的特殊处理
func ByTable(info *DistributedUtil.SyncServerInfo, dataMap map[string]any) {
	id := int64(dataMap["id"].(float64))
	md5 := dataMap["md5"].(string)
	path := download(info, md5, id)
	dataMap["path"] = path
}

// 日志同步时的特殊处理
func ByLog(info *DistributedUtil.SyncServerInfo, params []any) {
	id := int64(params[0].(float64)) //数值类型的Json反序列化之后都是float64类型

	//得到文件的md5
	md5 := params[2].(string)
	path := download(info, md5, id)
	params[1] = path
}

// 下载文件
// info 主机信息
// md5 文件md5
// masterStorageFileId 主机存储文件Id
func download(info *DistributedUtil.SyncServerInfo, md5 string, masterStorageFileId int64) string {

	//从本地数据库查找该文件
	existsStorageFile, isExists := StorageFileDao.SelectByFileMd5(md5)
	if !isExists { //本地不存在该文件,则从主机下载
		tmpFilePath, downloadErr := SyncDownloadUtil.Download(info, md5, 0)
		if downloadErr != nil {
			panic(downloadErr)
		}
		tempFileMd5 := File.ToMd5(tmpFilePath)
		if md5 != tempFileMd5 {
			panic("同步的文件数据不完整，目标文件MD5:" + md5 + "，实际文件MD5:" + tempFileMd5)
		}

		//获取文件信息
		stat, _ := os.Stat(tmpFilePath)
		saveLocalPath := DfsFileUtil.LocalPath(stat.Size())

		//移动文件
		moveFile(tmpFilePath, saveLocalPath)

		//将sql语句中的参数路劲修改为本地存储的文件
		return saveLocalPath
	} else { //本机存在同样的文件,直接使用

		//删除本地的数据
		info.DbTx().Exec("delete from storage_file where id = ?", existsStorageFile.Id)

		//更换本机所有本地文件ID为主机上的ID
		info.DbTx().Exec("update dfs_file set storageId = ? where storageId = ?", masterStorageFileId, existsStorageFile.Id)
		return existsStorageFile.Path
	}
}

// 移动文件到数据目录
func moveFile(src string, target string) {

	fmt.Println("-->01")

	//移动文件
	moveErr := os.Rename(src, target)
	if moveErr == nil {
		return
	}
	fmt.Println("-->1:", reflect.TypeOf(moveErr))

	//不同的盘符之间不能使用Rename操作
	if !strings.HasSuffix(moveErr.Error(), "The system cannot move the file to a different disk drive.") {
		fmt.Println("-->2:" + moveErr.Error())
		panic(moveErr)
	}
	fmt.Println("-->3:" + moveErr.Error())
	source, _ := os.Open(src)
	fmt.Println("-->4")
	defer source.Close()
	targetFile, _ := os.Create(target)
	defer targetFile.Close()
	fmt.Println("-->5")
	if _, err := io.Copy(targetFile, source); err != nil {
		os.Remove(target)
		panic(err)
	}
	fmt.Println("-->6")

	//确保文件已经写入到了磁盘，避免突然断电导致文件数据丢失
	if err := targetFile.Sync(); err != nil {
		os.Remove(target)
		panic(err)
	}
	fmt.Println("-->7")
	source.Close()
	os.Remove(src)
	fmt.Println("-->8")
}
