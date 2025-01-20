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
	Date string `json:"date"`

	/** 缩率图 **/
	Thumb string `json:"thumb"`
}
