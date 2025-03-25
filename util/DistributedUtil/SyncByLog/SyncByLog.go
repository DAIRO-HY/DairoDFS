package SyncByLog

import (
	"DairoDFS/application"
	"DairoDFS/controller/distributed/DistributedPush"
	"DairoDFS/dao/SqlLogDao"
	"DairoDFS/dao/dto"
	"DairoDFS/extension/Date"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBConnection"
	"DairoDFS/util/DistributedUtil"
	"DairoDFS/util/DistributedUtil/DfsFileSyncHandle"
	"DairoDFS/util/DistributedUtil/StorageFileSyncHandle"
	"DairoDFS/util/DistributedUtil/SyncHttp"
	"DairoDFS/util/DistributedUtil/SyncInfoManager"
	"context"
	"encoding/json"
	"io"
	"net"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"
)

// 最后同步的ID存放目录
var syncLastIdFilePath = application.DataPath + "/sync_last_id"

// 监听所有配置的主机
func ListenAll() {
	SyncInfoManager.ReloadList()
	for _, it := range SyncInfoManager.SyncInfoList {
		go listen(it)
	}
}

// 监听服务端日志变化
func listen(info *DistributedUtil.SyncServerInfo) {
	for {
		if info.IsStop { // 如果被强行终止
			break
		}
		time.Sleep(1 * time.Second)
		loopListen(info)
	}
}

// 循环发起请求
// return 是否停止循环
func loopListen(info *DistributedUtil.SyncServerInfo) {
	if info.IsStop {
		return
	}
	defer func() {
		if r := recover(); r != nil { //如果发生了panic错误
			switch rValue := r.(type) {
			case error:
				info.Msg = "日志同步失败:" + rValue.Error()
			case string:
				info.Msg = "日志同步失败:" + rValue
			}
			info.State = 2
			info.Rollback()
		}
	}()

	//监听之前先执行一次请求，达到上次执行失败，本次重试的目的
	callRequestSqlLog(info)
	if info.State == 2 { //执行失败了，没有必要继续往下执行
		return
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	info.CancelFunc = cancel
	url := info.Url + "/listen?lastId=" + String.ValueOf(getLastId(info.Url))
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	transport := &http.Transport{
		//MaxIdleConns:          1,
		//MaxConnsPerHost:       1,
		//IdleConnTimeout:       1 * time.Millisecond,
		DialContext:           (&net.Dialer{Timeout: 3 * time.Second}).DialContext,  //连接超时
		ResponseHeaderTimeout: (DistributedPush.KEEP_ALIVE_TIME + 10) * time.Second, //读数据超时
	}
	client := &http.Client{Transport: transport}
	defer client.CloseIdleConnections()

	// 创建HTTP GET请求
	resp, err := client.Do(req)
	if err != nil {
		sleepTime := 5
		info.Msg = Date.Format(time.Now()) + ":服务端心跳检查失败，间隔" + String.ValueOf(sleepTime) + "秒之后会重试。错误：" + err.Error()
		time.Sleep(time.Duration(sleepTime) * time.Second)
		return
	}
	defer resp.Body.Close()
	buf := make([]byte, 1)
	for {
		if info.IsStop {
			break
		}
		n, readErr := resp.Body.Read(buf)
		if n > 0 {

			//记录最有一次心跳时间
			info.LastHeartTime = time.Now().UnixMilli()
			info.Msg = "心跳检测中。"
			tag := buf[0]
			if tag == 1 { //接收到的标记为1时，代表服务器端有新的日志
				callRequestSqlLog(info)
				break
			} else if tag == 0 {
				continue
			} else { //返回标记不等于0、1时，有可能时token失效
				info.Msg = "监听日志连接返回了非0,1错误，可能是认证token失效。"
				break
			}
		}
		if readErr != nil {
			if readErr != io.EOF {
				info.Msg = "服务端心跳检查失败。" + readErr.Error()
			}
			break
		}
	}
	if info.State == 2 { //如果同步发生了错误
		info.Cancel()
	}
}

// 调起requestSqlLog函数
func callRequestSqlLog(info *DistributedUtil.SyncServerInfo) {
	defer DistributedUtil.SyncLock.Unlock()
	DistributedUtil.SyncLock.Lock() //单线程同步
	requestSqlLog(info)
}

// 循环取sql日志
// @return 是否处理完成
func requestSqlLog(info *DistributedUtil.SyncServerInfo) {
	if info.IsStop { // 如果被强行终止
		return
	}

	//得到最后请求的id
	lastId := getLastId(info.Url)
	url := info.Url + "/get_log?lastId=" + String.ValueOf(lastId)
	logData, err := SyncHttp.Request(url)
	if err != nil {
		info.State = 2 //标记为同步失败
		info.Msg = err.Error()
		return
	}
	if string(logData) == "[]" { //已经没有sql日志

		//执行日志sql
		runSql(info)

		info.State = 0 //同步完成，标记为待机中
		info.Msg = ""
		info.LastTime = time.Now().UnixMilli() //最后一次同步完成时间
		return
	}
	info.State = 1 //标记为正在同步中
	info.Msg = "日志同步中"
	logList := make([]dto.SqlLogDto, 0)
	json.Unmarshal(logData, &logList)

	//将sql日志添加到数据库
	insertLogErr := insertLog(info.Url, logList)
	if insertLogErr != nil {
		info.State = 2
		info.Msg = insertLogErr.Error()
		return
	}
	lastLog := logList[len(logList)-1]

	//执行成功之后立即将当前日志的日期保存到本地,降低sql被重复执行的BUG
	SaveLastId(info.Url, lastLog.Id)

	//执行日志sql
	runSql(info)

	//递归调用，直到服务端日志同步完成
	requestSqlLog(info)
}

// 从主机请求到的日志保存到本地日志
// host 主机域名
// dataList 日志数据列表
func insertLog(host string, sqlLogList []dto.SqlLogDto) error {
	for _, it := range sqlLogList {
		_, insertErr := DBConnection.DBConn.Exec("insert into sql_log(id,date,sql,param,state,source) values(?,?,?,?,?,?)",
			it.Id, it.Date, it.Sql, it.Param, 0, host)
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
func runSql(info *DistributedUtil.SyncServerInfo) {

	//获取还未执行的sql语句
	notRunList := SqlLogDao.GetNotRunList()
	if len(notRunList) == 0 {
		return
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
		if strings.HasPrefix(handleSql, "insertintostorage_file") { //如果当前sql语句是往本地文件表里添加一条数据
			StorageFileSyncHandle.ByLog(info, paramList)
		} else if strings.HasPrefix(handleSql, "insertintodfs_file(") { //如果该sql语句是添加文件
			afterSql = DfsFileSyncHandle.ByLog(info, paramList)
		} else {
		}
		if _, err := info.DbTx().Exec(it.Sql, paramList...); err != nil { //sql语句执行失败
			panic(err)
		}
		if afterSql != "" {
			if _, err := info.DbTx().Exec(afterSql); err != nil {
				panic(err)
			}
		}

		//最后一定要提交事务
		if err := info.Commit(); err != nil {
			panic(err)
		}
		DBConnection.DBConn.Exec("update sql_log set state = 1 where id = ?", it.Id)
	}

	//记录当前同步的数据条数
	info.SyncCount += len(notRunList)
	runSql(info)
}

// 保存最后一次请求的日志ID
// host 主机域名
// lastId 最后获取到的ID
func SaveLastId(host string, lastId int64) {

	//记录最后一次请求到的日志ID文件
	lastLogIdFile := syncLastIdFilePath + "." + String.ToMd5(host)

	//执行成功之后立即将当前日志的日期保存到本地,降低sql被重复执行的BUG
	os.WriteFile(lastLogIdFile, []byte(String.ValueOf(lastId)), 0644)
}

// 保存最后一次请求的日志ID
// host 主机域名
func getLastId(host string) int64 {

	//记录最后一次请求到的日志ID文件
	lastLogIdFile := syncLastIdFilePath + "." + String.ToMd5(host)
	data, err := os.ReadFile(lastLogIdFile)
	if err != nil {
		return 0
	}
	lastId, _ := strconv.ParseInt(string(data), 10, 64)
	return lastId
}
