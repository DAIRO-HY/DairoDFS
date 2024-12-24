package cn.dairo.dfs.controller.app.files.form

import io.swagger.v3.oas.annotations.Parameter
import jakarta.validation.constraints.AssertTrue
import jakarta.validation.constraints.NotNull
import jakarta.validation.constraints.Size

class ShareForm {

    @Parameter(description = "分享结束时间戳,0代表永久有效")
    @NotNull
    var endDateTime: Long? = null

    @Parameter(description = "分享密码")
    @Size(max = 32)
    var pwd: String? = null

    @Parameter(description = "分享的文件夹")
    var folder: String = ""

    @Parameter(description = "要分享的文件名或文件夹名列表")
    @NotNull
    var names: List<String>? = null

    /**
     * 验证截止日期是否正确输入
     */
    @AssertTrue(message = "结束日期必须在现在的时间之后")
    fun isEndDateTime(): Boolean {
        this.endDateTime ?: return true
        if (this.endDateTime == 0L) return true
        if (this.endDateTime!! < System.currentTimeMillis()) {
            return false
        }
        return true
    }
}