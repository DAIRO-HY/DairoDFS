package cn.dairo.file.sdk

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.Typeface
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.widget.Toast
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import java.net.HttpURLConnection
import java.net.SocketTimeoutException
import java.net.URL
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * SDK调试页面:查看运行状态,自检、App请求ping、强制重启本机HTTP服务
 * 用法:FileSdkDebugActivity.start(context, sdk)
 */
class FileSdkDebugActivity : Activity() {

    companion object {
        /** 页面使用的SDK实例,start时设置 */
        @Volatile
        private var sdk: FileSdk? = null

        /** 刷新间隔(毫秒) */
        private const val REFRESH_INTERVAL = 1000L

        /** App请求ping的超时时间(毫秒) */
        private const val PING_TIMEOUT = 3000

        fun start(context: Context, sdk: FileSdk) {
            this.sdk = sdk
            val intent = Intent(context, FileSdkDebugActivity::class.java)
            if (context !is Activity) {
                intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            context.startActivity(intent)
        }
    }

    private val scope = MainScope()
    private val handler = Handler(Looper.getMainLooper())
    private val timeFormat = SimpleDateFormat("MM-dd HH:mm:ss", Locale.getDefault())

    private lateinit var sdk: FileSdk
    private lateinit var selfCheckText: TextView
    private lateinit var pingText: TextView
    private lateinit var restartText: TextView
    private lateinit var infoText: TextView
    private lateinit var selfCheckButton: Button
    private lateinit var pingButton: Button
    private lateinit var groupsButton: Button

    /** 缓存统计要查数据库,只在打开页面和点击刷新时查询 */
    private var groups: List<GroupStats> = emptyList()
    private var groupsTime = ""

    private val refreshTask = object : Runnable {
        override fun run() {
            refresh()
            handler.postDelayed(this, REFRESH_INTERVAL)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        title = "SDK调试"
        val sdk = Companion.sdk
        if (sdk == null) {
            //进程被系统回收后重新打开时没有SDK实例
            Toast.makeText(this, "没有设置SDK实例", Toast.LENGTH_SHORT).show()
            finish()
            return
        }
        this.sdk = sdk
        val padding = dp(16)
        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(padding, padding, padding, padding)
        }
        selfCheckButton = button(layout, "自检(SDK内部请求ping)") { selfCheck() }
        selfCheckText = result(layout)
        pingButton = button(layout, "App请求ping") { appPing() }
        pingText = result(layout)
        button(layout, "强制重启本机服务") { restart() }.setTextColor(Color.RED)
        restartText = result(layout)
        groupsButton = button(layout, "刷新缓存统计") { refreshGroups() }
        button(layout, "复制全部信息") { copy() }
        infoText = TextView(this).apply {
            textSize = 12f
            typeface = Typeface.MONOSPACE
            setPadding(0, dp(12), 0, 0)
            setTextIsSelectable(true)
        }
        layout.addView(infoText)
        setContentView(ScrollView(this).apply { addView(layout) })
        refreshGroups()
    }

    override fun onResume() {
        super.onResume()
        if (::sdk.isInitialized) {
            handler.post(refreshTask)
        }
    }

    override fun onPause() {
        super.onPause()
        handler.removeCallbacks(refreshTask)
    }

    override fun onDestroy() {
        super.onDestroy()
        scope.cancel()
    }

    private fun dp(value: Int) = (value * resources.displayMetrics.density).toInt()

    private fun button(layout: LinearLayout, text: String, onClick: () -> Unit): Button {
        val button = Button(this).apply {
            this.text = text
            isAllCaps = false
            setOnClickListener { onClick() }
        }
        layout.addView(button, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT))
        return button
    }

    private fun result(layout: LinearLayout): TextView {
        val text = TextView(this).apply {
            textSize = 12f
            setTextColor(Color.GRAY)
            setTextIsSelectable(true)
        }
        layout.addView(text)
        return text
    }

    private fun refresh() {
        infoText.text = info()
    }

    /** 查询缓存统计,缓存多的时候查询比较慢,在后台线程查询 */
    private fun refreshGroups() {
        groupsButton.isEnabled = false
        groupsButton.text = "查询中..."
        Thread {
            val result = try {
                sdk.stats(null)
            } catch (e: Exception) {
                emptyList()
            }
            runOnUiThread {
                groups = result
                groupsTime = now()
                groupsButton.isEnabled = true
                groupsButton.text = "刷新缓存统计"
                refresh()
            }
        }.start()
    }

    /** SDK自己向本机服务发送ping请求 */
    private fun selfCheck() {
        selfCheckButton.isEnabled = false
        selfCheckText.text = "自检中..."
        scope.launch {
            val result = sdk.selfCheck()
            val text = StringBuilder(if (result.ok) "正常" else "失败")
            text.append(",端口:").append(result.port?.toString() ?: "无")
            text.append(",TCP:").append(if (result.connected) "已连接" else "未连接")
            text.append(",状态码:").append(result.status?.toString() ?: "无")
            text.append(",耗时:").append(result.elapsedMs).append("ms")
            result.error?.let { text.append("\n").append(it) }
            selfCheckText.text = "${now()} $text"
            selfCheckButton.isEnabled = true
            refresh()
        }
    }

    /** App用自己的HttpURLConnection请求本机服务的ping,和App加载图片的方式一样 */
    private fun appPing() {
        pingButton.isEnabled = false
        pingText.text = "请求中..."
        Thread {
            val start = System.currentTimeMillis()
            var port = "-"
            val text = try {
                //localBase()在服务没有运行时会启动服务
                val url = URL(sdk.localBase() + "/ping")
                port = url.port.toString()
                val conn = url.openConnection() as HttpURLConnection
                conn.connectTimeout = PING_TIMEOUT
                conn.readTimeout = PING_TIMEOUT
                conn.useCaches = false
                try {
                    val status = conn.responseCode
                    val body = (if (status < 400) conn.inputStream else conn.errorStream)?.bufferedReader()?.use { it.readText() } ?: ""
                    "${if (status == 200) "正常" else "失败"},端口:$port,状态码:$status\n$body"
                } finally {
                    conn.disconnect()
                }
            } catch (e: SocketTimeoutException) {
                "失败,端口:$port,超时"
            } catch (e: Exception) {
                "失败,端口:$port,$e"
            }
            val elapsed = System.currentTimeMillis() - start
            runOnUiThread {
                pingText.text = "${now()} $text,耗时:${elapsed}ms"
                pingButton.isEnabled = true
                refresh()
            }
        }.start()
    }

    /** 重启最多等待1秒,在后台线程执行 */
    private fun restart() {
        restartText.text = "重启中..."
        Thread {
            val old = sdk.debugInfo().serverPort?.toString() ?: "未运行"
            val text = try {
                "重启完成,端口:${sdk.restartServer()}(原端口:$old)"
            } catch (e: Exception) {
                "重启失败:$e"
            }
            runOnUiThread {
                restartText.text = "${now()} $text"
                refresh()
            }
        }.start()
    }

    private fun copy() {
        val text = "自检:${selfCheckText.text}\nApp ping:${pingText.text}\n重启:${restartText.text}\n\n${info()}"
        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        clipboard.setPrimaryClip(ClipData.newPlainText("SDK调试信息", text))
        Toast.makeText(this, "已复制", Toast.LENGTH_SHORT).show()
    }

    /** 运行状态 */
    private fun info(): String {
        val info = sdk.debugInfo()
        val text = StringBuilder()
        text.append("【本机HTTP服务】\n")
        text.append("状态:").append(if (info.serverRunning) "运行中" else "未运行").append("\n")
        text.append("端口:").append(info.serverPort?.toString() ?: "-").append("\n")
        text.append("本次启动:").append(info.serverStartTime?.let { "${time(it)} (${duration(it)})" } ?: "-").append("\n")
        text.append("启动次数:").append(info.serverStartCount).append("  手动重启:").append(info.serverRestartCount).append("\n")
        text.append("当前连接数:").append(info.serverConnectionCount).append("\n")
        text.append("请求数:").append(info.serverRequestCount).append("  返回数据量:").append(size(info.serverBytesSent)).append("\n")
        text.append("状态码 2xx:").append(info.serverStatus2xx).append(" 4xx:").append(info.serverStatus4xx)
            .append(" 5xx:").append(info.serverStatus5xx).append(" 其他:").append(info.serverStatusOther).append("\n")
        text.append("接收连接失败:").append(info.acceptErrorCount).append("\n")
        info.lastAcceptError?.let { text.append("最后一次失败:").append(it).append("\n") }

        text.append("\n【网络】\n")
        text.append("请求数:").append(info.netRequestCount).append("  失败次数:").append(info.netErrorCount).append("\n")
        text.append("下载数据量:").append(size(info.netBytesReceived)).append("  上传数据量:").append(size(info.netBytesSent)).append("\n")
        text.append("下载任务数:").append(info.downloadingCount).append("  上传任务数:").append(info.uploadingCount).append("\n")

        text.append("\n【缓存】").append(if (groupsTime.isEmpty()) "" else " 查询时间:$groupsTime").append("\n")
        if (groups.isEmpty()) {
            text.append("没有缓存\n")
        }
        for (it in groups) {
            text.append(it.group).append(":").append(it.count).append("个 ").append(size(it.size)).append("\n")
        }

        text.append("\n【SDK】\n")
        text.append("启动时间:").append(time(info.sdkStartTime)).append("  已运行:").append(duration(info.sdkStartTime)).append("\n")

        text.append("\n【事件(最近100条)】\n")
        if (info.events.isEmpty()) {
            text.append("没有事件\n")
        }
        for (it in info.events) {
            text.append(time(it.time)).append(" ").append(it.msg).append("\n")
        }
        return text.toString()
    }

    private fun now() = time(System.currentTimeMillis())

    private fun time(millis: Long): String = timeFormat.format(Date(millis))

    private fun duration(since: Long): String {
        val seconds = maxOf(0L, (System.currentTimeMillis() - since) / 1000)
        val d = seconds / 86400
        val h = seconds % 86400 / 3600
        val m = seconds % 3600 / 60
        val s = seconds % 60
        return when {
            d > 0 -> "${d}天${h}小时${m}分"
            h > 0 -> "${h}小时${m}分${s}秒"
            m > 0 -> "${m}分${s}秒"
            else -> "${s}秒"
        }
    }

    private fun size(bytes: ULong): String {
        val value = bytes.toDouble()
        return when {
            value >= 1024.0 * 1024 * 1024 -> String.format(Locale.US, "%.2f GB", value / 1024 / 1024 / 1024)
            value >= 1024.0 * 1024 -> String.format(Locale.US, "%.2f MB", value / 1024 / 1024)
            value >= 1024.0 -> String.format(Locale.US, "%.1f KB", value / 1024)
            else -> "$bytes B"
        }
    }
}
