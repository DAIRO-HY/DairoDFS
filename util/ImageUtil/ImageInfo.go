package ImageUtil

/**
 * 图片信息
 */
type ImageInfo struct {

	//宽
	Width int `json:"width,omitempty"`

	//高
	Height int `json:"height,omitempty"`

	//拍摄时间
	Date int64 `json:"date,omitempty"`

	//相机名称
	Camera string `json:"camera,omitempty"`
}
