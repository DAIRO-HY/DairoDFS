package SyncByTable

import (
	"DairoDFS/extension/String"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/Sync/SyncByLog"
	"DairoDFS/util/Sync/SyncHttp"
	"DairoDFS/util/Sync/bean"
	"encoding/json"
	"strconv"
	"strings"
	"sync"
)

/**
 * 全量同步工具
 */

var lock sync.Mutex

/**
 * 标记全量同步是否正在进行中
 */
var mIsRuning bool

/**
 * 同步信息Socket
 * 页面实时查看同步信息用
 */
//private val syncSocket = SyncWebSocketHandler::class.bean

/**
 * 获取运行状态
 */
func IsRuning() bool {
	var result bool
	lock.Lock()
	result = mIsRuning
	lock.Unlock()
	return result
}

/**
 * 开始同步
 * @param isForce 是否强制执行
 */
func Start(isForce bool) {
	if SyncByLog.IsRuning() { //日志同步正在进行中
		return
	}
	if mIsRuning { //并发防止
		return
	}
	defer func() {
		mIsRuning = false
	}()
	mIsRuning = true
	if isForce { //强行执行
		for _, it := range SyncByLog.SyncInfoList {
			it.State = 0
		}
	}
	doSync()
}

func doSync() {
	for _, info := range SyncByLog.SyncInfoList {
		if info.State != 0 { //只允许待机中的同步
			continue
		}
		info.State = 1
		info.Msg = ""
		//this.syncSocket.send(info)

		//断面ID,从主机端获取的数据ID不得大于该值
		aopId, aopIdErr := getAopId(info)
		if aopIdErr != nil {
			info.State = 2
			info.Msg = "获取断面ID失败：" + aopIdErr.Error()
			return
		}
		tbNames := []string{
			"user",
			"user_token",
			"dfs_file",
			"dfs_file_delete",
			"share",
			"local_file",
		}
		for _, it := range tbNames {
			loopSync(info, it, 0, aopId)
		}

		//从日志数据表中删除当前已经同步成功的服务端日志
		_, err := DBConnection.DBConn.Exec("delete from sql_log where source = ? and id < ?", info.Url, aopId)
		if err != nil {
			info.State = 2
			info.Msg = err.Error()
			return
		}

		//设置日志同步最后的ID
		SyncByLog.SaveLastId(info, aopId)
		info.State = 0
		info.Msg = "完成"
	}
}

/**
 * 循环同步数据，直到包数据同步完成
 */
func loopSync(info *bean.SyncServerInfo, tbName string, lastId int64, aopId int64) {

	//通过表名从主机端获取某个断面以后的id列表
	masterIds, masterIdsErr := getTableId(info, tbName, lastId, aopId)
	if masterIdsErr != nil {
		info.State = 2
		info.Msg = masterIdsErr.Error()
		return
	}
	if masterIds == "" { //同步主机端的数据已经全部取完
		return
	}

	//设置本次获取到的最后一个ID
	var currentLastId int64
	if strings.Contains(masterIds, ",") {
		currentLastId, _ = strconv.ParseInt(masterIds[strings.LastIndex(masterIds, ","):], 10, 64)
	} else {
		currentLastId, _ = strconv.ParseInt(masterIds, 10, 64)
	}

	//得到需要
	needSyncIds := filterNotExistsId(tbName, masterIds)
	if needSyncIds == "" { //本次获取到的数据，本地已经全部存在，继续获取下一篇段数据

		//再次同步
		loopSync(info, tbName, currentLastId, aopId)
		return
	}

	//得到需要同步的数据
	tableData, tableDataErr := getTableData(info, tbName, needSyncIds)
	if tableDataErr != nil {
		info.State = 2
		info.Msg = "获取表数据失败：" + masterIdsErr.Error()
		return
	}

	dataList := make([]map[string]any, 0)
	json.Unmarshal(tableData, &dataList)

	//插入数据
	insertData(info, tbName, dataList)

	//记录当前同步的数据条数
	info.SyncCount += len(dataList)
	//this.syncSocket.send(info)

	//再次同步
	loopSync(info, tbName, currentLastId, aopId)
}

/**
 * 获取一个断面ID，防止再全量同步的过程中，主机又增加数据，导致全量同步数据不完整
 * 其实就是服务器端的时间戳
 */
func getAopId(info *bean.SyncServerInfo) (int64, error) {
	url := info.Url + "/get_aop_id"
	data, err := SyncHttp.Request(url)
	if err != nil {
		return 0, err
	}
	aopId, _ := strconv.ParseInt(string(data), 10, 64)
	return aopId, nil
}

/**
 * 从主机获取某表的一批数据id
 * @param info 主机信息
 * @param tbName 表名
 * @param lastId 上次获取到的最后一个id
 * @param aopId 本次同步的服务器端的最大id
 */
func getTableId(info *bean.SyncServerInfo, tbName string, lastId int64, aopId int64) (string, error) {
	url := info.Url + "/get_table_id?tbName=" + tbName + "&lastId=" + String.ValueOf(lastId) + "&aopId=" + String.ValueOf(aopId)
	data, err := SyncHttp.Request(url)
	if err != nil {
		return "", err
	}
	return string(data), nil
}

/**
 * 筛选出本地不存在的ID
 */
func filterNotExistsId(tbName string, ids string) string {

	//得到已经存在的ID列表
	existsIdList := DBUtil.SelectList[string]("select id from " + tbName + " where id in (" + ids + ")")
	existsIdMap := make(map[string]struct{})
	for _, it := range existsIdList {
		existsIdMap[it] = struct{}{}
	}

	//得到本地不存在的id
	notExistsIds := ""
	for _, it := range strings.Split(ids, ",") {
		_, isExists := existsIdMap[it]
		if !isExists {
			notExistsIds += it + ","
		}
	}
	return notExistsIds
}

/**
 * 从同步主机端取数据
 */
func getTableData(info *bean.SyncServerInfo, tbName string, ids string) ([]byte, error) {
	url := info.Url + "/get_table_data?tbName=" + tbName + "&ids=" + ids
	return SyncHttp.Request(url)
}

/**
 * 同步主机数据
 */
func insertData(info *bean.SyncServerInfo, tbName string, dataList []map[string]any) {
	//data.forEach { item ->
	//    item as ObjectNode
	//    when (tbName) {
	//
	//        //当前请求的是本地文件存储表，先去下载文件
	//        "local_file" -> LocalFileSyncHandle.byTable(info, item)
	//
	//        //如果是用户文件表
	//        "dfs_file" -> DfsFileSyncHandle.handle(info, item)
	//    }
	//
	//    //要插入的字段
	//    val fields = ArrayList<String>()
	//    val values = ArrayList<String?>()
	//    item.fields().forEach {
	//        fields.add(it.key)
	//
	//        val value = it.value
	//        if (value.isNull) {//null值
	//            values.add(null)
	//        } else {
	//            values.add(it.value.asText())
	//        }
	//    }
	//    try {
	//        Constant.dbService.exec(
	//            "insert into $tbName(${fields.joinToString()}) values (${fields.joinToString { "?" }})",
	//            *values.toArray()
	//        )
	//    } catch (e: Exception) {
	//        if (e is SQLiteException && e.message!!.contains("UNIQUE constraint failed: user.name")) {
	//            throw Exception("同步失败，原因： 用户名“${values[2]}”已存在。请先修改用户名为“${values[2]}”的用户后再重试")
	//        }
	//        throw e
	//    }
	//}
}
