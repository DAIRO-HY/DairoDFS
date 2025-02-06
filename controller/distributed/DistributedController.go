package distributed

import (
	"DairoDFS/dao/SqlLogDao"
	"DairoDFS/extension/Number"
	"net/http"
	"time"
)

/**
 * 数据同步处理Controller
 */
//@Group:/distributed

/**
 * 长连接心跳间隔时间
 */
const KEEP_ALIVE_TIME = 120 * 1000

/**
 * 记录分机端的请求
 */
var distributedClientResponseList = make(map[string]DistributedClientResponseBean)

/**
 * 通知分机端同步
 */
func push() {
	for _, it := range distributedClientResponseList {
		it.writer.Write([]byte{1})
	}

	////一定要将同步客户端response信息列表复制一份在进行通知，因为调用notifyAll()时，其他线程有可能移除对象，而HashSet不能边遍历边移除对象，这回导致报错
	//this.distributedClientResponseList.map { it }.forEach {
	//    synchronized(it) {
	//
	//        //标记为已经结束
	//        it.isCancel = true
	//        (it as Object).notifyAll()
	//    }
	//}
}

/**
 * 分机端同步监听请求
 * 这是一个长连接，直到主机端有数据变更之后才返回
 * @param clientToken 分机端的票据
 * @param lastId 分机端同步到日志最大ID,用来解决分机端在判断是否最新日志的过程中,又有新的日志增加,虽然是小概率事件,但还是有发生的可能
 */
//@Get:/{clientToken}/listen
func Listen(writer http.ResponseWriter, request *http.Request, clientToken string, lastId int64) {
	defer delete(distributedClientResponseList, clientToken)
	preClient, isExists := distributedClientResponseList[clientToken]
	if isExists {

		//将上一个标记为已经结束
		preClient.isCancel = true
		//(preClient as Object).notifyAll()
	}

	//构建分机端同步response信息
	responseBean := DistributedClientResponseBean{
		clientToken: clientToken,
		writer:      writer,
	}

	//添加新的等待
	distributedClientResponseList[clientToken] = responseBean
	for {
		if SqlLogDao.LastID() > lastId { //分机端数据并不是最新的,立即通知更新
			writer.Write([]byte{1})
			break
		}

		//间隔一段时间往客户端发送0，以保持长连接
		writer.Write([]byte{1})
		time.Sleep(KEEP_ALIVE_TIME * time.Millisecond)
		if responseBean.isCancel {
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

/**
 * 获取每个表的id
 * @param tbName 表名
 * @param lastId 已经取到的最后一个id
 * @param aopId 断面ID
 */
//@Request:/get_table_id
func GetTableId(tbName string, lastId int64, aopId int64) string {
	//if (SyncByTable.isRuning || SyncByLog.isRunning) {
	//    throw BusinessException("主机正在同步数据中，请等待完成后继续。")
	//}
	//
	////条数不宜过大,过大可能导致客户端请求失败
	//val maxLimit = 100
	//return Constant.dbService.selectList(
	//    String::class,
	//    "select id from $tbName where id > ? and id < ? order by id asc limit ?",
	//    lastId,
	//    aopId,
	//    maxLimit
	//).joinToString(separator = ",") { it }
	return ""
}

/**
 * 获取表数据
 * @param tbName 表名
 * @param ids 要取的数据id列表
 */
//@Request:/get_table_data
func GetTableData(tbName string, ids string) []any {
	//if (SyncByTable.isRuning || SyncByLog.isRunning) {
	//    throw BusinessException("主机正在同步数据中，请等待完成后继续。")
	//}
	//return Constant.dbService.selectList(
	//    "select * from $tbName where id in ($ids)"
	//)
	return nil
}

/**
 * 文件下载
 * @param request 客户端请求
 * @param response 往客户端返回内容
 * @param id 文件ID
 */
//@Request:/download/{md5}
func Download(writer http.ResponseWriter, request *http.Request, md5 string) {
	//if (SyncByTable.isRuning || SyncByLog.isRunning) {
	//    throw BusinessException("主机正在同步数据中，请等待完成后继续。")
	//}
	//val localFileDto = this.localFileDao.selectByFileMd5(md5)
	//if (localFileDto == null) {
	//    response.status = HttpStatus.NOT_FOUND.value()
	//    return
	//}
	//response.reset() //清除buffer缓存
	//DfsFileUtil.download(localFileDto, request, response)
}
