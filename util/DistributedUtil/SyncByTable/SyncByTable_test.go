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
	aopId, _ := getAopId(info)
	err := loopSync(info, "storage_file", 0, aopId)
	if err != nil {
		t.Fatal(err)
		return
	}
}

/**
 * 获取一个断面ID，防止再全量同步的过程中，主机又增加数据，导致全量同步数据不完整
 * 其实就是服务器端的时间戳
 */
func TestGetAopId(t *testing.T) {
	application.Init()
	info := DistributedUtil.GetMasterInfo()
	aopId, err := getAopId(info)
	if err != nil {
		t.Fatal(err)
		return
	}
	fmt.Println(aopId)
}

/**
 * 从主机获取某表的一批数据id
 * @param info 主机信息
 * @param tbName 表名
 * @param lastId 上次获取到的最后一个id
 * @param aopId 本次同步的服务器端的最大id
 */
func TestGetTableId(t *testing.T) {
	application.Init()
	info := DistributedUtil.GetMasterInfo()
	aopId, _ := getAopId(info)
	ids, err := getTableId(info, "dfs_file", 0, aopId)
	if err != nil {
		t.Fatal(err)
		return
	}
	fmt.Println(ids)
}

/**
 * 筛选出本地不存在的ID
 */
func TestFilterNotExistsId(t *testing.T) {
}

/**
 * 从同步主机端取数据
 */
func TestGetTableData(t *testing.T) {
	application.Init()
	info := DistributedUtil.GetMasterInfo()
	aopId, _ := getAopId(info)
	ids, _ := getTableId(info, "user", 0, aopId)
	data, err := getTableData(info, "user", ids)
	if err != nil {
		t.Fatal(err)
		return
	}
	fmt.Println(data)
}
