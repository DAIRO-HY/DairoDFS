package SyncByLog

import (
	"DairoDFS/application"
	"DairoDFS/controller/distributed/DistributedPush"
	"DairoDFS/dao/dto"
	"DairoDFS/util/Sync"
	"fmt"
	"net"
	"net/http"
	"runtime"
	"testing"
	"time"
)

func TestAddLog(t *testing.T) {
	application.Init()
	insertLog("http://localhost:8031", []dto.SqlLogDto{
		{
			Id:    123,
			Date:  time.Now().UnixMilli(),
			Sql:   "insert into dfs_file(id, userId, parentId, name, size, contentType, storageId, date, isExtra, property, state) values (?,?,?,?,?,?,?,?,?,?,?)",
			Param: `[1738809663240311,1738220388533791,1738220529397743,"tt",30046042,"application/octet-stream",1738231266932659,"213456948562",false,"",0]`,
		},
	})
}

func TestExecuteSqlLog(t *testing.T) {
	application.Init()
	runSql(&Sync.SyncServerInfo{Url: "asdafsfsdfdsf"})
}

func TestSaveLastId(t *testing.T) {
	application.Init()
	SaveLastId("http://localhost:8031", 123456)
}

func TestGetLastId(t *testing.T) {
	application.Init()
	lastId := getLastId("http://localhost:8031")
	fmt.Println(lastId)
}

func TestListen(t *testing.T) {
	application.Init()
	listen(&Sync.SyncServerInfo{
		Url: "http://localhost:8031",
	})
}

func TestRequestSqlLog(t *testing.T) {
	application.Init()
	requestSqlLog(&Sync.SyncServerInfo{
		Url: "http://localhost:8031",
	})
}

func TestBigRequest(t *testing.T) {
	application.Init()
	go func() {
		for {
			time.Sleep(5 * time.Second)
			runtime.GC()
			fmt.Println("-->GC")
		}
	}()
	for i := 0; i < 1000; i++ {
		transport := &http.Transport{
			DialContext:           (&net.Dialer{Timeout: 3 * time.Second}).DialContext,  //连接超时
			ResponseHeaderTimeout: (DistributedPush.KEEP_ALIVE_TIME + 10) * time.Second, //读数据超时
		}
		client := &http.Client{Transport: transport}
		url := "http://localhost:8031/distributed/listen?lastId=123"

		// 创建HTTP GET请求
		resp, err := client.Get(url)
		if err != nil {
			fmt.Println(err)
		}
		if resp != nil {
		}
		//resp.Body.Close()
	}
	fmt.Println("finish")
	time.Sleep(1 * time.Hour)
}
