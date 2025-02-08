package SqlLogDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 获取未执行和执行失败的记录
 */
func GetNotRunList() []dto.SqlLogDto {
	return DBUtil.SelectList[dto.SqlLogDto]("select * from sql_log where state in (0,2) order by id limit 1000")
}

/**
 * 获取错误的日志记录
 */
func GetErrorLog() (dto.SqlLogDto, bool) {
	return DBUtil.SelectOne[dto.SqlLogDto]("select * from user where state = 2 order by id limit 1")
}

// 获取当前日志中最大一个ID
func LastID() int64 {
	return DBUtil.SelectSingleOneIgnoreError[int64]("select max(id) from sql_log")
}

// 获取sql日志
func GetLog(lastId int64) []map[string]any {
	return DBUtil.SelectToListMap("select id,date,sql,param from sql_log where id > ? order by id limit 100", lastId)
}
