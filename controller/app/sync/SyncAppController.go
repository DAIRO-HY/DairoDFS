package sync

import "DairoDFS/controller/app/sync/form"

// 数据同步状态
//@Group: /app/sync

/**
 * 页面初始化
 */
//@Html:.html
func Html() {}

/**
 * 页面数据初始化
 */
//@Post:/info_list
func InfoList() form.SyncServerForm {
	//val formList = SyncByLog.syncInfoList.map {
	//    val form = SyncServerForm()
	//    form.url = it.url
	//    form.state = it.state
	//    form.msg = it.msg
	//    form.no = it.no
	//    form.syncCount = it.syncCount
	//    form.lastHeartTime = it.lastHeartTime
	//    form.lastTime = it.lastTime
	//    form
	//}
	//return formList
	return form.SyncServerForm{}
}

/**
 * 日志同步
 */
//@Post:/by_log
func BySync() {
	//thread {
	//    SyncByLog.start(true)
	//}
}

/**
 * 同步分机端用
 * 全量同步
 */
//@Post:/by_table
func ByTable() {
	//thread {
	//    SyncByTable.start(true)
	//}
}
