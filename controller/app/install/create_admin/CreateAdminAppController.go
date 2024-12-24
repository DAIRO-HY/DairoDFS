package create_admin

import (
	"DairoDFS/dao/UserDao"
	"net/http"
)

/**
 * 管理员账号初始化
 */

/**
 * 页面初始化
 */
//get:/app/install/create_admin
//templates:app/install/create_admin.html
func Init(writer http.ResponseWriter, request *http.Request) {
	if UserDao.SelectOne(1) != nil { //管理员账号已经存在
		http.Redirect(writer, request, "/app/login", http.StatusFound)
	}
}

//    /**
//     * 账号初始化API
//     */
//    @PostMapping("/add_admin")
//    @ResponseBody
//    fun addAdmin(@Validated form: CreateAdminForm) {
//        if (this.userDao.selectOne(1) != null) {//管理员用户只能被创建一次
//            throw ErrorCode.NOT_ALLOW
//        }
//        val userDto = UserDto()
//        userDto.name = form.name
//        userDto.pwd = form.pwd!!.md5
//        userDto.state = 1
//        this.userService.add(userDto)
//    }
//}
