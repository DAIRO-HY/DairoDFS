package dto

/**
 * 包含缩略图的文件信息
 */
type DfsFileThumbDto struct {

	/**
	 * id
	 */
	Id int64

	/**
	 * 所属用户ID
	 */
	UserId int64

	/**
	 * 目录ID
	 */
	ParentId int64

	/**
	 * 名称
	 */
	Name string

	/**
	 * 大小
	 */
	Size int64

	/**
	 * 文件类型(文件专用)
	 */
	ContentType string

	/**
	 * 本地文件存储id(文件专用)
	 */
	LocalId int64

	/**
	 * 创建日期
	 */
	Date int64

	/**
	 * 文件属性，比如图片尺寸，视频分辨率等信息，JSON字符串
	 */
	Property string

	/**
	 * 是否附属文件，比如视频的标清文件，高清文件，PSD图片的预览图片，cr3的预览图片等
	 */
	IsExtra bool

	/**
	 * 是否历史版本(文件专用),1:历史版本 0:当前版本
	 */
	IsHistory bool

	/**
	 * 删除日期
	 */
	DeleteDate int64

	/**
	 * 文件处理状态，0：待处理 1：处理完成 2：处理出错，比如视频文件，需要转码；图片需要获取尺寸等信息
	 */
	State int8

	/**
	 * 文件处理出错信息
	 */
	StateMsg string

	/**
	 * 是否有缩率图
	 */
	HasThumb bool
}
