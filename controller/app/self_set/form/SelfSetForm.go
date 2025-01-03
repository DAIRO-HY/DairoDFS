package form

type SelfSetForm struct {

	/** 主键 **/
	Id *int64 `json:"id"`

	/** 用户名 **/
	Name *string `json:"name"`

	/** 用户电子邮箱 **/
	Email *string `json:"email"`

	/** 创建日期 **/
	Date *string `json:"date"`

	/** 用户文件访问路径前缀 **/
	UrlPath *string `json:"urlPath"`

	/** API操作TOKEN **/
	ApiToken *string `json:"apiToken"`

	/** 端对端加密密钥 **/
	EncryptionKey *string `json:"encryptionKey"`
}
