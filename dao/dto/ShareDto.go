package dto

type ShareDto struct {

	/**
	 * id
	 */
	Id int64

	/**
	 * 分享标题
	 */
	Title string

	/**
	 * 所属用户ID
	 */
	UserId int64

	/**
	 * 加密分享
	 */
	Pwd string

	/**
	 * 分享的文件夹
	 */
	Folder string

	/**
	 * 分享的文件夹或文件名,用|分割
	 */
	Names string

	/**
	 * 缩略图
	 */
	Thumb int64

	/**
	 * 是否是一个文件夹
	 */
	FolderFlag bool

	/**
	 * 文件数
	 */
	FileCount int

	/**
	 * 结束日期
	 */
	EndDate int64

	/**
	 * 创建日期
	 */
	Date int64
}
