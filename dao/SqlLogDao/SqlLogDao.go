package SqlLogDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 获取错误的日志记录
 */
func GetErrorLog() *dto.SqlLogDto {
	return DBUtil.SelectOne[dto.SqlLogDto]("select * from user where state = 2 order by id limit 1")
}
