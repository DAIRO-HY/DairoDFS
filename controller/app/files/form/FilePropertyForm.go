package form

type FilePropertyForm struct {

	/** 名称 **/
	Name string `json:"name"`

	/** 路径 **/
	Path string `json:"path"`

	/** 大小 **/
	Size string `json:"size"`

	/** 文件类型(文件专用) **/
	ContentType string `json:"contentType"`

	/** 创建日期 **/
	Date string `json:"date"`

	/** 文件状态 **/
	State int8 `json:"state"`

	/** 是否文件 **/
	IsFile bool `json:"isFile"`

	/** 文件数(文件夹属性专用) **/
	FileCount int `json:"fileCount"`

	/** 文件夹数(文件夹属性专用) **/
	FolderCount int `json:"folderCount"`

	/** 历史记录(文件属性专用) **/
	HistoryList []FilePropertyHistoryForm `json:"historyList"`

	/** 扩展文件列表 **/
	ExtraList []FilePropertyExtraForm `json:"extraList"`
}
