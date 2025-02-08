package DBSqlLog

import (
	"DairoDFS/controller/distributed/DistributedPush"
	"DairoDFS/dao/dto"
	"DairoDFS/extension/Number"
	"DairoDFS/util/GoroutineLocal"
	"DairoDFS/util/LogUtil"
	"database/sql"
	"encoding/json"
	_ "github.com/mattn/go-sqlite3"
	"time"
)

// 事务数据库协程本地变量
const _SQL_LOG_KEY = "SQL_LOG"

// 添加数据库日志
func Add(sql string, param []any) {
	paramData, _ := json.Marshal(param)
	logDto := dto.SqlLogDto{
		Sql:   sql,
		Param: string(paramData),
	}
	var logList *[]dto.SqlLogDto
	value, isExists := GoroutineLocal.Get(_SQL_LOG_KEY)
	if isExists {
		logList = value.(*[]dto.SqlLogDto)
	} else {
		logList = &[]dto.SqlLogDto{}
		GoroutineLocal.Set(_SQL_LOG_KEY, logList)
	}
	*logList = append(*logList, logDto)
}

// 保存日志数据库
func Insert(db *sql.DB) {
	value, isExists := GoroutineLocal.Get(_SQL_LOG_KEY)
	if !isExists {
		return
	}
	logList := value.(*[]dto.SqlLogDto)
	if len(*logList) == 0 {
		return
	}
	for _, it := range *logList {
		_, err := db.Exec("insert into sql_log(id,sql,param,date,state,source) values(?,?,?,?,1,'0.0.0.0')", Number.ID(), it.Sql, it.Param, time.Now().UnixMilli())
		LogUtil.Error2(err)
	}

	//保存完之后清空内容
	*logList = []dto.SqlLogDto{}
	DistributedPush.Push()
}

// 清空sql日志
func Clear() {
	GoroutineLocal.Remove(_SQL_LOG_KEY)
}
