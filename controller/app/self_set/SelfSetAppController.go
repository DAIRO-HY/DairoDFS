package self_set

import (
	application "DairoDFS/appication"
	"DairoDFS/controller/app/self_set/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/extension/Date"
	"DairoDFS/extension/String"
	"DairoDFS/util/LoginState"
	"math/rand"
	"strconv"
	"time"
)

/**
 * 系统设置
 */

/**
 * 页面初始化
 */
//@Get:/app/self_set
//@templates:app/self_set.html
func Html() {}

/**
 * 页面初始化
 */
//@Post:/app/self_set/init
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
//@Post:/app/self_set/make_api_token
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

/**
 * 生成web访问路径前缀
 */
//@Post:/app/self_set/make_url_path
func MakeUrlPath(flag int) {
	loginId := LoginState.LoginId()
	if flag == 0 {
		UserDao.SetUrlPath(loginId, nil)
		return
	}
	timespan := time.Now().UnixMilli() - application.BASE_TIME
	urlPath := String.ToShortString(timespan)
	UserDao.SetUrlPath(loginId, &urlPath)
}

/**
 * 生成端对端加密
 */
//@Post:/app/self_set/make_encryption
func MakeEncryption(flag int) {
	loginId := LoginState.LoginId()
	if flag == 0 {
		UserDao.SetEncryptionKey(loginId, nil)
		return
	}

	encryptionDataArray := make([]byte, 128)
	for i := 0; i < 128; i++ {
		encryptionDataArray[i] = byte(i)
	}
	shuffle(encryptionDataArray)
	encryptionKey := String.ToBase64(encryptionDataArray)
	UserDao.SetEncryptionKey(loginId, &encryptionKey)
}

// shuffle 打乱数组顺序
func shuffle(slice []byte) {
	rand.Seed(time.Now().UnixNano()) // 使用当前时间作为随机数种子
	for i := len(slice) - 1; i > 0; i-- {
		j := rand.Intn(i + 1) // 生成 [0, i] 范围内的随机数
		slice[i], slice[j] = slice[j], slice[i]
	}
}

//}
