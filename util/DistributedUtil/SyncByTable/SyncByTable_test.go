package SyncByTable

import (
	"DairoDFS/application"
	"DairoDFS/application/SystemConfig"
	"DairoDFS/util/DistributedUtil"
	"fmt"
	"testing"
	"time"
)

func TestDoSync(t *testing.T) {
	application.Init()
	SystemConfig.Instance().SyncDomains = []string{DistributedUtil.GetMasterInfo().Url}
	go SyncAll()
	time.Sleep(100 * time.Microsecond)
	SyncAll()
	time.Sleep(1 * time.Hour)
}

/**
 * 循环同步数据，直到包数据同步完成
 */
func TestLoopSync(t *testing.T) {
	application.Init()
	info := DistributedUtil.GetMasterInfo()
	aopId := getAopId(info)
	loopSync(info, "storage_file", 0, aopId)
}

/**
 * 获取一个断面ID，防止再全量同步的过程中，主机又增加数据，导致全量同步数据不完整
 * 其实就是服务器端的时间戳
 */
func TestGetAopId(t *testing.T) {
	application.Init()
	info := DistributedUtil.GetMasterInfo()
	aopId := getAopId(info)
	fmt.Println(aopId)
}

/**
 * 筛选出本地不存在的ID
 */
func TestGetTableCount(t *testing.T) {
	application.Init()
	info := DistributedUtil.GetMasterInfo()
	aopId := getAopId(info)
	count := GetTableCount(info, []string{
		"user",
		"user_token",
		"dfs_file",
		"dfs_file_delete",
		"share",
		"storage_file",
	}, aopId)
	fmt.Println(count)
}
