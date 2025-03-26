package advanced

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/Date"
	"DairoDFS/extension/Number"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/DfsFileHandleUtil"
	"DairoDFS/util/RecycleStorageTimer"
	"github.com/shirou/gopsutil/disk"
	"os"
	"strings"
)

/**
 * 数据同步状态
 */
//@Group:/app/advanced

/**
 * 页面初始化
 */
//@Html:.html
func Html() {}

// @Post:/init
func Init() map[string]any {

	//磁盘使用状况
	storageState := "磁盘使用（已使用/总大小）："
	systemConfig := SystemConfig.Instance()
	for _, it := range systemConfig.SaveFolderList {
		usage, usageErr := disk.Usage(it)
		if usageErr != nil {
			storageState += it + "(" + usageErr.Error() + ");"
		} else {
			storageState += it + "(" + Number.ToDataSize(usage.Used) + "/" + Number.ToDataSize(usage.Total) + ");"
		}
	}

	recycleStorageTimerState := "回收定时器"
	recycleStorageTimerState += Bool.Is(RecycleStorageTimer.IsRunning, "运行中；", "等待中；")
	recycleStorageTimerState += Bool.Is(RecycleStorageTimer.Error == "", "", "错误："+RecycleStorageTimer.Error+"；")
	recycleStorageTimerState += "上次执行时间：" + Date.FormatByTimespan(RecycleStorageTimer.LastRunTime)
	return map[string]any{
		"fileHandling":             Bool.Is(DfsFileHandleUtil.HasData(), "正在处理", "空闲中"),
		"recycleStorageTimerState": recycleStorageTimerState,
		"storageState":             storageState,
	}
}

// 页面数据初始化
// @Post:/exec_sql
func ExecSql(sql string) any {

	//去除前后空格
	sql = strings.TrimSpace(sql)
	if strings.HasPrefix(strings.ToLower(sql), "select") { //如果是查询语句
		data, columns := DBUtil.SelectToListMap("select * from (" + sql + ") limit 10000")
		return map[string]any{
			"data":    data,
			"columns": columns,
		}
	} else {
		DBUtil.Exec(sql)
	}
	return nil
}

// 开始处理线程
// @Post:/re_handle
func ReHandle() {
	DfsFileHandleUtil.NotifyWorker()
}

// 获取DFS正在使用的文件大小
// @Post:/used_size
func UsedSize() string {
	var total int64
	for _, it := range DBUtil.SelectList[string]("select path from main.storage_file") {
		stat, _ := os.Stat(it)
		total += stat.Size()
	}
	return Number.ToDataSize(total) + "(" + String.ValueOf(total) + "B" + ")"
}

// 立即回收未使用的文件
// @Post:/recycle_now
func RecycleNow() {
	go func() {
		notUseIds := DBUtil.SelectList[int64]("select id from storage_file where id not in (select storageId from dfs_file where storageId > 0) and id not in (select storageId from dfs_file_delete)")
		for _, id := range notUseIds {

			//判断文件是否还在被使用，没有使用则删除文件
			RecycleStorageTimer.DeleteNotUseStorage(id)
		}
	}()
}
