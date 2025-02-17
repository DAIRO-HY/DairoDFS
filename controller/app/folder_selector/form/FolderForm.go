package form

type FolderForm struct {

	/** 名称 **/
	Name string `json:"name"`

	/** 大小 **/
	Size string `json:"size"`

	/** 是否文件 **/
	FileFlag bool `json:"fileFlag"`

	/** 创建日期 **/
	Date string `json:"date"`
}
