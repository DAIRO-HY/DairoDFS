package advanced

import (
	"DairoDFS/util/DBUtil"
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
