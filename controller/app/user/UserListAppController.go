package user

import (
	"DairoDFS/controller/app/user/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/Date"
)

//用户列表
//@Group:/app/user_list

/**
 * 初始化
 */
//@Html:.html
func ListHtml() {}

/**
 * 页面初始化
 */
//@Post:/init
func ListInit() []form.UserListOutForm {
	dtoList := UserDao.SelectAll()
	var userList []form.UserListOutForm
	for _, it := range dtoList {
		date := Date.FormatByTimespan(it.Date)
		state := Bool.Is(it.State == 1, "启用", "禁用")
		userList = append(userList, form.UserListOutForm{
			Id:    it.Id,
			Name:  it.Name,
			Email: it.Email,
			Date:  date,
			State: state,
		})
	}
	return userList
}
