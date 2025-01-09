package form

type UserListOutForm struct {

	/** 主键 **/
	Id int64 `json:"id"`

	/** 用户名 **/
	Name string `json:"name"`

	/** 用户电子邮箱 **/
	Email string `json:"email"`

	/** 用户状态 **/
	State string `json:"state"`

	/** 创建日期 **/
	Date string `json:"date"`
}
