package RecycleStorageTimer

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/dao/DfsFileDao"
	"DairoDFS/dao/DfsFileDeleteDao"
	"DairoDFS/dao/StorageFileDao"
	"DairoDFS/extension/String"
	"DairoDFS/service/DfsFileDeleteService"
	"DairoDFS/util/DBConnection"
	"os"
	"time"

	"github.com/robfig/cron/v3"
)

// 标记是否正在运行中
var IsRunning bool

// 是否有错误
var Error string

// 最后一次执行耗时
var LastRunTime int64

/**
 * 回收存储空间计时器(毫秒)
 */
func Init() {

	// 创建一个新的cron实例
	cn := cron.New(cron.WithSeconds())

	// 添加一个每天凌晨3点执行的任务
	cn.AddFunc("0 0 3 * * *", start)
	//cn.AddFunc("*/5 * * * * *", start)

	// 启动cron调度器
	cn.Start()
}

// 开始回收
func start() {
	defer func() {
		IsRunning = false
		if r := recover(); r != nil { //如果发生了程序终止错误
			switch value := r.(type) {
			case string:
				Error = value
			case error:
				Error = value.Error()
			}
		} else {
			Error = ""
		}
	}()
	IsRunning = true

	now := time.Now().UnixMilli()
	//deleteSqlLog()         // 删除超出指定时间的sql日志
	//deleteTrashTimeout()   // 删除回收站到期的文件
	//recycleDeletedFile()   // 回收已经被删除的文件
	//recycleNotUseStorage() // 回收没有被使用的存储文件
	LastRunTime = time.Now().UnixMilli() - now
}

// 删除超出指定时间的sql日志
func deleteSqlLog() {

	//删除超过一天的日志
	deleteTime := time.Now().UnixMilli() - 1*24*60*60*1000
	DBConnection.DBConn.Exec("delete from sql_log where date < ?", deleteTime)
}

// 删除回收站到期的文件
func deleteTrashTimeout() {
	for {
		deleteTime := time.Now().UnixMilli() - SystemConfig.Instance().TrashTimeout*24*60*60*1000
		deleteIdsList := DfsFileDao.SelectIdsByDeleteAndTimeout(deleteTime)
		if len(deleteIdsList) == 0 {
			break
		}
		DfsFileDeleteService.AddDelete(deleteIdsList)
	}
}

// 回收已经被删除的文件
func recycleDeletedFile() {
	for {

		//获取要删除的截止时间
		deleteTime := time.Now().UnixMilli() - SystemConfig.Instance().DeleteStorageTimeout*24*60*60*1000
		deleteList := DfsFileDeleteDao.SelectIdsByTimeout(deleteTime)
		if len(deleteList) == 0 {
			break
		}

		//记录要删除的本地文件id
		storageIds := make(map[int64]struct{})
		deleteIds := ""
		for _, it := range deleteList {
			storageIds[it.StorageId] = struct{}{}
			deleteIds += String.ValueOf(it.Id) + ","
		}

		//去除最后的逗号
		deleteIds = deleteIds[:len(deleteIds)-1]

		//彻底删除文件表数据
		//删除文件不需要同步日志,所以不使用mybatis提交,让每个分机端走各自的删除逻辑,防止文件误删
		DBConnection.DBConn.Exec("delete from dfs_file_delete where id in (" + deleteIds + ")")
		for it, _ := range storageIds {
			DeleteNotUseStorage(it)
		}
	}
}

// 回收没有被使用的存储文件
func recycleNotUseStorage() {

	//获取要删除的截止时间
	deleteTime := time.Now().UnixMilli() - SystemConfig.Instance().DeleteStorageTimeout*24*60*60*1000

	////指定删除截止，避免文件正在上传或者正在处理时，还没有来得及写入dfs_file表就被删除的问题
	rows, _ := DBConnection.Query("select id from storage_file where id < ? order by id", deleteTime)
	for rows.Next() {
		var id int64
		if err := rows.Scan(&id); err != nil {
			panic(err)
		}

		//判断文件是否还在被使用，没有使用则删除文件
		DeleteNotUseStorage(id)
	}
}

// 删除本地文件
func DeleteNotUseStorage(id int64) {
	if DfsFileDao.IsFileUsing(id) { //文件还在使用中
		return
	}
	if DfsFileDeleteDao.IsFileUsing(id) { //文件还在使用中
		return
	}
	storageDto, isExists := StorageFileDao.SelectOne(id)
	if !isExists {
		return
	}
	if _, statErr := os.Stat(storageDto.Path); !os.IsNotExist(statErr) { //如果文件存在
		if removeErr := os.Remove(storageDto.Path); removeErr != nil { //删除文件
			panic("文件" + storageDto.Path + "删除失败:" + removeErr.Error())
		}
	}

	//删除本地文件表数据
	//删除文件不需要同步日志,所以不使用mybatis提交,让每个分机端走各自的删除逻辑,防止文件误删
	DBConnection.DBConn.Exec("delete from storage_file where id = ?", storageDto.Id)
}
