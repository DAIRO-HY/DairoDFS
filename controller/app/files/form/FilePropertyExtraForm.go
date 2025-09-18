package form

// 扩展文件表单
type FilePropertyExtraForm struct {
	//文件ID
	Id int64 `json:"id"`

	// 扩展文件名
	Name string `json:"name"`

	// 大小
	Size string `json:"size"`

	// 文件类型
	ContentType string `json:"contentType"`
}
