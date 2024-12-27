package self_set

import (
	"DairoDFS/controller/app/self_set/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/extension/Date"
)

/**
 * 系统设置
 */

/**
 * 页面初始化
 */
//@get:/app/self_set
//@templates:app/self_set.html
func Init() {}

/**
 * 页面初始化
 */
//@post:/app/self_set123
func InitData() form.SelfSetForm {
	//userDto := UserDao.SelectOne(super.loginId)
	userDto := UserDao.SelectOne(0)
	return form.SelfSetForm{
		Id:            *userDto.Id,
		Name:          *userDto.Name,
		Email:         *userDto.Email,
		Date:          Date.Format(*userDto.Date),
		UrlPath:       *userDto.UrlPath,
		ApiToken:      *userDto.ApiToken,
		EncryptionKey: *userDto.EncryptionKey,
	}
}

//    /**
//     * 生成API票据
//     */
//    @PostMapping("/make_api_token")
//    @ResponseBody
//    fun makeApiToken(flag: Int) {
//        val id = super.loginId
//        if (flag == 0) {
//            this.userDao.setApiToken(id, null)
//            return
//        }
//        val timespan = System.currentTimeMillis() - Constant.BASE_TIME
//        val apiToken = StringUtil.getRandomChar(5) + timespan.toShortString
//        this.userDao.setApiToken(id, apiToken)
//    }
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
