package advanced

import (
	"DairoDFS/extension/Bool"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/DfsFileHandleUtil"
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

// @Post:/init
func Init() map[string]any {
	return map[string]any{
		"fileHandling": Bool.Is(DfsFileHandleUtil.HasData(), "正在处理", "空闲中"),
	}
}

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

// 开始处理线程
// @Post:/re_handle
func ReHandle() {
	DfsFileHandleUtil.NotifyWorker()
}
