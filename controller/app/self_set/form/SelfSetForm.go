package form

type SelfSetForm struct {

	/** 主键 **/
	Id int64

	/** 用户名 **/
	Name string

	/** 用户电子邮箱 **/
	Email string

	/** 创建日期 **/
	Date string

	/** 用户文件访问路径前缀 **/
	UrlPath string

	/** API操作TOKEN **/
	ApiToken string

	/** 端对端加密密钥 **/
	EncryptionKey string
}
