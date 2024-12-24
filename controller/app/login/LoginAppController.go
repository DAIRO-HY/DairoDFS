package login

/**
 * 管理员登录画面
 */
//@Controller
//@RequestMapping("/app/login")
//class LoginAppController : AppBase() {
//
//    /**
//     * 文件路径
//     */
//    @Value("\${sqlite.path}")
//    private lateinit var dbPath: String
//
//    /**
//     * 用一个用户允许登录的客户端数量限制
//     */
//    @Value("\${user.token.limit}")
//    private var userTokenLimit = 0
//
//    /**
//     * 用户操作Dao
//     */
//    @Autowired
//    private lateinit var userDao: UserDao
//
//    /**
//     * 用户登录票据
//     */
//    @Autowired
//    private lateinit var userTokenDao: UserTokenDao

/**
 * 页面初始化
 */
//get:/app/login
//templates:app/login.html
func Execute() {
	//if !this.userDao.isInit() { //是否已经初始化
	//	return "redirect:/app/install/ffmpeg"
	//}
	//return "app/login"
}

//    @Operation(summary = "用户登录")
//    @PostMapping("/do-login")
//    @ResponseBody
//    fun doLogin(
//        @Valid loginForm: LoginAppForm,
//        @Parameter(name = "客户端标志") @RequestParam("_clientFlag") clientFlag: Int,
//        @Parameter(name = "客户端版本") @RequestParam("_version") version: Int
//    ): String {
//        val userDto = this.userDao.selectByName(loginForm.name!!) ?: throw BusinessException("用户名或密码错误")
//        if (loginForm.pwd != userDto.pwd) {
//            throw BusinessException("用户名或密码错误")
//        }
//
//        //删除已经存在登录记录
//        this.userTokenDao.deleteByUserIdAndDeviceId(userDto.id!!, loginForm.deviceId!!)
//
//        val token = System.currentTimeMillis().toString().md5
//        val userTokenDto = UserTokenDto()
//        userTokenDto.id = DBID.id
//        userTokenDto.userId = userDto.id
//        userTokenDto.date = Date()
//        userTokenDto.ip = ServletTool.getClientIp()
//        userTokenDto.clientFlag = clientFlag
//        userTokenDto.version = version
//        userTokenDto.token = token
//        userTokenDto.deviceId = loginForm.deviceId
//
//        //添加一条登录记录
//        this.userTokenDao.add(userTokenDto)
//
//        val userTokenList = this.userTokenDao.listByUserId(userDto.id!!) as ArrayList
//        while (userTokenList.size > userTokenLimit) {//挤掉以前的登录记录
//
//            //删除登录记录
//            this.userTokenDao.deleteByToken(userTokenList[0].token!!)
//            userTokenList.removeAt(0)
//        }
//        return token
//    }
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
