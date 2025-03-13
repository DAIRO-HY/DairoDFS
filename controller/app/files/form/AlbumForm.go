package form

type AlbumForm struct {

	/** 文件id **/
	Id int64 `json:"id"`

	/** 名称 **/
	Name string `json:"name"`

	/** 大小 **/
	Size int64 `json:"size"`

	/** 是否文件 **/
	FileFlag bool `json:"fileFlag"`

	/** 创建日期 **/
	Date int64 `json:"date"`

	/** 缩率图 **/
	Thumb string `json:"thumb"`

	/** 属性 **/
	//Property string `json:"property"`

	///** 拍摄时间 **/
	//CameraDate int64 `json:"cameraDate"`

	/** 相机名 **/
	CameraName string `json:"cameraName"`

	/** 视频时长 **/
	Duration string `json:"duration"`
}
