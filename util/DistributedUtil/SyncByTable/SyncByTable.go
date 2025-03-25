package SyncByTable

import (
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

// 同步所有数据
func SyncAll() {

	// 重新加载同步信息
	SyncInfoManager.ReloadList()

	//避免并发
	defer DistributedUtil.SyncLock.Unlock()
	DistributedUtil.SyncLock.Lock()

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

	//通过表名从主机端获取某个断面以后的id列表
	masterIds := getTableId(info, tbName, lastId, aopId)
	if masterIds == "" { //同步主机端的数据已经全部取完
		return
	}

	//设置本次获取到的最后一个ID
	var currentLastId int64
	if strings.Contains(masterIds, ",") {
		currentLastId, _ = strconv.ParseInt(masterIds[strings.LastIndex(masterIds, ",")+1:], 10, 64)
	} else {
		currentLastId, _ = strconv.ParseInt(masterIds, 10, 64)
	}

	//筛选出本地不存在的ID
	needSyncIds := filterNotExistsId(tbName, masterIds)
	if needSyncIds == "" { //本次获取到的数据，本地已经全部存在，继续获取下一篇段数据

		//再次同步
		loopSync(info, tbName, currentLastId, aopId)
		return
	}

	//得到需要同步的数据
	dataMapList := getTableData(info, tbName, needSyncIds)

	//插入数据
	insertData(info, tbName, dataMapList)

	//记录当前同步的数据条数
	info.SyncCount += len(dataMapList)

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

/**
 * 从主机获取某表的一批数据id
 * @param info 主机信息
 * @param tbName 表名
 * @param lastId 上次获取到的最后一个id
 * @param aopId 本次同步的服务器端的最大id
 */
func getTableId(info *DistributedUtil.SyncServerInfo, tbName string, lastId int64, aopId int64) string {
	url := info.Url + "/get_table_id?tbName=" + tbName + "&lastId=" + String.ValueOf(lastId) + "&aopId=" + String.ValueOf(aopId)
	data, err := SyncHttp.Request(url)
	if err != nil {
		panic(err)
	}
	return string(data)
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
	if len(notExistsIds) > 0 {
		notExistsIds = notExistsIds[:len(notExistsIds)-1]
	}
	return notExistsIds
}

// 从同步主机端取数据
func getTableData(info *DistributedUtil.SyncServerInfo, tbName string, ids string) []map[string]any {
	url := info.Url + "/get_table_data?tbName=" + tbName + "&ids=" + ids
	data, err := SyncHttp.Request(url)
	if err != nil {
		panic(err)
	}
	dataMapList := make([]map[string]any, 0)
	json.Unmarshal(data, &dataMapList)
	return dataMapList
}

// 往数据库插入数据
func insertData(info *DistributedUtil.SyncServerInfo, tbName string, dataMapList []map[string]any) {
	for _, dataMap := range dataMapList {
		switch tbName {
		case "storage_file": //当前请求的是本地文件存储表，先去下载文件
			StorageFileSyncHandle.ByTable(info, dataMap)
		case "dfs_file": //如果是用户文件表
			DfsFileSyncHandle.ByTable(info, dataMap)
		}
		insertKeys := ""
		insertValueReplaces := ""
		insertValues := make([]any, 0)
		for k, v := range dataMap {
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
