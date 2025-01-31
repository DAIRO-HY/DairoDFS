package cn.dairo.dfs.sync.sync_handle

import com.fasterxml.jackson.databind.node.ObjectNode
import cn.dairo.dfs.config.Constant
import cn.dairo.dfs.dao.DfsFileDao
import cn.dairo.dfs.dao.UserDao
import cn.dairo.dfs.extension.bean
import cn.dairo.dfs.service.DfsFileService
import cn.dairo.dfs.sync.bean.SyncServerInfo

/**
 * DFS文件同步之前，本地文件的一些操作
 */
object DfsFileSyncHandle {
    fun handle(info: SyncServerInfo, item: ObjectNode) {
        val deleteDate = item.path("deleteDate").asLong()
        if (deleteDate > 0) {//该文件已经被删除，不用做任何处理
            return
        }
        val isHistory = item.path("isHistory").asInt()
        if (isHistory == 1) {//这是一个历史文件
            return
        }

        //用户文件id
        val id = item.path("id").asLong()

        //用户id
        val userId = item.path("userId").asLong()

        //父级文件夹id
        val parentId = item.path("parentId").asLong()

        //文件（夹）名
        val name = item.path("name").asText()
        val dfsFile = DfsFileDao::class.bean.selectByParentIdAndName(userId, parentId, name)
        if (dfsFile == null) {//文件不存在时，不做任何处理
            return
        }

        //本地存储文件id
        val localId = item.path("localId").asLong()
        if (localId == 0L && dfsFile.localId == 0L) {
            // 如果都是文件夹，则保留主机端的文件夹，具体步骤如下
            // 1、将本地的DFS文件夹下的所有文件及文件夹全部移动到主机端的文件夹下
            // 2、删除本地文件夹（这可能会导致已经分享出去的连接失效）
            Constant.dbService.exec("update dfs_file set parentId = ? where parentId = ?", id, dfsFile.id)
            Constant.dbService.exec("delete from dfs_file where id = ?", dfsFile.id)
        } else if (localId > 0 && dfsFile.localId!! > 0) {
            // 如果都是文件，则保留最新的一个文件，将日期比较老的文件加入到历史记录
            if (id > dfsFile.id!!) {//当前主机端的文件比较新，则将本地的文件设置为历史文件
                Constant.dbService.exec("update dfs_file set isHistory = 1 where id = ?", dfsFile.id)
            } else {//本地的文件比较新，则将主机端的文件设置为历史文件
                item.put("isHistory", 1)
            }
        } else {

            //得到用户信息
            val user = UserDao::class.bean.selectOne(userId)

            //得到发生错误的文件路径
            val path = DfsFileService::class.bean.getPathById(dfsFile.id!!)
            throw RuntimeException("同步失败，服务器：${info.url}  用户名：${user?.name}  路径：$path 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请重命名当前或者服务器端 $path 的文件名。")
        }
    }

    fun handleBySyncLog(info: SyncServerInfo, params: ArrayList<Any>): String? {

        //用户文件id
        val id = params[0].toString().toLong()

        //用户id
        val userId = params[1].toString().toLong()

        //父级文件夹id
        val parentId = params[2].toString().toLong()

        //文件（夹）名
        val name = params[3] as String
        val dfsFile = DfsFileDao::class.bean.selectByParentIdAndName(userId, parentId, name)
        if (dfsFile == null) {//文件不存在时，不做任何处理
            return null
        }

        //本地存储文件id
        val localId = params[6].toString().toLong()
        if (localId == 0L && dfsFile.localId == 0L) {
            // 如果都是文件夹，则保留主机端的文件夹，具体步骤如下
            // 1、将本地的DFS文件夹下的所有文件及文件夹全部移动到主机端的文件夹下
            // 2、删除本地文件夹（这可能会导致已经分享出去的连接失效）
            Constant.dbService.exec("update dfs_file set parentId = ? where parentId = ?", id, dfsFile.id)
            Constant.dbService.exec("delete from dfs_file where id = ?", dfsFile.id)
        } else if (localId > 0 && dfsFile.localId!! > 0) {
            // 如果都是文件，则保留最新的一个文件，将日期比较老的文件加入到历史记录
            if (id > dfsFile.id!!) {//当前主机端的文件比较新，则将本地的文件设置为历史文件
                Constant.dbService.exec("update dfs_file set isHistory = 1 where id = ?", dfsFile.id)
            } else {//本地的文件比较新，则将主机端的文件设置为历史文件
                //该日志执行成功之后要执行的SQL语句
                val afterSql = "update dfs_file set isHistory = 1 where id = $id"
                return afterSql
            }
        } else {

            //得到用户信息
            val user = UserDao::class.bean.selectOne(userId)

            //得到发生错误的文件路径
            val path = DfsFileService::class.bean.getPathById(dfsFile.id!!)
            throw RuntimeException("同步失败，服务器：${info.url}  用户名：${user?.name}  路径：$path 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请重命名当前或者服务器端 $path 的文件名。")
        }
        return null
    }
}