package SqlLogDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 获取错误的日志记录
 */
func GetErrorLog() *dto.SqlLogDto {
	return DBUtil.SelectOne(`select *
        from user
        where state = 2
        order by id asc
        limit 1`)
}
