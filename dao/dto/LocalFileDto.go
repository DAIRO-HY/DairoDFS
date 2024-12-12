package dto

type LocalFileDto struct {

	/**
	 *  id
	 */
	Id int64

	/**
	 * 本地存储目录
	 */
	Path string

	/**
	 * 文件MD5
	 */
	Md5 string
}
