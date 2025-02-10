package SyncByTable

import (
	"DairoDFS/application"
	"DairoDFS/extension/String"
	"DairoDFS/util/Sync/bean"
	"fmt"
	"testing"
)

func TestDoSync(t *testing.T) {
}

/**
 * 循环同步数据，直到包数据同步完成
 */
func TestLoopSync(t *testing.T) {
	application.Init()
	aopId, _ := getAopId(&bean.SyncServerInfo{
		Url: "http://localhost:" + String.ValueOf(application.Args.Port),
	})
	info := &bean.SyncServerInfo{
		Url: "http://localhost:" + String.ValueOf(application.Args.Port),
	}
	err := loopSync(info, "user", 0, aopId)
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
	aopId, err := getAopId(&bean.SyncServerInfo{
		Url: "http://localhost:" + String.ValueOf(application.Args.Port),
	})
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

	info := &bean.SyncServerInfo{
		Url: "http://localhost:" + String.ValueOf(application.Args.Port),
	}
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
	info := &bean.SyncServerInfo{
		Url: "http://localhost:" + String.ValueOf(application.Args.Port),
	}
	aopId, _ := getAopId(info)
	ids, _ := getTableId(info, "user", 0, aopId)
	data, err := getTableData(info, "user", ids)
	if err != nil {
		t.Fatal(err)
		return
	}
	fmt.Println(string(data))
}
