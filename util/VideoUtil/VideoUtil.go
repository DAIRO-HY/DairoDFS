package VideoUtil

    /**
     * 生成视频缩略图
     * @param path 视频文件路径
     * @param maxWidth 图片最大宽度
     * @param maxHeight 图片最大高度
     * @return 图片字节数组
     */
    fun thumb(path: String, maxWidth: Int, maxHeight: Int): ByteArray {

        //获取视频第一帧作为缩略图
        val jpgData =
            ShellUtil.execToByteArray(""""${Constant.FFMPEG_PATH}/ffmpeg" -i "$path" -vf select=eq(n\,0) -q:v 1 -f image2pipe -vcodec mjpeg -""")
        return ImageUtil.thumb(jpgData.inputStream(), maxWidth, maxHeight)
    }

    /**
     * 获取视频信息
     * @param path 视频文件路径
     * @return 图片字节数组
     */
    fun getInfo(path: String): VedioInfo {

        //获取视频第一帧作为缩略图
        val videoInfoData = ShellUtil.execToByteArray(""""${Constant.FFPROBE_PATH}/ffprobe" -i "$path"""", true)
        val videoInfoStr = String(videoInfoData)

        //时长
        var duration: Long? = null
        try {
            var durationStr = Regex("Duration: \\d{2}:\\d{2}:\\d{2}\\.\\d{2}").find(videoInfoStr)!!.value
            durationStr = durationStr.substring(10)
            val durationArr = durationStr.split(":")
            duration =
                ((durationArr[0].toInt() * 60 * 60 + durationArr[1].toInt() * 60 + durationArr[2].toFloat()) * 1000).toLong()
        } catch (e: Exception) {
            //e.printStackTrace()
        }

        //创建时间
        var date: Long? = null
        try {
            var dateStr = Regex("creation_time   \\: .*\\.").find(videoInfoStr)!!.value
            dateStr = dateStr.substring(dateStr.indexOf(":") + 1, dateStr.length - 1)
            dateStr = dateStr.trim()
            dateStr = dateStr.replace("T", " ")
            date = SimpleDateFormat("yyyy-MM-dd HH:mm:ss").parse(dateStr).time
        } catch (e: Exception) {
            //e.printStackTrace()
        }

        //视频比特率
        var bitrate: Int? = null
        try {
            var bitrateStr = Regex("\\d+ kb/s,.*fps").find(videoInfoStr)!!.value
            bitrateStr = bitrateStr.substring(0, bitrateStr.indexOf("kb") - 1)
            bitrate = bitrateStr.toInt()
        } catch (e: Exception) {
            //e.printStackTrace()
        }

        //帧率
        var fps = 0F
        try {
            var fpsStr = Regex("\\d+ kb/s,.*fps").find(videoInfoStr)!!.value
            fpsStr = fpsStr.substring(fpsStr.indexOf(",") + 1, fpsStr.length - 3)
            fps = fpsStr.toFloat()
        } catch (e: Exception) {
            //e.printStackTrace()
        }

        //视频宽高
        var width = 0
        var height = 0
        try {
            var whStr = Regex("Stream.+Video:.+, \\d+x\\d+").find(videoInfoStr)!!.value
            whStr = Regex(", \\d+x\\d+").find(whStr)!!.value
            whStr = Regex("\\d+x\\d+").find(whStr)!!.value
            width = whStr.split("x")[0].toInt()
            height = whStr.split("x")[1].toInt()
        } catch (e: Exception) {
            //e.printStackTrace()
        }

        //音频格式
        var audioFormat: String? = null
        try {
            val audioFormatStr = Regex("Audio: [A-z,0-9]+").find(videoInfoStr)!!.value
            audioFormat = audioFormatStr.substring(7)
        } catch (e: Exception) {
            //e.printStackTrace()
        }

        //音频采样率
        var audioSampleRate: Int? = null
        try {
            var audioSamplerateStr = Regex("Audio: .* Hz").find(videoInfoStr)!!.value
            audioSamplerateStr = Regex("\\d+ Hz").find(audioSamplerateStr)!!.value
            audioSampleRate = audioSamplerateStr.substring(0, audioSamplerateStr.length - 3).toInt()
        } catch (e: Exception) {
            //e.printStackTrace()
        }

        //音频比特率
        var audioBitrate: Int? = null
        try {
            var audioBitrateStr = Regex("Audio: .*\\d+ kb/s").find(videoInfoStr)!!.value
            audioBitrateStr = Regex("\\d+ kb/s").find(audioBitrateStr)!!.value
            audioBitrate = audioBitrateStr.substring(0, audioBitrateStr.length - 5).toInt()
        } catch (e: Exception) {
            //e.printStackTrace()
        }
        val info = VedioInfo()

        //宽
        info.width = width

        //高
        info.height = height

        //帧数
        info.fps = fps

        //视频比特率
        info.bitrate = bitrate

        //视频时长（毫秒）
        info.duration = duration

        //视频创建时间戳
        info.date = date

        //音频比特率
        info.audioBitrate = audioBitrate

        //音频采样率（HZ）
        info.audioSampleRate = audioSampleRate

        //音频格式
        info.audioFormat = audioFormat
        return info
    }

    /**
     * 视频转码
     */
    fun transfer(path: String, targetW: Int, targetH: Int, targetFps: Float, targetPath: String) {
        ShellUtil.exec(""""${Constant.FFMPEG_PATH}/ffmpeg" -i "$path" -vf scale=$targetW:$targetH -r $targetFps -f mp4 "$targetPath"""")
    }

    /**
     * 视频转码
     */
//    fun transfer(path: String, targetW: Int, targetH: Int, targetFps: Float, targetPath: String) {
//        File(targetPath).outputStream().use { oStream ->
//            ShellUtil.execToInputStream("""ffmpeg -i $path -vf scale=$targetW:$targetH -r $targetFps -f mp4 -""") {
//                it.transferTo(oStream)
//            }
//        }
//    }

    @JvmStatic
    fun main(args: Array<String>) {
        transfer("C:/Users/user/Desktop/test/bi.mp4", 640, 360, 15F, "C:/Users/user/Desktop/test/bi.after.mp4")
    }
}