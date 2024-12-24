package cn.dairo.dfs.controller.app.share

import cn.dairo.dfs.code.ErrorCode
import cn.dairo.dfs.config.Constant
import cn.dairo.dfs.controller.app.share.form.ShareForm
import cn.dairo.dfs.controller.base.AjaxBase
import cn.dairo.dfs.dao.DfsFileDao
import cn.dairo.dfs.dao.ShareDao
import cn.dairo.dfs.dao.dto.DfsFileThumbDto
import cn.dairo.dfs.dao.dto.ShareDto
import cn.dairo.dfs.exception.BusinessException
import cn.dairo.dfs.extension.*
import cn.dairo.dfs.service.DfsFileService
import cn.dairo.dfs.util.DfsFileUtil
import cn.dairo.dfs.util.ServletTool
import cn.dairo.dfs.util.file_handle.DfsFileHandleUtil
import cn.dairo.lib.server.AESUtil
import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Controller
import org.springframework.ui.Model
import org.springframework.web.bind.annotation.*
import java.util.*

/**
 * 提取分享的文件
 */
@Controller
@RequestMapping("/app/share/{id}")
class ShareAppController : AjaxBase() {

    /**
     * 文件夹数据操作Service
     */
    @Autowired
    private lateinit var dfsFileService: DfsFileService

    /**
     * 文件数据操作Dao
     */
    @Autowired
    private lateinit var dfsFileDao: DfsFileDao

    /**
     * 分享操作Dao
     */
    @Autowired
    private lateinit var shareDao: ShareDao

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute(@PathVariable id: Long, model: Model): String {
        try {
            this.getShare(id)
            return "app/share"
        } catch (e: BusinessException) {
            return when (e.code) {
                ErrorCode.SHARE_NEED_PWD.code -> "app/share_pwd"
                else -> {
                    model.addAttribute("error", e.message)
                    "app/share_error"
                }
            }
        }
    }

    /**
     * 验证密码
     * @param id 分享ID
     */
    @PostMapping("/valid_pwd")
    @ResponseBody
    fun validPwd(@PathVariable id: Long, pwd: String) {
        val shareDto = this.shareDao.selectOne(id) ?: throw ErrorCode.SHARE_NOT_FOUND//分享不存在
        if (pwd == shareDto.pwd) {
            ServletTool.session.setAttribute(id.toString(), true)
        } else {
            throw BusinessException("密码不正确")
        }
    }

    /**
     * 转存
     * @param id 分享ID
     * @param folder 所选择的父级文件夹
     * @param names 所选择的文件夹或文件名数组
     * @param target 要转存的目标文件夹
     */
    @PostMapping("/save_to")
    @ResponseBody
    fun saveTo(
        request: HttpServletRequest,
        @PathVariable id: Long,
        @RequestParam("folder", defaultValue = "") folder: String,
        @RequestParam("names", defaultValue = "") names: Array<String>,
        @RequestParam("target", defaultValue = "") target: String
    ) {
        val userId = request.getAttribute(Constant.REQUEST_USER_ID) as Long?
        if (userId == null) {//没有登录不允许转存
            throw ErrorCode.NO_LOGIN
        }
        val paths = names.map { folder + "/" + it }
        val shareDto = this.validateShare(id, *paths.toTypedArray())
        val truePaths = paths.map {
            shareDto.folder + it
        }
        this.dfsFileService.shareSaveTo(
            shareUserId = shareDto.userId!!,
            userId = userId,
            sourcePaths = truePaths,
            targetFolder = target
        )

        //开启生成缩略图线程
        DfsFileHandleUtil.start()
    }

    /**
     * 重置密码
     * @param id 分享ID
     * @param folder 分享的文件夹路径
     */
    @PostMapping("/get_list")
    @ResponseBody
    fun getList(@PathVariable id: Long, folder: String): List<ShareForm> {
        val shareDto = this.validateShare(id, folder)

        //用户ID
        val userId = shareDto.userId!!

        //文件列表
        val fileList: List<DfsFileThumbDto>
        if (folder == "") {//分享的根目录

            //得到分享的父文件夹ID
            val shareFolderId =
                this.dfsFileService.getIdByFolder(userId, shareDto.folder!!) ?: throw ErrorCode.NO_FOLDER


            //分享的文件名或文件夹名列表
            val shareFileNameSet = shareDto.names!!.split("|").toHashSet()

            //需要筛选出分享的文件
            fileList =
                this.dfsFileDao.selectSubFile(userId, shareFolderId).filter { shareFileNameSet.contains(it.name) }
        } else {

            //实际文件夹目录
            val trueFolder = shareDto.folder + folder

            //得到分享的父文件夹ID
            val folderId = this.dfsFileService.getIdByFolder(userId, trueFolder) ?: throw ErrorCode.NO_FOLDER

            fileList = this.dfsFileDao.selectSubFile(userId, folderId)
        }
        return fileList.map {

            //将id加密之后再生成图片链接,防止图片链接被非法盗用,保证数据安全
            val encodeId = AESUtil.encrypt(it.id.toString(), id.toString())
            ShareForm().apply {
                this.name = it.name
                this.size = it.size
                this.date = it.date?.format()
                this.fileFlag = it.isFile
                this.thumb = if (it.hasThumb) "/app/share/$id/thumb?tag=$encodeId" else null
            }
        }
    }

    /**
     * 文件下载
     * @param request 客户端请求
     * @param response 往客户端返回内容
     * @param id 分享ID
     * @param name 文件名
     * @param folder 所在文件夹
     */
    @GetMapping("/download/{name}")
    fun download(
        request: HttpServletRequest,
        response: HttpServletResponse,
        @PathVariable("id") id: Long,
        @PathVariable("name") name: String,
        folder: String
    ) {
        val path = folder + "/" + name
        val shareDto = this.validateShare(id, path)

        //用户ID
        val userId = shareDto.userId!!

        //实际文件目录
        val truePath = shareDto.folder + path

        //得到文件ID
        val fileId = this.dfsFileDao.selectIdByPath(userId, truePath.toDfsFileNameList)
        if (fileId == null) {//文件不存在
            response.status = HttpStatus.NOT_FOUND.value()
            return
        }
        DfsFileUtil.download(fileId, request, response)
    }


    /**
     * 转存
     * @param id 分享ID
     * @param path 分享的路径数组
     */
    private fun validateShare(id: Long, vararg path: String): ShareDto {
        val shareDto = this.getShare(id)

        //得到分享的父文件夹ID
        val shareFolderId =
            this.dfsFileService.getIdByFolder(shareDto.userId!!, shareDto.folder!!) ?: throw ErrorCode.NO_FOLDER

        if (shareFolderId > 0) {//非根目录时,要验证是否存在文件夹
            val dfsFile = this.dfsFileDao.selectOne(shareFolderId) ?: throw ErrorCode.NO_FOLDER
            if (dfsFile.deleteDate != null) {//文件已经删除
                throw ErrorCode.NO_FOLDER
            }
        }

        //分享的文件名或文件夹名列表
        val shareNameList = shareDto.names!!.split("|")
        val shareNameSet = HashSet(shareNameList)
        path.forEach {
            if (it.isEmpty()) {
                return@forEach
            }
            val shareFirstName = it.toDfsFileNameList[1]
            if (!shareNameSet.contains(shareFirstName)) {
                throw BusinessException("[$it]不是分享的文件")
            }
        }
        return shareDto
    }

    /**
     * 获取分享的信息
     * @param id 分享ID
     * @return 分享信息
     */
    private fun getShare(id: Long): ShareDto {
        val shareDto = this.shareDao.selectOne(id) ?: throw ErrorCode.SHARE_NOT_FOUND//分享不存在
        if (shareDto.endDate != null) {
            val endDate = Date(shareDto.endDate!!)
            if (endDate < Date()) {//分享已过期
                throw ErrorCode.SHARE_IS_END
            }
        }
        if (shareDto.pwd != null) {//如果需要提取码
            ServletTool.session.getAttribute(id.toString()) ?: throw ErrorCode.SHARE_NEED_PWD
        }
        return shareDto
    }

    /**
     * 缩略图
     * @param request 客户端请求
     * @param response 往客户端返回内容
     * @param id 文件ID
     */
    @GetMapping("/thumb")
    fun thumb(
        request: HttpServletRequest,
        response: HttpServletResponse,
        @PathVariable id: Long,
        @RequestParam("tag") tag: String
    ) {
        this.shareDao.selectOne(id) ?: throw ErrorCode.SHARE_NOT_FOUND//分享不存在
        val dfsId = AESUtil.decrypt(tag.replace(" ", "+"), id.toString())
        val dfsDto = this.dfsFileDao.selectOne(dfsId!!.toLong())
        if (dfsDto == null) {//文件不存在
            response.status = HttpStatus.NOT_FOUND.value()
            return
        }

        //获取缩率图附属文件
        val thumb = this.dfsFileDao.selectExtra(dfsDto.id!!, "thumb")
        DfsFileUtil.download(thumb, request, response)
    }
}
