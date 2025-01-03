package self_set

import (
	"DairoDFS/controller/app/self_set/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/extension/Date"
	"DairoDFS/extension/String"
	"DairoDFS/util/LoginState"
	"strconv"
	"time"
)

/**
 * 系统设置
 */

/**
 * 页面初始化
 */
//@get:/app/self_set
//@templates:app/self_set.html
func Html() {}

/**
 * 页面初始化
 */
//@post:/app/self_set/init
func Init() form.SelfSetForm {
	loginId := LoginState.LoginId()
	userDto := UserDao.SelectOne(loginId)
	date := Date.Format(*userDto.Date)
	return form.SelfSetForm{
		Id:            userDto.Id,
		Name:          userDto.Name,
		Email:         userDto.Email,
		Date:          &date,
		UrlPath:       userDto.UrlPath,
		ApiToken:      userDto.ApiToken,
		EncryptionKey: userDto.EncryptionKey,
	}
}

/**
 * 生成API票据
 */
//@post:/app/self_set/make_api_token
func MakeApiToken(flag int) {
	loginId := LoginState.LoginId()
	if flag == 0 {
		UserDao.SetApiToken(loginId, nil)
		return
	}
	timespan := strconv.FormatInt(time.Now().UnixMicro(), 10)
	apiToken := String.ToMd5(timespan)
	UserDao.SetApiToken(loginId, &apiToken)
}

//
//    /**
//     * 生成web访问路径前缀
//     */
//    @PostMapping("/make_url_path")
//    @ResponseBody
//    fun makeUrlPath(flag: Int) {
//        val id = super.loginId
//        if (flag == 0) {
//            this.userDao.setUrlPath(id, null)
//            return
//        }
//        val timespan = System.currentTimeMillis() - Constant.BASE_TIME
//        val urlPath = timespan.toShortString
//        this.userDao.setUrlPath(id, urlPath)
//    }
//
//    /**
//     * 生成端对端加密
//     */
//    @PostMapping("/make_encryption")
//    @ResponseBody
//    fun makeEncryption(flag: Int) {
//        val id = super.loginId
//        if (flag == 0) {
//            this.userDao.setEncryptionKey(id, null)
//            return
//        }
//        val encryptionDataArray = ByteArray(128) { it.toByte() }
//        encryptionDataArray.shuffle()
//        val encryptionKey = encryptionDataArray.base64
//        this.userDao.setEncryptionKey(id, encryptionKey)
//    }
//}
