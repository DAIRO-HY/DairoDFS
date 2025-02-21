package form

type LoginAppOutForm struct {

	/** 用户名 **/
	Token string `json:"token"`

	/** 是否管理员 **/
	IsAdmin bool `json:"isAdmin"`
}
