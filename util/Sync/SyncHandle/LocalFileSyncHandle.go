package SyncHandle

import (
	"DairoDFS/dao/LocalFileDao"
	"DairoDFS/exception"
	"DairoDFS/extension/File"
	"DairoDFS/util/DBConnection"
	"DairoDFS/util/DfsFileUtil"
	"DairoDFS/util/Sync/SyncFileUtil"
	"DairoDFS/util/Sync/bean"
	"os"
)

/**
 * 本地文件数据表同步操作
 */

/**
 * 全量同步时的特殊处理
 */
func byTable(info bean.SyncServerInfo, item map[string]any) {
	//val md5 = item.path("md5").textValue()
	//
	////从本地数据库查找该文件
	//val existsLocalFile = LocalFileDao::class.bean.selectByFileMd5(md5)
	//if (existsLocalFile == null) {
	//    val tmpFilePath = SyncFileUtil.download(info, md5)
	//    val tempFileMd5 = File(tmpFilePath).md5
	//    if (md5 != tempFileMd5) {
	//        throw RuntimeException("同步的文件数据不完整，目标文件MD5:$md5，实际文件MD5(${File(tmpFilePath).length()}):$tempFileMd5")
	//    }
	//    val saveLocalPath = DfsFileUtil.localPath
	//    val saveLocalFile = File(saveLocalPath)
	//
	//    //移动文件
	//    val isMove = File(tmpFilePath).renameTo(saveLocalFile)
	//    if (!isMove) {
	//        throw RuntimeException("文件下载完成,但移动文件失败;\n文件:$tmpFilePath 移动到:$saveLocalPath 失败.")
	//    }
	//    item.put("path", saveLocalPath)
	//} else {//本机存在同样的文件,将本地记录删除，然后改用主机端同步过来的id
	//    val id = item.path("id").longValue()
	//
	//    //删除本地的数据
	//    Constant.dbService.exec("delete from local_file where id = ?", existsLocalFile.id)
	//
	//    //更换ID
	//    Constant.dbService.exec("update dfs_file set localId = ? where localId = ?", id, existsLocalFile.id)
	//    item.put("path", existsLocalFile.path)
	//}
}

/**
 * 日志同步时的特殊处理
 */
func ByLog(info *bean.SyncServerInfo, params []any) error {

	//得到文件的md5
	md5 := params[2].(string)

	//从本地数据库查找该文件
	existsLocalFile, isExists := LocalFileDao.SelectByFileMd5(md5)
	if !isExists { //本地不存在该文件,则从主机下载
		tmpFilePath, downloadErr := SyncFileUtil.Download(info, md5, 0)
		if downloadErr != nil {
			return downloadErr
		}
		tempFileMd5 := File.ToMd5(tmpFilePath)
		if md5 != tempFileMd5 {
			return exception.Biz("同步的文件数据不完整，目标文件MD5:" + md5 + "，实际文件MD5:" + tempFileMd5)
		}
		saveLocalPath, saveLocalPathErr := DfsFileUtil.LocalPath()
		if saveLocalPathErr != nil {
			return saveLocalPathErr
		}

		//移动文件
		moveErr := os.Rename(tmpFilePath, saveLocalPath)
		if moveErr != nil {
			return moveErr
		}

		//将sql语句中的参数路劲修改为本地存储的文件
		params[1] = saveLocalPath
	} else { //本机存在同样的文件,直接使用
		id := params[0].(float64) //数值类型的Json反序列化之后都是float64类型

		//删除本地的数据
		DBConnection.DBConn.Exec("delete from local_file where id = ?", existsLocalFile.Id)

		//更换本机所有本地文件ID为主机上的ID
		DBConnection.DBConn.Exec("update dfs_file set localId = ? where localId = ?", id, existsLocalFile.Id)
		params[1] = existsLocalFile.Path
	}
	return nil
}
