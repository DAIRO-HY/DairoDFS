package form

type MyShareForm struct {

	/** id **/
	Id int64 `json:"id"`

	/** 分享的标题（文件名） **/
	Title string `json:"title"`

	/** 文件数量 **/
	FileCount int `json:"fileCount"`

	/** 是否分享的仅仅是一个文件夹 **/
	FolderFlag bool `json:"folderFlag"`

	/** 结束时间 **/
	EndDate string `json:"endDate"`

	/** 创建日期 **/
	Date string `json:"date"`

	/** 缩略图 **/
	Thumb string `json:"thumb"`
}
