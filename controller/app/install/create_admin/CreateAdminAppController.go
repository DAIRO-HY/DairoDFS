package create_admin

import (
	"DairoDFS/controller/app/install/create_admin/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/String"
	"DairoDFS/service/UserService"
	"net/http"
)

/**
 * 管理员账号初始化
 */
//@Get:/app/install/create_admin
//@templates:app/install/create_admin.html
func Init(writer http.ResponseWriter, request *http.Request) {
	if *UserDao.IsInit() { //管理员账号已经存在
		http.Redirect(writer, request, "/app/login", http.StatusFound)
	}
}

// 账号初始化API
// @Post:/app/install/create_admin/add_admin
func AddAdmin(inForm form.CreateAdminForm) any {
	if *UserDao.IsInit() { //管理员用户只能被创建一次
		return exception.NOT_ALLOW()
	}
	pwd := String.ToMd5(inForm.Pwd)
	state := int8(1)
	userDto := dto.UserDto{
		Name:  &inForm.Name,
		Pwd:   &pwd,
		State: &state,
	}
	UserService.Add(userDto)
	return nil
}
