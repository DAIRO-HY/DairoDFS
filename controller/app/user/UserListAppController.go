package user

import (
	"DairoDFS/controller/app/user/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/Date"
)

/**
 * 用户列表
 */

/**
 * 初始化
 */
//@get:/app/user_list
//@templates:app/user_list.html
func ListHtml() {}

/**
 * 页面初始化
 */
//@post:/app/user_list/init
func ListInit() []form.UserListOutForm {
	dtoList := UserDao.SelectAll()
	var userList []form.UserListOutForm
	for _, it := range dtoList {
		date := Date.Format(*it.Date)
		state := Bool.Is(*it.State == 1, "启用", "禁用")
		userList = append(userList, form.UserListOutForm{
			Id:    it.Id,
			Name:  it.Name,
			Email: it.Email,
			Date:  &date,
			State: &state,
		})
	}
	return userList
}
