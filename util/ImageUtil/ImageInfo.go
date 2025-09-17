package ImageUtil

// ImageInfo 照片信息
type ImageInfo struct {

	//宽
	Width int `json:"width,omitempty"`

	//高
	Height int `json:"height,omitempty"`

	//拍摄时间
	Date int64 `json:"date,omitempty"`

	//相机制造商
	Make string `json:"make,omitempty"`

	//设备型号
	Model string `json:"camera,omitempty"`

	//光圈大小
	FNumber string `json:"fNumber,omitempty"`

	//快门速度
	ShutterSpeed string `json:"shutterSpeed,omitempty"`

	//曝光
	ISO int `json:"iso,omitempty"`

	//纬度
	Lat float64 `json:"lat,omitempty"`

	//经度
	Long float64 `json:"long,omitempty"`

	//图片方向
	// Orientation 值的意义如下：
	// 1 = 正常方向
	// 6 = 需要顺时针旋转90度（宽高对调）
	// 8 = 逆时针旋转90度（宽高对调）
	// 3 = 旋转180度
	Orientation int `json:"orientation,omitempty"`
}
