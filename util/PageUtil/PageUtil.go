package PageUtil

import (
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/String"
)

// 分页请求数据
type PageRequest struct {

	//请求序号，用于防止缓存响应
	Draw int

	//开始数据索引
	Start int

	//每页显示件数
	Length int

	//排序方式
	SortType string

	//排序字段
	SortName string

	//搜索内容
	Search string
}

// 分页返回数据
type PageResponse struct {

	//请求序号，用于防止缓存响应
	Draw int `json:"draw"`

	//总数据条数
	RecordsTotal int `json:"recordsTotal"`

	//搜索过滤后的数据条数（没启用搜索时同上）
	RecordsFiltered int `json:"recordsFiltered"`

	//数据
	Data any `json:"data"`
}

func (mine PageRequest) PageSql() string {
	pageSql := " "
	if len(mine.SortName) > 0 {
		sortBy := Bool.Is(mine.SortType == "desc", "desc", "asc")
		pageSql += "order by " + mine.SortName + " " + sortBy + " "
	}
	pageSql += "limit " + String.ValueOf(mine.Start) + "," + String.ValueOf(mine.Length)
	return pageSql
}
