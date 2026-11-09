package cn.dairo.file.sdk

import android.app.Activity
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.TextView
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.runner.lifecycle.ActivityLifecycleMonitorRegistry
import androidx.test.runner.lifecycle.Stage
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.net.HttpURLConnection
import java.net.URL

/** 调试接口和调试页面,不需要DFS */
@RunWith(AndroidJUnit4::class)
class DebugTest {

    private val instrumentation = InstrumentationRegistry.getInstrumentation()
    private val context = instrumentation.targetContext

    private fun makeSdk(): FileSdk {
        val root = File(context.noBackupFilesDir, "dairo-file-sdk-debug-${System.nanoTime()}")
        return FileSdk(SdkConfig(rootDir = root.path))
    }

    private fun ping(base: String): Pair<Int, String> {
        val conn = URL("$base/ping").openConnection() as HttpURLConnection
        conn.connectTimeout = 3000
        conn.readTimeout = 3000
        return conn.responseCode to conn.inputStream.bufferedReader().use { it.readText() }
    }

    @Test
    fun pingSelfCheckRestart() {
        val sdk = makeSdk()
        val base = sdk.localBase()
        val port = URL(base).port
        val (status, body) = ping(base)
        assertEquals(200, status)
        assertTrue(body, body.contains("\"ok\":true") && body.contains("\"port\":$port"))

        val result = runBlocking { sdk.selfCheck() }
        assertTrue(result.toString(), result.ok)
        assertEquals(port, result.port?.toInt())

        assertEquals(port, sdk.restartServer().toInt())
        assertEquals(200, ping(base).first)
        val info = sdk.debugInfo()
        assertEquals(1UL, info.serverRestartCount)
        assertTrue(info.serverStatus2xx >= 2UL)

        sdk.stopServer()
        val stopped = runBlocking { sdk.selfCheck() }
        assertFalse(stopped.ok || stopped.running)
        assertEquals("服务没有运行", stopped.error)
    }

    private fun currentActivity(): Activity {
        var activity: Activity? = null
        instrumentation.runOnMainSync {
            activity = ActivityLifecycleMonitorRegistry.getInstance().getActivitiesInStage(Stage.RESUMED).firstOrNull()
        }
        return activity!!
    }

    private fun views(view: View): List<View> =
        if (view is ViewGroup) listOf(view) + (0 until view.childCount).flatMap { views(view.getChildAt(it)) } else listOf(view)

    /** 按钮下面的结果文字 */
    private fun resultOf(activity: Activity, buttonText: String): String {
        var text = ""
        instrumentation.runOnMainSync {
            val all = views(activity.window.decorView)
            val index = all.indexOfFirst { it is Button && it.text == buttonText }
            text = (all[index + 1] as TextView).text.toString()
        }
        return text
    }

    private fun click(activity: Activity, buttonText: String) {
        instrumentation.runOnMainSync {
            views(activity.window.decorView).first { it is Button && it.text == buttonText }.performClick()
        }
    }

    private fun waitResult(activity: Activity, buttonText: String): String {
        val start = System.currentTimeMillis()
        while (System.currentTimeMillis() - start < 10000) {
            val text = resultOf(activity, buttonText)
            if (text.isNotEmpty() && !text.endsWith("...")) {
                return text
            }
            Thread.sleep(100)
        }
        return resultOf(activity, buttonText)
    }

    @Test
    fun debugActivity() {
        val sdk = makeSdk()
        val port = URL(sdk.localBase()).port
        sdk.putData("http://example.com/a.jpg", Options(group = "image"), ByteArray(2048))
        FileSdkDebugActivity.start(context, sdk)
        instrumentation.waitForIdleSync()
        Thread.sleep(500)
        val activity = currentActivity()
        assertTrue(activity is FileSdkDebugActivity)

        click(activity, "自检(SDK内部请求ping)")
        val check = waitResult(activity, "自检(SDK内部请求ping)")
        assertTrue(check, check.contains("正常,端口:$port,TCP:已连接,状态码:200"))

        click(activity, "App请求ping")
        val ping = waitResult(activity, "App请求ping")
        assertTrue(ping, ping.contains("正常,端口:$port,状态码:200"))

        click(activity, "强制重启本机服务")
        val restart = waitResult(activity, "强制重启本机服务")
        assertTrue(restart, restart.contains("重启完成,端口:$port(原端口:$port)"))

        Thread.sleep(1500)
        var info = ""
        instrumentation.runOnMainSync {
            info = views(activity.window.decorView).filterIsInstance<TextView>().filter { it !is Button }.joinToString("\n") { it.text }
        }
        assertTrue(info, info.contains("手动重启:1") && info.contains("停止:手动重启") && info.contains("自检正常"))
        assertTrue(info, info.contains("查询时间:") && info.contains("image:1个 2.0 KB"))
        println("调试页面内容:\n$info")
        //留时间截图
        Thread.sleep(8000)
    }
}
