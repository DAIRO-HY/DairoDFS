package cn.dairo.dfs.controller.app.install.ffmpeg

import cn.dairo.dfs.config.Constant
import cn.dairo.dfs.controller.app.install.ffmpeg.form.FfmpegInstallProgressForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.extension.fileParent
import cn.dairo.dfs.extension.toDataSize
import cn.dairo.dfs.util.ShellUtil
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL
import java.util.*
import java.util.zip.ZipInputStream
import kotlin.concurrent.thread


/**
 * 安装ffmpeg
 */
@Controller
@RequestMapping("/app/install/ffmpeg")
class FfmpegInstallAppController : AppBase() {

    /**
     * 下载信息
     */
    private var info: String? = null

    /**
     * 文件总大小
     */
    private var total = 0L

    /**
     * 已经下载大小
     */
    private var downloadedSize = 0L

    /**
     * 是否正在下载中
     */
    private var isRuning = false

    /**
     * 记录最后一次请求下载大小(用来计算网速)
     */
    private var lastDownloadedSize = 0L

    /**
     * 记录最后一次请求进度时间
     */
    private var lastProgressTime = 0L

    /**
     * 下载地址
     */
    private val url: String
        get() {
            val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
            if (osName == "linux") {
                return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-linux-amd64.zip"
            } else if (osName.contains("windows")) {
                return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-win.zip"
            } else if (osName.contains("mac")) {
                return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/ffmpeg-7.0.2-macOS.zip"
            } else {
                return ""
            }
        }

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute() = "app/install/install_ffmpeg"

    /**
     * 开始安装
     */
    @PostMapping("/install")
    @ResponseBody
    fun install() {
        if (File(Constant.FFMPEG_PATH).exists()) {
            return
        }
        if (this.isRuning) {
            return
        }
        this.isRuning = true
        thread {
            this.total = 0L
            this.downloadedSize = 0L
            val http =
                URL(this.url).openConnection() as HttpURLConnection
            try {
                this.info = "正在下载"
                http.connect()
                if (http.responseCode != 200) {
                    this.info = String(http.errorStream.readAllBytes())
                    return@thread
                }
                this.total = http.contentLengthLong
                val iStream = http.inputStream
                val file = File(Constant.FFMPEG_PATH + ".zip")
                if (!file.parentFile.exists()) {
                    file.parentFile.mkdirs()
                }
                val oStream = file.outputStream()

                //下载缓存
                val cache = ByteArray(8 * 1024)
                var len: Int
                while (iStream.read(cache, 0, cache.size).also { len = it } != -1) {
                    oStream.write(cache, 0, len)
                    this.downloadedSize += len
                }
                this.info = "正在解压"

                //解压
                unzip(file, Constant.FFMPEG_PATH + ".temp")
                File(Constant.FFMPEG_PATH + ".temp").renameTo(File(Constant.FFMPEG_PATH))
                file.delete()
                this.info = "安装完成"
            } catch (e: Exception) {
                this.info = "安装失败：$e"
            } finally {
                this.isRuning = false
                http.disconnect()
            }
        }
    }

    /**
     * 解压
     */
    fun unzip(zipFile: File, destDir: String) {
        val dir = File(destDir)
        // 如果目标目录不存在，则创建它
        if (!dir.exists()) dir.mkdirs()

        val buffer = ByteArray(8 * 1024)
        // 创建ZipInputStream对象来读取ZIP文件
        val zis = ZipInputStream(FileInputStream(zipFile))
        var zipEntry = zis.nextEntry
        // 遍历ZIP文件中的每一个条目
        while (zipEntry != null) {
            val filePath = destDir + File.separator + zipEntry.name
            if (!zipEntry.isDirectory) {
                // 如果是文件，提取它
                extractFile(zis, filePath, buffer)
            } else {
                // 如果是目录，创建目录
                val dirFile = File(filePath)
                dirFile.mkdirs()
            }
            zis.closeEntry()
            zipEntry = zis.nextEntry
        }
        zis.close()
    }

    /**
     * 解压存储文件
     */
    private fun extractFile(`is`: InputStream, filePath: String, buffer: ByteArray) {
        val fos = FileOutputStream(filePath)
        var len: Int
        while ((`is`.read(buffer).also { len = it }) > 0) {
            fos.write(buffer, 0, len)
        }
        fos.close()
    }

    /**
     * 获取下载进度
     */
    @PostMapping("/progress")
    @ResponseBody
    fun progress(): FfmpegInstallProgressForm {
        val form = FfmpegInstallProgressForm()
        form.info = this.info
        if (File(Constant.FFMPEG_PATH).exists()) {//已经安装完成
            try {
                val version = ShellUtil.exec("${Constant.FFMPEG_PATH}/ffmpeg -version")
                form.info = "安装完成:$version"
                form.hasFinish = true
            } catch (e: Exception) {
                form.hasFinish = false
                val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
                if (osName == "linux") {
                    if (e.message!!.contains("error=13")) {

                        //开启可执行权限
                        ShellUtil.exec(
                            """chmod -R +x "${
                                File(Constant.FFMPEG_PATH).absolutePath.replace(
                                    "/./",
                                    "/"
                                )
                            }""""
                        )
                        val version = ShellUtil.exec("${Constant.FFMPEG_PATH}/ffmpeg -version")
                        form.info = "安装完成:$version"
                        form.hasFinish = true
                        return form
                    }
                } else if (osName.contains("mac")) {//mac系统是
                    if (e.message!!.contains("error=13")) {
                        form.error =
                            "安装失败:$e\n解决方案:请在弹出的Terminal窗口中验证密码,然后回到当前页面,再次点击安装按钮即可."

                        // 使用 ProcessBuilder 打开终端并执行指定的命令
                        val pb = ProcessBuilder(
                            "osascript",
                            "-e",
                            "tell application \"Terminal\" to do script \"sudo chmod -R +x ${
                                File(Constant.FFMPEG_PATH).absolutePath.replace(
                                    "/./",
                                    "/"
                                )
                            }\"",
                            "-e", "tell application \"Terminal\" to activate"
                        )
                        pb.start()
                    }
                    return form
                }
                form.error = "安装失败:$e"
            }
            return form
        }
        if (!this.isRuning) {//还没有开始安装
            form.hasRuning = false
            return form
        }
        form.hasRuning = true

        val downloadedSize = this.downloadedSize
        val now = System.currentTimeMillis()

        //计算下载速度
        val speed = (downloadedSize - this.lastDownloadedSize).toDouble() / (now - this.lastProgressTime) * 1000

        this.lastProgressTime = now
        this.lastDownloadedSize = downloadedSize

//        form.url = FFMPEGInstallAppController.DOWNLOAD_URL
        form.total = this.total.toDataSize
        form.downloadedSize = downloadedSize.toDataSize
        form.speed = speed.toDataSize + "/S"
        val progress: Double = if (this.total == 0L) {
            0.0
        } else {
            downloadedSize.toDouble() / total
        }
        form.progress = (progress * 100).toInt()
        return form
    }
}
