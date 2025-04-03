package DfsFileSyncHandle

import (
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/UserDao"
	"DairoDFS/service/DfsFileService"
	"DairoDFS/util/DistributedUtil"
)

/**
 * DFS文件同步之前，本地文件的一些操作
 */
func ByTable(info *DistributedUtil.SyncServerInfo, dataMap map[string]any) {

	//用户文件id
	id := int64(dataMap["id"].(float64))

	//用户id
	userId := int64(dataMap["userId"].(float64))

	//父级文件夹id
	parentId := int64(dataMap["parentId"].(float64))

	//文件（夹）名
	name := dataMap["name"].(string)

	//存储文件id
	storageId := int64(dataMap["storageId"].(float64))

	existsFile, isExists := DfsFileDao.SelectByParentIdAndName(userId, parentId, name)
	if !isExists { //文件不存在时，不做任何处理
		return
	}
	if id == existsFile.Id { //同一个文件时，什么也不做
		return
	}
	//该分机端DFS文件已经存在的话，要做一些特殊处理
	if storageId == 0 && existsFile.StorageId == 0 {
		// 如果都是文件夹，则保留主机端的文件夹，具体步骤如下
		// 1、将本地的DFS文件夹下的所有文件及文件夹全部移动到主机端的文件夹下
		// 2、删除本地文件夹（这可能会导致已经分享出去的连接失效）
		if _, err := info.DbTx().Exec("update dfs_file set parentId = ? where parentId = ?", id, existsFile.Id); err != nil {
			panic(err)
		}
		if _, err := info.DbTx().Exec("delete from dfs_file where id = ?", existsFile.Id); err != nil {
			panic(err)
		}
	} else if storageId > 0 && existsFile.StorageId > 0 {
		// 如果都是文件，则保留最新的一个文件，将日期比较老的文件加入到历史记录
		if id > existsFile.Id { //当前主机端的文件比较新，则将本地的文件设置为历史文件
			if _, err := info.DbTx().Exec("update dfs_file set isHistory = 1 where id = ?", existsFile.Id); err != nil {
				panic(err)
			}
		} else { //本地的文件比较新，则将主机端的文件设置为历史文件
			//这里一定要设置为float64(1)，不能时1。因为从主机端反序列化的数据中，1被序列化成了float类型
			dataMap["isHistory"] = float64(1)
		}
	} else { //主机端和分几端，一个时文件，一个是文件夹，无法同步

		//得到用户信息
		user, _ := UserDao.SelectOne(userId)

		//得到发生错误的文件路径
		path := DfsFileService.GetPathById(id)
		panic("同步失败，服务器：" + info.Url + "  用户名：" + user.Name + "  路径：" + path + " 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请重命名当前或者服务器端 " + path + " 的文件名。")
	}
}

func ByLog(info *DistributedUtil.SyncServerInfo, params []any) string {

	//用户文件id
	id := int64(params[0].(float64))

	//用户id
	userId := int64(params[1].(float64))

	//父级文件夹id
	parentId := int64(params[2].(float64))

	//文件（夹）名
	name := params[3].(string)

	//存储文件id
	storageId := int64(params[7].(float64))

	existsFile, isExists := DfsFileDao.SelectByParentIdAndName(userId, parentId, name)
	if !isExists { //文件不存在时，不做任何处理
		return ""
	}

	if storageId == 0 && existsFile.StorageId == 0 {
		// 如果都是文件夹，则保留主机端的文件夹，具体步骤如下
		// 1、将本地的DFS文件夹下的所有文件及文件夹全部移动到主机端的文件夹下
		// 2、删除本地文件夹（这可能会导致已经分享出去的连接失效）
		if _, err := info.DbTx().Exec("update dfs_file set parentId = ? where parentId = ?", id, existsFile.Id); err != nil {
			panic(err)
		}
		if _, err := info.DbTx().Exec("delete from dfs_file where id = ?", existsFile.Id); err != nil {
			panic(err)
		}
	} else if storageId > 0 && existsFile.StorageId > 0 {
		// 如果都是文件，则保留最新的一个文件，将日期比较老的文件加入到历史记录
		if id > existsFile.Id { //当前主机端的文件比较新，则将本地的文件设置为历史文件
			if _, err := info.DbTx().Exec("update dfs_file set isHistory = 1 where id = ?", existsFile.Id); err != nil {
				panic(err)
			}
		} else { //本地的文件比较新，则将主机端的文件设置为历史文件
			//该日志执行成功之后要执行的SQL语句
			return "update dfs_file set isHistory = 1 where id = $id"
		}
	} else {

		//得到用户信息
		user, _ := UserDao.SelectOne(userId)

		//得到发生错误的文件路径
		path := DfsFileService.GetPathById(existsFile.Id)
		panic("同步失败，服务器：" + info.Url + "  用户名：" + user.Name + "  路径：" + path + " 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请重命名当前或者服务器端 " + path + " 的文件名。")
	}
	return ""
}
