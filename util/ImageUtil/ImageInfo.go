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

	//快门速度
	ISO int `json:"iso,omitempty"`

	//纬度
	Lat float64 `json:"lat,omitempty"`

	//经度
	Long float64 `json:"long,omitempty"`
}
