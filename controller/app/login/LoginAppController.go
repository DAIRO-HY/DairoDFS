package login

import (
	application "DairoDFS/appication"
	"DairoDFS/controller/app/login/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/UserTokenDao"
	"DairoDFS/dao/dto"
	"DairoDFS/exception"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBUtil"
	"net/http"
	"strconv"
	"time"
)

/** 页面初始化 */
//@get:/app/login
//@templates:app/login.html
func Init(writer http.ResponseWriter, request *http.Request) {
	if !*UserDao.IsInit() { //是否已经初始化
		http.Redirect(writer, request, "/app/install/create_admin", http.StatusFound)
	}
}

/** 用户登录 */
//@post:/app/login/do-login
func DoLogin(loginForm form.LoginAppInForm, _clientFlag int, _version int) any {
	userDto := UserDao.SelectByName(loginForm.Name)
	if userDto == nil { //用户不存在
		return exception.LOGIN_ERROR()
	}
	if loginForm.Pwd != *userDto.Pwd { //密码不正确
		return exception.LOGIN_ERROR()
	}

	//删除已经存在登录记录
	UserTokenDao.DeleteByUserIdAndDeviceId(*userDto.Id, loginForm.DeviceId)

	//登录token
	token := strconv.FormatInt(time.Now().UnixMicro(), 10)
	token = String.ToMd5(token)

	id := DBUtil.ID()
	date := time.Now()

	//TODO:
	ip := "0.0.0.0"
	userTokenDto := dto.UserTokenDto{
		Id:         &id,
		UserId:     userDto.Id,
		Date:       &date,
		Ip:         &ip,
		ClientFlag: &_clientFlag,
		Version:    &_version,
		Token:      &token,
		DeviceId:   &loginForm.DeviceId,
	}

	//添加一条登录记录
	UserTokenDao.Add(userTokenDto)
	userTokenList := UserTokenDao.ListByUserId(*userDto.Id)
	for len(userTokenList) > application.UserTokenLimit { //挤掉以前的登录记录

		//删除登录记录
		UserTokenDao.DeleteByToken(*userTokenList[0].Token)

		//移除第一个元素
		userTokenList = userTokenList[1:]
	}
	return token
}

//
//    /**
//     * 退出登录
//     */
//    @PostMapping("/logout")
//    @ResponseBody
//    fun logout(session: HttpSession) {
//        session.removeAttribute("LOGIN_DATE")
//    }
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
