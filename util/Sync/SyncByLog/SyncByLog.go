package SyncByLog

import (
	"DairoDFS/application"
	"DairoDFS/dao/SqlLogDao"
	"DairoDFS/dao/dto"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/Sync/SyncByTable"
	"DairoDFS/util/Sync/SyncHandle"
	"DairoDFS/util/Sync/SyncHttp"
	"DairoDFS/util/Sync/bean"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"
)

/**
 * 应用启动执行
 */
//class SyncByLogBoot : ApplicationRunner {
//    override fun run(args: ApplicationArguments) {
//        SyncByLog.init()
//        SyncByLog.listenAll()
//    }
//}

/**
 * 当前同步主机信息
 */
var SyncInfoList []*bean.SyncServerInfo

/**
 * 是否正在同步中
 */
var mIsRuning bool

/**
 * 同步信息Socket
 * 页面实时查看同步信息用
 */
//private val syncSocket = SyncWebSocketHandler::class.bean

var lock sync.Mutex

/**
 * 记录等待了的时间
 */
var waitTimes = 0

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
* 最后同步的ID存放目录
 */
var syncLastIdFilePath = application.DataPath + "/sync_last_id"

///**
// * 获取配置同步主机
// */
//fun init() {
//    this.syncInfoList = SystemConfig.instance.syncDomains.mapIndexed { index, it ->
//        val info = SyncServerInfo()
//        info.url = it
//        info.no = index + 1
//        info
//    }
//}
//
///**
// * 等待中的请求
// */
//private val waitingHttpList = HashSet<SyncLogListenHttpBean>()
//
///**
// * 监听所有配置的主机
// */
//fun listenAll() = thread {
//
//    //先停止掉之前所有的轮询
//    this.waitingHttpList.forEach {
//        try {
//            it.cancle()
//        } catch (_: Exception) {
//        }
//    }
//
//    //清空监听等待
//    this.waitingHttpList.clear()
//    this.syncInfoList.forEach {
//        listen(it)
//    }
//}
//
///**
// * 监听服务端日志变化
// */
//private fun listen(info: SyncServerInfo) {
//    thread {
//        while (true) {
//            sleep(1000)
//            val http =
//                URL(info.url + "/${SystemConfig.instance.token}/listen?lastId=" + this.getLastId(info)).openConnection() as HttpURLConnection
//            http.readTimeout = DistributedController.KEEP_ALIVE_TIME + 10
//            http.connectTimeout = 10000
//
//            val listenHttp = SyncLogListenHttpBean(http)
//            this.waitingHttpList.add(listenHttp)
//            try {
//                http.connect()
//                val iStream = http.inputStream
//                iStream.use {
//                    var tag: Int
//                    while (it.read().also { tag = it } != -1) {
//
//                        //记录最有一次心跳时间
//                        info.lastHeartTime = System.currentTimeMillis()
//                        info.msg = "心跳检测中。"
//                        this.syncSocket.send(info)
//                        if (tag == 1) {//接收到的标记为1时，代表服务器端有新的日志
//                            this.start()
//                            break
//                        }
//                    }
//                }
//            } catch (e: Exception) {
//                if (!listenHttp.isCanceled) {
//                    //e.printStackTrace()
//                    info.msg = "服务端心跳检查失败。"
//                    this.syncSocket.send(info)
//
//                    //如果网络连接报错，则等待一段时间之后在恢复
//                    sleep(10000)
//                }
//            } finally {
//                http.disconnect()
//            }
//
//            //每次同步完成之后都重新开启新的请求
//            this.waitingHttpList.remove(listenHttp)
//            if (listenHttp.isCanceled) {//如果已经被取消
//                break
//            }
//            if (info.state == 2) {//如果同步发生了错误
//                break
//            }
//        }
//    }
//}

/**
* 启动执行
* @param isForce 是否强制执行
 */
func Start(isForce bool) {
	if SyncByTable.IsRuning() { //全量同步正在进行中
		return
	}
	if mIsRuning { //并发防止
		return
	}
	mIsRuning = true
	if isForce { //强行执行
		for _, it := range SyncInfoList {
			it.State = 0
		}
	}
	for _, it := range SyncInfoList {
		if it.State != 0 { //只允许待机中的同步
			continue
		}
		it.State = 1 //标记为同步中
		it.Msg = ""
		//this.syncSocket.send(it)
		requestSqlLog(it)
	}
	mIsRuning = false
	waitTimes = 0
}

// 循环取sql日志
// @return 是否处理完成
func requestSqlLog(info *bean.SyncServerInfo) {

	//得到最后请求的id
	lastId := getLastId(info)
	url := info.Url + "/get_log?lastId=" + String.ValueOf(lastId)
	logData, err := SyncHttp.Request(url)
	if err != nil {
		info.State = 2 //标记为同步失败
		info.Msg = err.Error()
		//this.syncSocket.send(info)
		return
	}
	if string(logData) == "[]" { //已经没有sql日志

		//执行日志sql
		executeSqlLog(info)

		info.State = 0 //同步完成，标记为待机中
		info.Msg = ""
		info.LastTime = time.Now().UnixMilli() //最后一次同步完成时间
		//this.syncSocket.send(info)
		return
	}
	logList := make([]dto.SqlLogDto, 0)
	json.Unmarshal(logData, &logList)
	addLog(info, logList)
	lastLog := logList[len(logList)-1]

	//执行成功之后立即将当前日志的日期保存到本地,降低sql被重复执行的BUG
	SaveLastId(info, lastLog.Id)

	//执行日志sql
	executeSqlLog(info)

	//递归调用，直到服务端日志同步完成
	requestSqlLog(info)
}

// 从主机请求到的日志保存到本地日志
func addLog(info *bean.SyncServerInfo, dataList []dto.SqlLogDto) {
	for _, it := range dataList {
		_, err := DBUtil.DBConn.Exec("insert into sql_log(id,date,sql,param,state,source) values(?,?,?,?,?,?)",
			it.Id, it.Date, it.Sql, it.Param, 0, info.Url)
		if err != nil {
			if strings.Contains(err.Error(), "UNIQUE constraint failed: sql_log.id") {
				// 这条数据已经添加
				continue
			} else {
				//@TODO:这条数据添加报错，需要做点什么
				fmt.Println(err)
			}
		}
	}
}

/**
* 执行日志里的sql语句
 */
func executeSqlLog(info *bean.SyncServerInfo) {
	list := SqlLogDao.GetNotExecuteList()
	if len(list) == 0 {
		return
	}
	for _, it := range list {

		//sql语句的参数列表
		paramtaList := make([]any, 0)
		json.Unmarshal([]byte(it.Param), &paramtaList)

		//用来判断是否指定的sql语句
		handleSql := it.Sql
		handleSql = strings.ReplaceAll(handleSql, " ", "")
		handleSql = strings.ReplaceAll(handleSql, "\r\n", "")
		handleSql = strings.ReplaceAll(handleSql, "\r", "")
		handleSql = strings.ReplaceAll(handleSql, "\n", "")

		//日志执行结束后执行sql
		var afterSql string
		if strings.HasPrefix(handleSql, "insertintolocal_file") { //如果当前sql语句是往本地文件表里添加一条数据
			SyncHandle.ByLog(info, paramtaList)
		} else if strings.HasPrefix(handleSql, "insertintodfs_file(") { //如果该sql语句是添加文件
			afterSql = SyncHandle.HandleBySyncLog(info, paramtaList)
		} else {
		}
		_, execSqlErr := DBUtil.DBConn.Exec(it.Sql, paramtaList...)
		if execSqlErr != nil { //sql语句执行失败
			DBUtil.DBConn.Exec("update sql_log set state = 2, err = ? where id = ?", execSqlErr.Error(), it.Id)
			return
		}
		if afterSql != "" {
			DBUtil.DBConn.Exec(afterSql)
		}
		DBUtil.DBConn.Exec("update sql_log set state = 1 where id = ?", it.Id)
	}

	//记录当前同步的数据条数
	info.SyncCount += len(list)
	//this.syncSocket.send(info)
	executeSqlLog(info)
}

// 保存最后一次请求的日志ID
func SaveLastId(info *bean.SyncServerInfo, lastId int64) {

	//记录最后一次请求到的日志ID文件
	lastLogIdFile := syncLastIdFilePath + "." + String.ToMd5(info.Url)

	//执行成功之后立即将当前日志的日期保存到本地,降低sql被重复执行的BUG
	os.WriteFile(lastLogIdFile, []byte(String.ValueOf(lastId)), 0644)
}

// 保存最后一次请求的日志ID
func getLastId(info *bean.SyncServerInfo) int64 {

	//记录最后一次请求到的日志ID文件
	lastLogIdFile := syncLastIdFilePath + "." + String.ToMd5(info.Url)
	data, err := os.ReadFile(lastLogIdFile)
	if err != nil {
		return 0
	}
	lastId, _ := strconv.ParseInt(string(data), 10, 64)
	return lastId
}

///**
// * 发送同步通知
// */
//fun sendNotify() {
//    this.syncInfoList.forEach { info ->
//        val url = "${info.url}/push_notify"
//        try {
//            SyncHttp.request(url)
//        } catch (e: Exception) {
//            info.state = 2//标记为同步失败
//            info.msg = "发送同步通知失败：$e"
//        }
//    }
//}
