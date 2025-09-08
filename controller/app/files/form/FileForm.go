package form

type FileForm struct {

	/** 文件id **/
	Id int64 `json:"id"`

	/** 名称 **/
	Name string `json:"name"`

	/** 大小 **/
	Size int64 `json:"size"`

	/** 是否文件 **/
	FileFlag bool `json:"fileFlag"`

	/** 创建日期 **/
	//Date string `json:"date"`
	Date int64 `json:"date"`

	/** 缩率图 **/
	Thumb string `json:"thumb"`

	/** 其他属性1,视频时为视频总时长(毫秒) **/
	//Other1 string `json:"other1"`
	Other1 any `json:"other1"`
}
