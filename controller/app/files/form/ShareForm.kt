package form
type ShareForm struct{

    @Parameter(description = "分享结束时间戳,0代表永久有效")
    @NotNull
    endDateTime int64

    @Parameter(description = "分享密码")
    @Size(max = 32)
    pwd string

    @Parameter(description = "分享的文件夹")
    folder string

    @Parameter(description = "要分享的文件名或文件夹名列表")
    @NotNull
    names: List<String>? = null

    /** 验证截止日期是否正确输入 **/
    @AssertTrue(message = "结束日期必须在现在的时间之后")
    fun isEndDateTime() bool
        this.endDateTime ?: return true
        if (this.endDateTime == 0L) return true
        if (this.endDateTime!! < System.currentTimeMillis()) {
            return false
        }
        return true
    }
}