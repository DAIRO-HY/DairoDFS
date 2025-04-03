package SyncByTable

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBConnection"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/DistributedUtil"
	"DairoDFS/util/DistributedUtil/DfsFileSyncHandle"
	"DairoDFS/util/DistributedUtil/StorageFileSyncHandle"
	"DairoDFS/util/DistributedUtil/SyncByLog"
	"DairoDFS/util/DistributedUtil/SyncHttp"
	"DairoDFS/util/DistributedUtil/SyncInfoManager"
	"encoding/json"
	"strconv"
	"strings"
)

//全量同步时，从主机端下载所有数据和本地数据对比
//避免由于网络延迟，本机没有完全同步主机日志（主要是update，delete语句），导致本机数据与主机不一致。

// 同步所有数据
func SyncAll() {

	// 重新加载同步信息
	SyncInfoManager.ReloadList()

	//避免并发
	defer func() {

		//标记正在全量同步结束
		DistributedUtil.IsTableSyncing = false
		DistributedUtil.SyncLock.Unlock()
	}()
	DistributedUtil.SyncLock.Lock()

	//标记正在全量同步中
	DistributedUtil.IsTableSyncing = true

	for _, info := range SyncInfoManager.SyncInfoList {
		syncByInfo(info)
	}

	if !SyncInfoManager.HasError() {

		// 全量同步完成，如果没有错误消息，立马开启日志同步
		go SyncByLog.ListenAll()
	}
}

// 全量同步主机数据
func syncByInfo(info *DistributedUtil.SyncServerInfo) {
	defer func() {
		if r := recover(); r != nil { //如果发生了panic错误
			switch rValue := r.(type) {
			case error:
				info.Msg = "全量同步失败:" + rValue.Error()
			case string:
				info.Msg = "全量同步失败:" + rValue
			}
			info.State = 2

			//同步过程发生了错误，回滚数据
			info.Rollback()
		}
	}()
	info.State = 1
	info.Msg = "全量同步中"

	//从主机端获取断面ID,避免同步过程中，主机数据发生变化导致数据不一致的BUG
	aopId := getAopId(info)
	tbNames := []string{
		"user",
		"user_token",
		"dfs_file",
		"dfs_file_delete",
		"share",
		"storage_file",
	}
	info.Count = GetTableCount(info, tbNames, aopId)
	info.SyncCount = 0
	for _, it := range tbNames {
		loopSync(info, it, 0, aopId)
	}

	//从日志数据表中删除当前已经同步成功的服务端日志
	DBConnection.DBConn.Exec("delete from sql_log where source = ? and id < ?", info.Url, aopId)

	//设置日志同步最后的ID
	SyncByLog.SaveLastId(info.Url, aopId)
	info.State = 0
	info.Msg = "全量同步完成"
}

/**
 * 循环同步数据，直到包数据同步完成
 */
func loopSync(info *DistributedUtil.SyncServerInfo, tbName string, lastId int64, aopId int64) {
	if info.IsStop {
		panic("同步被强制取消")
	}

	//得到主机端的数据
	masterDataMapList := getTableData(info, tbName, lastId, aopId)
	if len(masterDataMapList) == 0 {
		return
	}

	//插入数据
	insertData(info, tbName, masterDataMapList)

	//设置本次获取到数据的最后一个ID
	currentLastId := int64(masterDataMapList[len(masterDataMapList)-1]["id"].(float64))

	//再次同步
	loopSync(info, tbName, currentLastId, aopId)
}

/**
 * 获取一个断面ID，防止再全量同步的过程中，主机又增加数据，导致全量同步数据不完整
 * 其实就是服务器端的时间戳
 */
func getAopId(info *DistributedUtil.SyncServerInfo) int64 {
	url := info.Url + "/get_aop_id"
	data, err := SyncHttp.Request(url)
	if err != nil {
		panic(err)
	}
	aopId, _ := strconv.ParseInt(string(data), 10, 64)
	return aopId
}

// 获取要同步数据总条数
// tbNames 表名
// aopId 断面ID
func GetTableCount(info *DistributedUtil.SyncServerInfo, tbNames []string, aopId int64) int {
	url := info.Url + "/get_table_count?aopId=" + String.ValueOf(aopId) + "&tbNames=" + strings.Join(tbNames, "&tbNames=")
	data, err := SyncHttp.Request(url)
	if err != nil {
		panic(err)
	}
	count, _ := strconv.Atoi(string(data))
	return count
}

// 从同步主机端取数据
func getTableData(info *DistributedUtil.SyncServerInfo, tbName string, lastId int64, aopId int64) []map[string]any {
	url := info.Url + "/get_table_data?tbName=" + tbName + "&lastId=" + String.ValueOf(lastId) + "&aopId=" + String.ValueOf(aopId)
	data, err := SyncHttp.Request(url)
	if err != nil {
		panic(err)
	}
	dataMapList := make([]map[string]any, 0)
	json.Unmarshal(data, &dataMapList)
	return dataMapList
}

// 往数据库插入数据
func insertData(info *DistributedUtil.SyncServerInfo, tbName string, masterDataMapList []map[string]any) {
	for _, masterDataMap := range masterDataMapList {
		info.SyncCount++
		switch tbName {
		case "storage_file": //当前请求的是本地文件存储表，先去下载文件
			StorageFileSyncHandle.ByTable(info, masterDataMap)
		case "dfs_file": //如果是用户文件表
			DfsFileSyncHandle.ByTable(info, masterDataMap)
		}

		//当前ID
		masterId := masterDataMap["id"]
		existsData := DBUtil.SelectOneMap("select * from "+tbName+" where id = ?", masterId)
		if existsData != nil { //如果该文件已经存在，则需要对比文件内容是否一致

			//将数据库查询出来的数据从新序列化再反序列，确保数据类型和主机端返回的数据类型一直。
			//DBUtil.SelectOneMap查询结果的数据类型会根据数据库数据类型保持一直。
			//比如数据库int类型，查询结果就是int类型，而通过json反序列化之后就成了float64类型
			existsJson, _ := json.Marshal(existsData)
			json.Unmarshal(existsJson, &existsData)

			//标记是否数据一直
			isEqual := true
			for key, value := range existsData {
				if masterDataMap[key] == value {
					continue
				}

				//当数据不一致时，应该做点什么
				isEqual = false
				if SystemConfig.Instance().IgnoreSyncError { //忽略同步错误
					break
				}
				panic("表" + tbName + "数据同步失败，该id[" + String.ValueOf(masterId) + "]数据已经存在，字段[" + key + "]主机：" + String.ValueOf(masterDataMap[key]) + "，本机：" + String.ValueOf(value))
			}
			if isEqual { //数据没有变化。不做任何处理
				info.Commit()
				continue
			}

			//删掉本机已经存在的数据，使用新的数据
			info.DbTx().Exec("delete from "+tbName+" where id = ?", masterId)
		}
		insertKeys := ""
		insertValueReplaces := ""
		insertValues := make([]any, 0)
		for k, v := range masterDataMap {
			insertKeys += k + ","
			insertValueReplaces += "?,"
			insertValues = append(insertValues, v)
		}

		//去掉最后的逗号
		insertKeys = insertKeys[:len(insertKeys)-1]
		insertValueReplaces = insertValueReplaces[:len(insertValueReplaces)-1]

		//拼接sql语句
		insertSql := "insert into " + tbName + " (" + insertKeys + ") values (" + insertValueReplaces + ")"
		if _, err := info.DbTx().Exec(insertSql, insertValues...); err != nil {
			panic(err)
		}
		if err := info.Commit(); err != nil { //最后记得提交事务，将被数据反应到数据库
			panic(err)
		}
	}
}
