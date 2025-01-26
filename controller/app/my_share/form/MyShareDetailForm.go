package form

type MyShareDetailForm struct {

	/** id **/
	Id int64 `json:"id,omitempty"`

	/** 链接 **/
	Url string `json:"url,omitempty"`

	/** 加密分享 **/
	Pwd string `json:"pwd,omitempty"`

	/** 分享的文件夹 **/
	Folder string `json:"folder,omitempty"`

	/** 分享的文件夹或文件名,用|分割 **/
	Names string `json:"names,omitempty"`

	/** 结束日期 **/
	EndDate string `json:"end_date,omitempty"`

	/** 创建日期 **/
	Date string `json:"date,omitempty"`
}
