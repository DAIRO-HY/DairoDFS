package SyncByLog

import (
	"DairoDFS/application"
	"DairoDFS/controller/distributed"
	"DairoDFS/dao/SqlLogDao"
	"DairoDFS/dao/dto"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBConnection"
	"DairoDFS/util/Sync/SyncHandle"
	"DairoDFS/util/Sync/SyncHttp"
	"DairoDFS/util/Sync/bean"
	"encoding/json"
	"io"
	"net"
	"net/http"
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

/**
* 监听服务端日志变化
 */
func listen(info *bean.SyncServerInfo) {
	loopFunc := func() bool {
		transport := &http.Transport{
			DialContext:           (&net.Dialer{Timeout: 3 * time.Second}).DialContext, //连接超时
			ResponseHeaderTimeout: (distributed.KEEP_ALIVE_TIME + 10) * time.Second,    //读数据超时
		}
		client := &http.Client{Transport: transport}
		url := info.Url + "/distributed/listen?lastId=" + String.ValueOf(getLastId(info))

		// 创建HTTP GET请求
		resp, err := client.Get(url)
		if err != nil {
			//fmt.Println("服务端心跳检查失败，间隔30秒之后会重试。错误：" + err.Error())
			info.Msg = "服务端心跳检查失败，间隔30秒之后会重试。错误：" + err.Error()
			time.Sleep(5 * time.Second)
			return false
		}
		defer resp.Body.Close()
		//if resp.StatusCode != http.StatusOK {
		//	bodyData, _ := io.ReadAll(resp.Body)
		//	info.Msg = "服务端心跳检查失败,Status:" + String.ValueOf(resp.StatusCode) + ",Body:" + string(bodyData) + "。错误：" + err.Error()
		//	time.Sleep(30 * time.Second)
		//	break
		//}
		buf := make([]byte, 1)
		for {
			n, readErr := resp.Body.Read(buf)
			if n > 0 {

				//记录最有一次心跳时间
				info.LastHeartTime = time.Now().UnixMilli()
				info.Msg = "心跳检测中。"
				tag := buf[0]
				if tag == 1 { //接收到的标记为1时，代表服务器端有新的日志
					requestSqlLog(info)
					break
				}
				continue
			}
			if readErr != nil {
				if readErr != io.EOF {
					info.Msg = "服务端心跳检查失败。" + readErr.Error()
				}
				break
			}
		}
		if info.State == 2 { //如果同步发生了错误
			return true
		}
		return false
	}
	for {
		time.Sleep(3 * time.Second)
		if loopFunc() {
			break
		}
	}
}

/**
* 启动执行
* @param isForce 是否强制执行
 */
//准备废弃该函数
func Start(isForce bool) {
	//if SyncByTable.IsRuning() { //全量同步正在进行中 @TODO:应该判断全量同步是否正在进行中
	//	return
	//}
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

var syncLock sync.Mutex

// 循环取sql日志
// @return 是否处理完成
func requestSqlLog(info *bean.SyncServerInfo) {

	//单线程同步
	syncLock.Lock()

	//得到最后请求的id
	lastId := getLastId(info)
	url := info.Url + "/distributed/get_log?lastId=" + String.ValueOf(lastId)
	logData, err := SyncHttp.Request(url)
	if err != nil {
		info.State = 2 //标记为同步失败
		info.Msg = err.Error()
		//this.syncSocket.send(info)
		syncLock.Unlock()
		return
	}
	if string(logData) == "[]" { //已经没有sql日志

		//执行日志sql
		runSqlErr := runSql(info)
		if runSqlErr != nil {
			info.State = 2
			info.Msg = runSqlErr.Error()
			return
		}

		info.State = 0 //同步完成，标记为待机中
		info.Msg = ""
		info.LastTime = time.Now().UnixMilli() //最后一次同步完成时间
		//this.syncSocket.send(info)
		syncLock.Unlock()
		return
	}
	logList := make([]dto.SqlLogDto, 0)
	json.Unmarshal(logData, &logList)

	//将sql日志添加到数据库
	insertLogErr := insertLog(info, logList)
	if insertLogErr != nil {
		info.State = 2
		info.Msg = insertLogErr.Error()
		return
	}
	lastLog := logList[len(logList)-1]

	//执行成功之后立即将当前日志的日期保存到本地,降低sql被重复执行的BUG
	SaveLastId(info, lastLog.Id)

	//执行日志sql
	runSqlErr := runSql(info)
	if runSqlErr != nil {
		info.State = 2
		info.Msg = runSqlErr.Error()
		return
	}

	syncLock.Unlock()

	//递归调用，直到服务端日志同步完成
	requestSqlLog(info)
}

// 从主机请求到的日志保存到本地日志
func insertLog(info *bean.SyncServerInfo, dataList []dto.SqlLogDto) error {
	for _, it := range dataList {
		_, insertErr := DBConnection.DBConn.Exec("insert into sql_log(id,date,sql,param,state,source) values(?,?,?,?,?,?)",
			it.Id, it.Date, it.Sql, it.Param, 0, info.Url)
		if insertErr != nil {
			if strings.Contains(insertErr.Error(), "UNIQUE constraint failed: sql_log.id") {
				// 这条数据已经添加
				continue
			} else {
				return insertErr
			}
		}
	}
	return nil
}

/**
* 执行日志里的sql语句
 */
func runSql(info *bean.SyncServerInfo) error {

	//获取还未执行的sql语句
	notRunList := SqlLogDao.GetNotRunList()
	if len(notRunList) == 0 {
		return nil
	}
	for _, it := range notRunList {

		//sql语句的参数列表
		paramList := make([]any, 0)
		json.Unmarshal([]byte(it.Param), &paramList)

		//用来判断是否指定的sql语句
		handleSql := it.Sql
		handleSql = strings.ReplaceAll(handleSql, " ", "")
		handleSql = strings.ReplaceAll(handleSql, "\r\n", "")
		handleSql = strings.ReplaceAll(handleSql, "\r", "")
		handleSql = strings.ReplaceAll(handleSql, "\n", "")

		//日志执行结束后执行sql
		var afterSql string
		if strings.HasPrefix(handleSql, "insertintolocal_file") { //如果当前sql语句是往本地文件表里添加一条数据
			handleLocalFileErr := SyncHandle.ByLog(info, paramList)
			if handleLocalFileErr != nil {
				DBConnection.DBConn.Exec("update sql_log set state = 2, err = ? where id = ?", handleLocalFileErr.Error(), it.Id)
				return handleLocalFileErr
			}
		} else if strings.HasPrefix(handleSql, "insertintodfs_file(") { //如果该sql语句是添加文件
			sql, err := SyncHandle.HandleBySyncLog(info, paramList)
			if err != nil {
				return err
			}
			afterSql = sql
		} else {
		}
		_, execSqlErr := DBConnection.DBConn.Exec(it.Sql, paramList...)
		if execSqlErr != nil { //sql语句执行失败
			DBConnection.DBConn.Exec("update sql_log set state = 2, err = ? where id = ?", execSqlErr.Error(), it.Id)
			return execSqlErr
		}
		if afterSql != "" {
			DBConnection.DBConn.Exec(afterSql)
		}
		DBConnection.DBConn.Exec("update sql_log set state = 1 where id = ?", it.Id)
	}

	//记录当前同步的数据条数
	info.SyncCount += len(notRunList)
	//this.syncSocket.send(info)
	return runSql(info)
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
