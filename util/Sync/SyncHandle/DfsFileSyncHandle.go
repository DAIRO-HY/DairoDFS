package SyncHandle

import (
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/Sync/bean"
)

/**
 * DFS文件同步之前，本地文件的一些操作
 */
func Handle(info bean.SyncServerInfo, dfsFile *dto.DfsFileDto) error {
	if dfsFile.DeleteDate > 0 { //该文件已经被删除，不用做任何处理 @TODO:等待验证
		return nil
	}
	if dfsFile.IsHistory { //这是一个历史文件
		return nil
	}
	existsDfsFile, isExists := DfsFileDao.SelectByParentIdAndName(dfsFile.UserId, dfsFile.ParentId, dfsFile.Name)
	if !isExists { //文件不存在时，不做任何处理
		return nil
	}
	//该分机端DFS文件已经存在的话，要做一些特殊处理
	if dfsFile.LocalId == 0 && existsDfsFile.LocalId == 0 {
		// 如果都是文件夹，则保留主机端的文件夹，具体步骤如下
		// 1、将本地的DFS文件夹下的所有文件及文件夹全部移动到主机端的文件夹下
		// 2、删除本地文件夹（这可能会导致已经分享出去的连接失效）
		_, err := DBUtil.DBConn.Exec("update dfs_file set parentId = ? where parentId = ?", dfsFile.Id, existsDfsFile.Id)
		if err != nil {
			//@TODO:待确认
		}
		_, err = DBUtil.DBConn.Exec("delete from dfs_file where id = ?", existsDfsFile.Id)
		if err != nil {
			//@TODO:待确认
		}
	} else if dfsFile.LocalId > 0 && existsDfsFile.LocalId > 0 {
		// 如果都是文件，则保留最新的一个文件，将日期比较老的文件加入到历史记录
		if dfsFile.Id > existsDfsFile.Id { //当前主机端的文件比较新，则将本地的文件设置为历史文件
			_, err := DBUtil.DBConn.Exec("update dfs_file set isHistory = 1 where id = ?", existsDfsFile.Id)
			if err != nil {
				//@TODO:待确认
			}
		} else { //本地的文件比较新，则将主机端的文件设置为历史文件
			dfsFile.IsHistory = true
		}
	} else { //主机端和分几端，一个时文件，一个是文件夹，无法同步

		//得到用户信息
		user, _ := UserDao.SelectOne(dfsFile.UserId)

		//得到发生错误的文件路径
		path, _ := DfsFileService.GetPathById(dfsFile.Id)
		return exception.Biz("同步失败，服务器：${info.url}  用户名：" + user.Name + "  路径：" + path + " 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请重命名当前或者服务器端 $path 的文件名。")
	}
	return nil
}

func HandleBySyncLog(info *bean.SyncServerInfo, params []any) string {

	////用户文件id
	//val id = params[0].toString().toLong()
	//
	////用户id
	//val userId = params[1].toString().toLong()
	//
	////父级文件夹id
	//val parentId = params[2].toString().toLong()
	//
	////文件（夹）名
	//val name = params[3] as String
	//val dfsFile = DfsFileDao::class.bean.selectByParentIdAndName(userId, parentId, name)
	//if (dfsFile == null) {//文件不存在时，不做任何处理
	//    return null
	//}
	//
	////本地存储文件id
	//val localId = params[6].toString().toLong()
	//if (localId == 0L && dfsFile.localId == 0L) {
	//    // 如果都是文件夹，则保留主机端的文件夹，具体步骤如下
	//    // 1、将本地的DFS文件夹下的所有文件及文件夹全部移动到主机端的文件夹下
	//    // 2、删除本地文件夹（这可能会导致已经分享出去的连接失效）
	//    Constant.dbService.exec("update dfs_file set parentId = ? where parentId = ?", id, dfsFile.id)
	//    Constant.dbService.exec("delete from dfs_file where id = ?", dfsFile.id)
	//} else if (localId > 0 && dfsFile.localId!! > 0) {
	//    // 如果都是文件，则保留最新的一个文件，将日期比较老的文件加入到历史记录
	//    if (id > dfsFile.id!!) {//当前主机端的文件比较新，则将本地的文件设置为历史文件
	//        Constant.dbService.exec("update dfs_file set isHistory = 1 where id = ?", dfsFile.id)
	//    } else {//本地的文件比较新，则将主机端的文件设置为历史文件
	//        //该日志执行成功之后要执行的SQL语句
	//        val afterSql = "update dfs_file set isHistory = 1 where id = $id"
	//        return afterSql
	//    }
	//} else {
	//
	//    //得到用户信息
	//    val user = UserDao::class.bean.selectOne(userId)
	//
	//    //得到发生错误的文件路径
	//    val path = DfsFileService::class.bean.getPathById(dfsFile.id!!)
	//    throw RuntimeException("同步失败，服务器：${info.url}  用户名：${user?.name}  路径：$path 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请重命名当前或者服务器端 $path 的文件名。")
	//}
	return ""
}

//}
