package distributed

import (
	"DairoDFS/controller/distributed/DistributedPush"
	"DairoDFS/dao/SqlLogDao"
	"DairoDFS/dao/StorageFileDao"
	"DairoDFS/extension/Number"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/DfsFileUtil"
	"net/http"
	"strings"
	"sync"
	"time"
)

/**
 * 数据同步处理Controller
 */
//@Group:/distributed/{token}/{clientToken}

// 记录分机端的请求
var waitingRequestMap = make(map[string]int64)

var waitingRequestLock sync.Mutex

// 限制每次同步数据量，条数不宜过大,过大可能导致客户端请求时，url太长导致请求失败
const _MAX_SYNC_DATA_LIMIT = 100

/**
 * 分机端同步监听请求
 * 这是一个长连接，直到主机端有数据变更之后才返回
 * @param clientToken 分机端的票据
 * @param lastId 分机端同步到日志最大ID,用来解决分机端在判断是否最新日志的过程中,又有新的日志增加,虽然是小概率事件,但还是有发生的可能
 */
//@Get:/listen
func Listen(writer http.ResponseWriter, clientToken string, lastId int64) {
	waitingRequestLock.Lock()
	time.Sleep(1 * time.Microsecond) //确保每次生成的时间戳不一样
	nowMicro := time.Now().UnixMicro()
	waitingRequestMap[clientToken] = nowMicro //添加新的等待
	waitingRequestLock.Unlock()
	DistributedPush.Cond.Broadcast()
	for {
		if SqlLogDao.LastID() > lastId { //分机端数据并不是最新的,立即通知更新
			writer.Write([]byte{1})
			writer.(http.Flusher).Flush()
			break
		}

		//间隔一段时间往客户端发送0，以保持长连接
		writer.Write([]byte{0})
		writer.(http.Flusher).Flush()
		DistributedPush.Cond.L.Lock()
		DistributedPush.Cond.Wait()
		DistributedPush.Cond.L.Unlock()
		existsMicro, isExists := waitingRequestMap[clientToken]
		if !isExists || existsMicro != nowMicro { //已经被新的请求替代
			break
		}
	}
}

// 获取sql日志
// @Request:/get_log
func GetLog(lastId int64) []map[string]any {
	return SqlLogDao.GetLog(lastId)
}

/**
 * 主机发起同步通知
 */
//@//Request:/push_notify
func pushNotify() {
	//thread {
	//    SyncByLog.start()
	//}
}

/**
 * 获取一个断面ID，防止再全量同步的过程中，主机又增加数据，导致全量同步数据不完整
 * 其实就是当前服务器时间戳
 */
//@Request:/get_aop_id
func GetAopId() int64 {
	return Number.ID()
}

// 获取每个表的id
// tbName 表名
// lastId 已经取到的最后一个id
// aopId 断面ID
// @Request:/get_table_id
func GetTableId(tbName string, lastId int64, aopId int64) string {
	//TODO:如果本机正在同步数据,则禁止往分机端传递文件
	//if (SyncByTable.isRuning || SyncByLog.isRunning) {
	//   throw BusinessException("主机正在同步数据中，请等待完成后继续。")
	//}
	idList := DBUtil.SelectList[string](
		"select id from "+tbName+" where id > ? and id < ? order by id asc limit ?",
		lastId,
		aopId,
		_MAX_SYNC_DATA_LIMIT,
	)
	return strings.Join(idList, ",")
}

/**
 * 获取表数据
 * @param tbName 表名
 * @param ids 要取的数据id列表
 */
//@Request:/get_table_data
func GetTableData(tbName string, ids string) []map[string]any {
	//TODO:如果本机正在同步数据,则禁止往分机端传递文件
	//if (SyncByTable.isRuning || SyncByLog.isRunning) {
	//   throw BusinessException("主机正在同步数据中，请等待完成后继续。")
	//}
	list, _ := DBUtil.SelectToListMap("select * from " + tbName + " where id in (" + ids + ")")
	return list
}

/**
 * 文件下载
 * @param request 客户端请求
 * @param response 往客户端返回内容
 * @param id 文件ID
 */
//@Request:/download/{md5}
func Download(writer http.ResponseWriter, request *http.Request, md5 string) {
	//TODO:如果本机正在同步数据,则禁止往分机端传递文件
	//if (SyncByTable.isRuning || SyncByLog.isRunning) {
	//   throw BusinessException("主机正在同步数据中，请等待完成后继续。")
	//}
	storageFileDto, isExists := StorageFileDao.SelectByFileMd5(md5)
	if !isExists {
		writer.WriteHeader(http.StatusNotFound)
		return
	}
	DfsFileUtil.Download(storageFileDto.Path, writer, request)
}
