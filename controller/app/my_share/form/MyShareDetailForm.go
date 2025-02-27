package form

type MyShareDetailForm struct {

	/** id **/
	Id int64 `json:"id"`

	/** 链接 **/
	Url string `json:"url"`

	/** 加密分享 **/
	Pwd string `json:"pwd"`

	/** 分享的文件夹 **/
	Folder string `json:"folder"`

	/** 分享的文件夹或文件名,用|分割 **/
	Names string `json:"names"`

	/** 结束日期 **/
	EndDate string `json:"endDate"`

	/** 创建日期 **/
	Date string `json:"date"`
}
