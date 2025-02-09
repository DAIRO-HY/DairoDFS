package SyncHandle

import (
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/DBConnection"
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
	existsFile, isExists := DfsFileDao.SelectByParentIdAndName(dfsFile.UserId, dfsFile.ParentId, dfsFile.Name)
	if !isExists { //文件不存在时，不做任何处理
		return nil
	}
	//该分机端DFS文件已经存在的话，要做一些特殊处理
	if dfsFile.LocalId == 0 && existsFile.LocalId == 0 {
		// 如果都是文件夹，则保留主机端的文件夹，具体步骤如下
		// 1、将本地的DFS文件夹下的所有文件及文件夹全部移动到主机端的文件夹下
		// 2、删除本地文件夹（这可能会导致已经分享出去的连接失效）
		if _, err := DBConnection.DBConn.Exec("update dfs_file set parentId = ? where parentId = ?", dfsFile.Id, existsFile.Id); err != nil {
			return err
		}
		if _, err := DBConnection.DBConn.Exec("delete from dfs_file where id = ?", existsFile.Id); err != nil {
			return err
		}
	} else if dfsFile.LocalId > 0 && existsFile.LocalId > 0 {
		// 如果都是文件，则保留最新的一个文件，将日期比较老的文件加入到历史记录
		if dfsFile.Id > existsFile.Id { //当前主机端的文件比较新，则将本地的文件设置为历史文件
			if _, err := DBConnection.DBConn.Exec("update dfs_file set isHistory = 1 where id = ?", existsFile.Id); err != nil {
				return err
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

// @TODO:应该开启事务,防止数据不完整
func HandleBySyncLog(info *bean.SyncServerInfo, params []any) (string, error) {

	//用户文件id
	id := int64(params[0].(float64))

	//用户id
	userId := int64(params[1].(float64))

	//父级文件夹id
	parentId := int64(params[2].(float64))

	//文件（夹）名
	name := params[3].(string)

	//存储文件id
	localId := int64(params[6].(float64))

	existsFile, isExists := DfsFileDao.SelectByParentIdAndName(userId, parentId, name)
	if !isExists { //文件不存在时，不做任何处理
		return "", nil
	}

	if localId == 0 && existsFile.LocalId == 0 {
		// 如果都是文件夹，则保留主机端的文件夹，具体步骤如下
		// 1、将本地的DFS文件夹下的所有文件及文件夹全部移动到主机端的文件夹下
		// 2、删除本地文件夹（这可能会导致已经分享出去的连接失效）
		if _, err := DBConnection.DBConn.Exec("update dfs_file set parentId = ? where parentId = ?", id, existsFile.Id); err != nil {
			return "", err
		}
		if _, err := DBConnection.DBConn.Exec("delete from dfs_file where id = ?", existsFile.Id); err != nil {
			return "", err
		}
	} else if localId > 0 && existsFile.LocalId > 0 {
		// 如果都是文件，则保留最新的一个文件，将日期比较老的文件加入到历史记录
		if id > existsFile.Id { //当前主机端的文件比较新，则将本地的文件设置为历史文件
			if _, err := DBConnection.DBConn.Exec("update dfs_file set isHistory = 1 where id = ?", existsFile.Id); err != nil {
				return "", err
			}
		} else { //本地的文件比较新，则将主机端的文件设置为历史文件
			//该日志执行成功之后要执行的SQL语句
			return "update dfs_file set isHistory = 1 where id = $id", nil
		}
	} else {

		//得到用户信息
		user, _ := UserDao.SelectOne(userId)

		//得到发生错误的文件路径
		path, _ := DfsFileService.GetPathById(existsFile.Id)
		return "", exception.Biz("同步失败，服务器：" + info.Url + "  用户名：" + user.Name + "  路径：" + path + " 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请重命名当前或者服务器端 " + path + " 的文件名。")
	}
	return "", nil
}
