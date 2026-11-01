package cn.dairo.file.sdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.runBlocking
import org.junit.AfterClass
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.BeforeClass
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest
import java.util.concurrent.CountDownLatch
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit

/**
 * 在模拟器或者手机上运行,访问电脑上的DFS
 * 测试参数:dfsServer(例如 http://10.0.2.2:8031)、dfsToken、fileId、fileMd5
 */
@RunWith(AndroidJUnit4::class)
class FileSdkTest {

    companion object {
        private val args = InstrumentationRegistry.getArguments()
        private val server = args.getString("dfsServer")!!
        private val token = args.getString("dfsToken")!!
        private val fileId = args.getString("fileId")!!
        private val fileMd5 = args.getString("fileMd5")!!

        private val context = InstrumentationRegistry.getInstrumentation().targetContext
        private val root = File(context.noBackupFilesDir, "dairo-file-sdk")
        private lateinit var sdk: FileSdk

        /** 收到的事件 */
        private val events = LinkedBlockingQueue<Event>()

        @BeforeClass
        @JvmStatic
        fun setUp() {
            root.deleteRecursively()
            sdk = FileSdk(SdkConfig(rootDir = root.absolutePath))
            sdk.setListener(object : EventListener {
                override fun onEvent(event: Event) {
                    events.add(event)
                }
            })
            sdk.setDfsAccount("test", server, token)
        }

        @AfterClass
        @JvmStatic
        fun tearDown() {
            sdk.close()
        }
    }

    private val url = "$server/api/files/download/$fileId/test.bin"
    private val options = Options(headers = mapOf("Authorization" to "Bearer $token"), name = "test.bin")

    private fun md5(data: ByteArray) = MessageDigest.getInstance("MD5").digest(data).joinToString("") { "%02x".format(it) }

    /** 等待事件 */
    private fun waitEvent(predicate: (Event) -> Boolean): Event {
        val deadline = System.currentTimeMillis() + 30_000
        while (System.currentTimeMillis() < deadline) {
            val event = events.poll(1, TimeUnit.SECONDS) ?: continue
            if (predicate(event)) {
                return event
            }
        }
        fail("没有等到事件")
        throw IllegalStateException()
    }

    @Test
    fun fetchAsyncAndCache() = runBlocking {
        val path = sdk.fetchAsync(url, options)
        assertEquals(fileMd5, md5(File(path).readBytes()))
        assertEquals(path, sdk.cachedPath(url, options))

        val id = sdk.cacheId(url, options)
        val info = sdk.get(id)!!
        assertEquals(DownloadState.FINISH, info.state)
        assertEquals(5242880uL, info.size)
    }

    @Test
    fun fetchWithCallback() {
        val latch = CountDownLatch(1)
        var result: String? = null
        val other = Options(headers = options.headers, cacheKey = "callback-test")
        sdk.fetch(url, other, object : FetchListener {
            override fun onProgress(size: ULong?, downloaded: ULong) {}

            override fun onSuccess(path: String) {
                result = path
                latch.countDown()
            }

            override fun onError(error: SdkException) {
                latch.countDown()
            }
        })
        assertTrue(latch.await(30, TimeUnit.SECONDS))
        assertEquals(fileMd5, md5(File(result!!).readBytes()))
    }

    @Test
    fun localHttpWithRange() {
        //播放器、图片组件通过本机HTTP地址访问
        val local = sdk.open(url, Options(headers = options.headers, cacheKey = "local-http-test", name = "test.bin"))
        assertTrue(local.startsWith("http://127.0.0.1:"))
        val conn = URL(local).openConnection() as HttpURLConnection
        conn.setRequestProperty("Range", "bytes=1000000-1000099")
        assertEquals(206, conn.responseCode)
        assertEquals("bytes 1000000-1000099/5242880", conn.getHeaderField("Content-Range"))
        val part = conn.inputStream.readBytes()
        conn.disconnect()

        val path = runBlocking { sdk.fetchAsync(url, options) }
        assertArrayEquals(File(path).readBytes().copyOfRange(1000000, 1000100), part)
    }

    @Test
    fun errors() {
        try {
            sdk.open("abc", Options())
            fail()
        } catch (e: SdkException.InvalidUrl) {
            assertTrue(e.message.contains("abc"))
        }
        try {
            runBlocking { sdk.fetchAsync(url, Options(cacheKey = "no-token")) }
            fail()
        } catch (e: SdkException.UrlExpired) {
            assertEquals(401.toUShort(), e.status)
        }
    }

    @Test
    fun uploadFile() {
        val data = ByteArray(3_000_000) { (it * 7 % 251).toByte() }
        val file = File(context.cacheDir, "upload.bin").apply { writeBytes(data) }
        val id = sdk.addUploads(listOf(UploadItem(UploadSource.File(file.absolutePath), "/sdk-test/android-${System.currentTimeMillis()}.bin")))[0]
        val event = waitEvent { it is Event.UploadStateChanged && it.id == id && it.state != UploadState.WAITING && it.state != UploadState.UPLOADING }
        assertEquals(UploadState.FINISH, (event as Event.UploadStateChanged).state)
        assertEquals(md5(data), sdk.getUpload(id)!!.md5)
    }

    @Test
    fun uploadCustomSource() {
        //由App提供数据,例如相册里的照片
        val data = ByteArray(1_500_000) { (it * 13 % 251).toByte() }
        val closed = LinkedBlockingQueue<String>()
        sdk.setUploadSourceProvider(object : UploadSourceProvider {
            override fun size(id: String): ULong = data.size.toULong()

            override fun read(id: String, offset: ULong, len: ULong): ByteArray {
                val end = minOf(offset.toLong() + len.toLong(), data.size.toLong()).toInt()
                return data.copyOfRange(offset.toInt(), end)
            }

            override fun close(id: String) {
                closed.add(id)
            }
        })
        val id = sdk.addUploads(listOf(UploadItem(UploadSource.Custom("photo-1"), "/sdk-test/photo-${System.currentTimeMillis()}.jpg")))[0]
        val event = waitEvent { it is Event.UploadStateChanged && it.id == id && it.state != UploadState.WAITING && it.state != UploadState.UPLOADING }
        assertEquals(UploadState.FINISH, (event as Event.UploadStateChanged).state)
        assertEquals("photo-1", closed.poll(5, TimeUnit.SECONDS))
        assertNotNull(sdk.listUploads(UploadQuery(account = "test")).find { it.id == id })
    }
}
