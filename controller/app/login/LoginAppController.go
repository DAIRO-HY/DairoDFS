package login

import (
	application "DairoDFS/application"
	"DairoDFS/controller/app/login/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/UserTokenDao"
	"DairoDFS/dao/dto"
	"DairoDFS/extension/Number"
	"DairoDFS/extension/String"
	"net/http"
	"strconv"
	"time"
)

//登录页面
//@Group:/app/login

/** 页面初始化 */
//@Html:.html
func Init(writer http.ResponseWriter, request *http.Request) {
	if !UserDao.IsInit() { //是否已经初始化
		http.Redirect(writer, request, "/app/install/create_admin", http.StatusFound)
	}
}

/** 用户登录 */
//@Post:/do-login
func DoLogin(loginForm form.LoginAppInForm, _clientFlag int, _version int) any {
	userDto, _ := UserDao.SelectByName(loginForm.Name)

	//删除已经存在登录记录
	UserTokenDao.DeleteByUserIdAndDeviceId(userDto.Id, loginForm.DeviceId)

	//登录token
	token := strconv.FormatInt(time.Now().UnixMicro(), 10)
	token = String.ToMd5(token)

	//TODO:
	ip := "0.0.0.0"
	userTokenDto := dto.UserTokenDto{
		Id:         Number.ID(),
		UserId:     userDto.Id,
		Date:       time.Now(),
		Ip:         ip,
		ClientFlag: _clientFlag,
		Version:    _version,
		Token:      token,
		DeviceId:   loginForm.DeviceId,
	}

	//添加一条登录记录
	UserTokenDao.Add(userTokenDto)
	userTokenList := UserTokenDao.ListByUserId(userDto.Id)
	for len(userTokenList) > application.UserTokenLimit { //挤掉以前的登录记录

		//删除登录记录
		UserTokenDao.DeleteByToken(userTokenList[0].Token)

		//移除第一个元素
		userTokenList = userTokenList[1:]
	}
	return token
}

/**
 * 退出登录
 */
//@Post:/logout
func Logout(request *http.Request) {

	//获取APP登录票据
	cookieToken, _ := request.Cookie("token")
	if cookieToken == nil {
		return
	}
	token := cookieToken.Value
	if len(token) == 0 {
		return
	}

	//删除登录记录
	UserTokenDao.DeleteByToken(token)
}

//
//    /**
//     * 忘记密码
//     */
//    @PostMapping("/forget")
//    @ResponseBody
//    fun forget(session: HttpSession): String {
//        val msg = "账户密码保存在"
//        return dbPath
//    }
//}
