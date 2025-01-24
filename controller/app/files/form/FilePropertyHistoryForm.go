package form

type FilePropertyHistoryForm struct {

	/** 文件ID **/
	Id int64 `json:"id"`

	/** 大小 **/
	Size string `json:"size"`

	/** 创建日期 **/
	Date string `json:"date"`
}
