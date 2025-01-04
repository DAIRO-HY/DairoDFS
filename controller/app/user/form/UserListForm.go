package form

type UserListForm struct {

	/** 主键 **/
	Id *int64

	/** 用户名 **/
	Name *string

	/** 用户电子邮箱 **/
	Email *string

	/** 用户状态 **/
	State *string

	/** 创建日期 **/
	Date *string
}
