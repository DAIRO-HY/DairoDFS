package cn.dairo.dfs.controller.app.install.libraw

import cn.dairo.dfs.config.Constant
import cn.dairo.dfs.controller.app.install.libraw.form.LibrawInstallProgressForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.extension.toDataSize
import cn.dairo.dfs.util.ShellUtil
import jakarta.servlet.http.HttpServletResponse
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody
import java.io.*
import java.net.HttpURLConnection
import java.net.URL
import java.util.*
import java.util.zip.ZipInputStream
import kotlin.concurrent.thread

/**
 * 安装libraw
 */
@Controller
@RequestMapping("/app/install/libraw")
class LibrawInstallAppController : AppBase() {

    /**
     * 是否正在下载中
     */
    private var isRuning = false

    private var browerOStream: OutputStream? = null

    private val browerOStreamLock = Object()

    private var consoleByteArrayOutputStream: ByteArrayOutputStream? = null

    /**
     * 下载地址
     */
    private val url: String
        get() {
            val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
            if (osName == "linux") {
                return "https://github.com/LibRaw/LibRaw/archive/refs/tags/0.21.2.zip"
            } else if (osName.contains("windows")) {
                return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/LibRaw-0.21.2-Win64.zip"
            } else if (osName.contains("mac")) {//mac系统
                return "https://github.com/DAIRO-HY/DairoDfsLib/raw/main/LibRaw-0.21.2-macOS.zip"
            } else {
                return ""
            }
        }

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute(): String {
        if (this.consoleByteArrayOutputStream == null) {
            this.consoleByteArrayOutputStream = ByteArrayOutputStream()
        }
        return "app/install/install_libraw"
    }

    /**
     * 开始安装
     */
    @PostMapping("/install")
    @ResponseBody
    fun install() {
        if (this.isRuning) {
            return
        }
        if (File(Constant.LIBRAW_BIN).exists()) {
            this.check()
            return
        }
        this.isRuning = true
        thread {
            val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
            if (osName == "linux") {
                this::class.java.classLoader.getResourceAsStream("libraw-install.sh").use { iStream ->
                    val installShellFile = File(Constant.LIBRAW_PATH + "/libraw-install.sh")
                    if (!installShellFile.parentFile.exists()) {
                        installShellFile.parentFile.mkdirs()
                    }
                    installShellFile.outputStream().use {
                        iStream.transferTo(it)
                    }
                }

                //将sh赋予可执行权限
                ShellUtil.execToInputStream("chmod +x ${Constant.LIBRAW_PATH}/libraw-install.sh", true) {
                    it.transferTo(this.browerOStream)
                }
                this.browerOStream?.flush()
                ShellUtil.execToInputStream(Constant.LIBRAW_PATH + "/libraw-install.sh", true) {
                    while (true) {
                        val line = BufferedReader(InputStreamReader(it)).readLine()
                        this.write(line)
                        if (line == null) {
                            break
                        }
                    }
                }
                this.check()
                return@thread
            }
            val http = URL(this.url).openConnection() as HttpURLConnection
            var oStream: OutputStream? = null
            try {
                this.write("正在下载")
                http.connect()
                if (http.responseCode != 200) {
                    this.write(String(http.errorStream.readAllBytes()))
                    return@thread
                }

                //文件总大小
                val total = http.contentLengthLong
                val iStream = http.inputStream
                val file = File(Constant.LIBRAW_PATH + ".zip")
                if (!file.parentFile.exists()) {
                    file.parentFile.mkdirs()
                }
                oStream = file.outputStream()

                //记录最后一次请求下载大小(用来计算网速)
                var lastDownloadedSize = 0L

                //记录最后一次请求进度时间
                var lastProgressTime = 0L

                //已经下载大小
                var downloadedSize = 0L

                //下载缓存
                val cache = ByteArray(64 * 1024)
                var len: Int
                while (iStream.read(cache, 0, cache.size).also { len = it } != -1) {
                    oStream.write(cache, 0, len)
                    downloadedSize += len
                    val now = System.currentTimeMillis()
                    if ((now - lastProgressTime) > 1000) {//控制频率

                        //计算下载速度
                        val speed =
                            (downloadedSize - lastDownloadedSize).toDouble() / (now - lastProgressTime) * 1000

                        val progress: Double = if (total == 0L) {
                            0.0
                        } else {
                            downloadedSize.toDouble() / total
                        }
                        this.write("下载进度:${(progress * 100).toInt()}% (${speed.toDataSize}/S ${downloadedSize.toDataSize}/${total.toDataSize})")

                        //保存最后一次统计信息
                        lastProgressTime = now
                        lastDownloadedSize = downloadedSize
                    }
                }
                this.write("正在解压")

                //解压
                unzip(file, Constant.LIBRAW_PATH + ".temp")
                File(Constant.LIBRAW_PATH + ".temp").renameTo(File(Constant.LIBRAW_PATH))
                oStream.close()
                file.delete()
                this.write("下载完成")
                this.check()
            } catch (e: Exception) {
                this.write("安装出错:$e")
            } finally {
                this.isRuning = false
                http.disconnect()
                oStream?.close()
            }
        }
    }

    /**
     * 解压
     */
    private fun unzip(zipFile: File, destDir: String) {
        val dir = File(destDir)
        // 如果目标目录不存在，则创建它
        if (!dir.exists()) dir.mkdirs()
        var zis: ZipInputStream? = null
        try {// 创建ZipInputStream对象来读取ZIP文件
            zis = ZipInputStream(FileInputStream(zipFile))
            var zipEntry = zis.nextEntry

            val buffer = ByteArray(64 * 1024)
            // 遍历ZIP文件中的每一个条目
            while (zipEntry != null) {
                val filePath = destDir + File.separator + zipEntry.name
                if (!zipEntry.isDirectory) {
                    this.write("正在解压:$filePath")

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
        } catch (e: Exception) {
            throw e
        } finally {
            zis?.close()
        }
    }

    /**
     * 解压存储文件
     */
    private fun extractFile(`is`: InputStream, filePath: String, buffer: ByteArray) {
        val file = File(filePath)
        if (!file.parentFile.exists()) {
            file.parentFile.mkdirs()
        }
        val fos = FileOutputStream(filePath)
        var len: Int
        while ((`is`.read(buffer).also { len = it }) > 0) {
            fos.write(buffer, 0, len)
        }
        fos.close()
    }

    /**
     * MAC系统需要验证管理员密码
     */
    private fun validateAdminPasswordByMacOS() {
        val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
        if (!osName.contains("mac")) {
            return
        }

        // 使用 ProcessBuilder 打开终端并执行指定的命令
        val pb = ProcessBuilder(
            "osascript", "-e", "tell application \"Terminal\" to do script \"sudo chmod -R +x ${
                File(Constant.LIBRAW_BIN).absolutePath.replace(
                    "/./", "/"
                )
            }\"", "-e", "tell application \"Terminal\" to activate"
        )
        pb.start()
    }

    /**
     * 检查安装情况
     */
    fun check() {
        try {
            this.test()
            this.write("安装成功")
            synchronized(this.browerOStreamLock) {
                this.browerOStreamLock.notifyAll()
            }
        } catch (e: Exception) {
            val osName = System.getProperty("os.name").lowercase(Locale.getDefault())
            if (osName.contains("mac")) {//mac系统是
                if (e.message!!.contains("error=13")) {
                    this.write("安装失败:$e\n解决方案:请在弹出的Terminal窗口中验证密码,然后回到当前页面,再次点击安装按钮即可.")
                }
                validateAdminPasswordByMacOS()
            }
            this.write("安装失败:$e")
        }
    }

    /**
     * 测试一下是否安装成功
     */
    fun test() {
        try {

            //检查是否安装成功
            ShellUtil.exec("${Constant.LIBRAW_BIN}/dcraw_emu -verson")
        } catch (e: Exception) {
            if (e.message!!.startsWith("Unknown option \"-verson\".")) {
                return
            }
            throw e
        }
    }

    /**
     * 检查安装情况
     */
    @PostMapping("/state")
    @ResponseBody
    fun state(): LibrawInstallProgressForm {
        val form = LibrawInstallProgressForm()
        try {
            this.test()
            this.write("安装已完成")
            synchronized(this.browerOStreamLock) {
                this.browerOStreamLock.notifyAll()
            }
            form.hasFinish = true
            this.clear()
        } catch (e: Exception) {
            if (!this.isRuning) {
                this.write("等待安装")
            }
            form.hasFinish = false
        }
        return form
    }

    /**
     * 控制台
     */
    @GetMapping("/console")
    @ResponseBody
    fun console(response: HttpServletResponse) {
        synchronized(this.browerOStreamLock) {
            this.browerOStreamLock.notifyAll()
            response.reset()
            response.characterEncoding = "UTF-8"
            response.contentType = "text/event-stream"
            this.browerOStream = response.outputStream
            this.consoleByteArrayOutputStream?.writeTo(this.browerOStream)
            this.browerOStreamLock.wait()
        }
    }

    /**
     * 网页面写入控制台信息
     */
    private fun write(txt: String) {
        try {
            this.consoleByteArrayOutputStream?.write(txt.toByteArray())
            this.consoleByteArrayOutputStream?.write("\n".toByteArray())

            this.browerOStream?.write(txt.toByteArray())
            this.browerOStream?.write("\n".toByteArray())

            this.browerOStream?.flush()
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    /**
     * 安装完成之后,垃圾回收
     */
    fun clear() {
        try {
            this.consoleByteArrayOutputStream?.close()
        } catch (_: Exception) {
        }
        this.consoleByteArrayOutputStream = null

        try {
            this.browerOStream?.close()
        } catch (_: Exception) {
        }
        this.browerOStream = null
    }
}
