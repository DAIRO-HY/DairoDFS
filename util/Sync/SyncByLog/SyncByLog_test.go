package SyncByLog

import (
	"DairoDFS/application"
	"DairoDFS/dao/dto"
	"DairoDFS/util/Sync/bean"
	"fmt"
	"testing"
	"time"
)

func TestAddLog(t *testing.T) {
	application.Init()
	addLog(bean.SyncServerInfo{Url: "asdafsfsdfdsf"}, []dto.SqlLogDto{
		{
			Id:    123,
			Date:  time.Now().UnixMilli(),
			Sql:   "insert into dfs_file(id, userId, parentId, name, size, contentType, localId, date, isExtra, property, state) values (?,?,?,?,?,?,?,?,?,?,?)",
			Param: `[1738809663240311,1738220388533791,1738220529397743,"tt",30046042,"application/octet-stream",1738231266932659,"213456948562",false,"",0]`,
		},
	})
}

func TestExecuteSqlLog(t *testing.T) {
	application.Init()
	executeSqlLog(bean.SyncServerInfo{Url: "asdafsfsdfdsf"})
}

func TestSaveLastId(t *testing.T) {
	application.Init()
	SaveLastId(bean.SyncServerInfo{
		Url: "http://sdfsfsf.com/dsfsdfdsfds",
	}, 123456)
}

func TestGetLastId(t *testing.T) {
	application.Init()
	lastId := getLastId(bean.SyncServerInfo{
		Url: "http://sdfsfsf.com/dsfsdfdsfds",
	})
	fmt.Println(lastId)
}
