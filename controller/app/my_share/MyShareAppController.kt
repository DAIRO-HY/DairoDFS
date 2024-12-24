package cn.dairo.dfs.controller.app.my_share

import cn.dairo.dfs.code.ErrorCode
import cn.dairo.dfs.controller.app.my_share.form.MyShareDetailForm
import cn.dairo.dfs.controller.app.my_share.form.MyShareForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.dao.ShareDao
import cn.dairo.dfs.extension.*
import io.swagger.v3.oas.annotations.Operation
import io.swagger.v3.oas.annotations.Parameter
import jakarta.servlet.http.HttpServletRequest
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.*
import java.util.*

/**
 * 提取分享的文件
 */
@Controller
@RequestMapping("/app/my_share")
class MyShareAppController : AppBase() {

    /**
     * 分享操作Dao
     */
    @Autowired
    private lateinit var shareDao: ShareDao

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute() = "app/my_share"

    @Operation(summary = "获取所有的分享")
    @PostMapping("/get_list")
    @ResponseBody
    fun getList(): List<MyShareForm> {
        val userId = super.loginId
        val now = System.currentTimeMillis()
        val list = this.shareDao.selectByUser(userId).map {
            val endDate: String
            if (it.endDate == null) {
                endDate = "永久有效"
            } else {
                val endTime = it.endDate!! - now
                if (endTime < 0) {
                    endDate = "已过期"
                } else {
                    if (endTime > 24 * 60 * 60 * 1000) {//超过1天
                        endDate = "${endTime / (24 * 60 * 60 * 1000)}天后过期"
                    } else if (endTime > 60 * 60 * 1000) {//超过1小时
                        endDate = "${endTime / (60 * 60 * 1000)}小时后过期"
                    } else if (endTime > 60 * 1000) {//超过1分钟
                        endDate = "${endTime / (60 * 1000)}分钟后过期"
                    } else {
                        endDate = "即将过期"
                    }
                }
            }
            val form = MyShareForm()
            form.id = it.id
            form.title = it.title
            form.fileCount = it.fileCount
            form.folderFlag = it.folderFlag
            form.endDate = endDate
            form.thumb = if (it.thumb != null) "/app/files/thumb/${it.thumb}" else null
            form.date = it.date!!.format()
            form
        }
        return list
    }

    @Operation(summary = "获取分享详细")
    @PostMapping("/get_detail")
    @ResponseBody
    fun getDetail(
        request: HttpServletRequest,
        @Parameter(description = "分享id") @RequestParam("id", required = true) id: Long
    ): MyShareDetailForm {
        val userId = super.loginId
        val shareDto = this.shareDao.selectOne(id)!!
        if (shareDto.userId != userId) {
            throw ErrorCode.NOT_ALLOW
        }
        var folder = shareDto.folder!!
        if (folder.isEmpty()) {
            folder = "/"
        }
        val endDate = if (shareDto.endDate == null) {
            "永久有效"
        } else {
            Date(shareDto.endDate!!).format()
        }

        //分享链接
        val url = "/app/share/${shareDto.id}"

        val form = MyShareDetailForm()
        form.id = shareDto.id
        form.url = url
        form.names = shareDto.names
        form.date = shareDto.date!!.format()
        form.endDate = endDate
        form.pwd = shareDto.pwd ?: "无"
        form.folder = folder
        return form
    }

    @Operation(summary = "取消所选分享")
    @PostMapping("/delete")
    @ResponseBody
    fun delete(@Parameter(description = "分享id列表") @RequestParam("ids", required = true) ids: List<Long>) {
        this.shareDao.delete(super.loginId, ids.joinToString(separator = ","))
    }
}
