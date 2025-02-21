package create_admin

import (
	"DairoDFS/controller/app/install/create_admin/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/Number"
	"DairoDFS/extension/String"
	"DairoDFS/service/UserService"
	"net/http"
	"runtime"
)

//@Group: /app/install/create_admin

// 管理员账号初始化
// @Get:
// @Html:app/install/create_admin.html
func Init(writer http.ResponseWriter, request *http.Request) {
	runtime.GC()
	if UserDao.IsInit() { //管理员账号已经存在
		http.Redirect(writer, request, "/app/login", http.StatusFound)
	}
}

// 账号初始化API
// @Post:/add_admin
func AddAdmin(inForm form.CreateAdminForm) any {
	if UserDao.IsInit() { //管理员用户只能被创建一次
		return exception.NOT_ALLOW()
	}
	pwd := String.ToMd5(inForm.Pwd)
	state := int8(1)
	userDto := dto.UserDto{
		Id:    Number.ID(),
		Name:  inForm.Name,
		Pwd:   pwd,
		State: state,
	}
	UserService.Add(userDto)
	return nil
}
