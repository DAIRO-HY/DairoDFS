package cn.dairo.dfs.controller.distributed

import cn.dairo.dfs.config.Constant
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.dao.LocalFileDao
import cn.dairo.dfs.exception.BusinessException
import cn.dairo.dfs.extension.bean
import cn.dairo.dfs.interceptor.MybatisInterceptor
import cn.dairo.dfs.sync.SyncByTable
import cn.dairo.dfs.sync.SyncByLog
import cn.dairo.dfs.util.DBID
import cn.dairo.dfs.util.DfsFileUtil
import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody
import kotlin.concurrent.thread

/**
 * 数据同步处理Controller
 */
//@Group:/distributed/{token}
class DistributedController : AppBase() {
    companion object {

        /**
         * 长连接心跳间隔时间
         */
        const val KEEP_ALIVE_TIME = 120 * 1000
    }

    /**
     * 本地存储文件数据操作Dao
     */
    @Autowired
    private lateinit var localFileDao: LocalFileDao

    /**
     * 记录分机端的请求
     */
    private val distributedClientResponseList = HashSet<DistributedClientResponseBean>()

    /**
     * 通知分机端同步
     */
    fun push() {
        this.distributedClientResponseList.forEach { res ->
            res.response.outputStream.use {
                it.write(1)
                it.flush()
            }
        }

        //一定要将同步客户端response信息列表复制一份在进行通知，因为调用notifyAll()时，其他线程有可能移除对象，而HashSet不能边遍历边移除对象，这回导致报错
        this.distributedClientResponseList.map { it }.forEach {
            synchronized(it) {

                //标记为已经结束
                it.isCancel = true
                (it as Object).notifyAll()
            }
        }
    }

    /**
     * 分机端同步监听请求
     * 这是一个长连接，直到主机端有数据变更之后才返回
     * @param clientToken 分机端的票据
     * @param lastId 分机端同步到日志最大ID,用来解决分机端在判断是否最新日志的过程中,又有新的日志增加,虽然是小概率事件,但还是有发生的可能
     */
    @GetMapping("/{clientToken}/listen")
    @ResponseBody
    fun listen(response: HttpServletResponse, @PathVariable clientToken: String, lastId: Long) {

        //检查客户端token是否已经存在，保证同一个token的客户端只能有一个等待
        this.distributedClientResponseList.find { it.clientToken == clientToken }?.also {
            try {

                //将上一个标记为已经结束
                it.isCancel = true
                it.response.outputStream.close()
            } finally {
                synchronized(it) {
                    (it as Object).notifyAll()
                }
            }
        }

        //构建分机端同步response信息
        val responseBean = DistributedClientResponseBean(clientToken, response)
        synchronized(responseBean) {

            //添加新的等待
            this.distributedClientResponseList.add(responseBean)
            try {
                while (true) {
                    if (MybatisInterceptor::class.bean.lastID > lastId) {//分机端数据并不是最新的
                        response.outputStream.write(1)
                        break
                    }

                    //间隔一段时间往客户端发送0，以保持长连接
                    response.outputStream.write(0)
                    response.outputStream.flush()
                    (responseBean as Object).wait(KEEP_ALIVE_TIME.toLong())
                    if (responseBean.isCancel) {
                        break
                    }
                }
            } finally {
                this.distributedClientResponseList.remove(responseBean)
            }
        }
    }

    /**
     * 获取sql日志
     */
    @GetMapping("/get_log")
    @ResponseBody
    fun getLog(lastId: Long): List<Map<String, Any?>> {
        val logList = Constant.dbService.selectList(
            "select id,date,sql,param from sql_log where id > ? order by id limit 100", lastId
        )
        return logList
    }

    /**
     * 主机发起同步通知
     */
    @GetMapping("/push_notify")
    @ResponseBody
    fun pushNotify() {
        thread {
            SyncByLog.start()
        }
    }

    /**
     * 获取一个断面ID，防止再全量同步的过程中，主机又增加数据，导致全量同步数据不完整
     * 其实就是当前服务器时间戳
     */
    @GetMapping("/get_aop_id")
    @ResponseBody
    fun getAopId() = DBID.id

    /**
     * 获取每个表的id
     * @param tbName 表名
     * @param lastId 已经取到的最后一个id
     * @param aopId 断面ID
     */
    @GetMapping("/get_table_id")
    @ResponseBody
    fun getTableId(tbName: String, lastId: Long, aopId: Long): String {
        if (SyncByTable.isRuning || SyncByLog.isRunning) {
            throw BusinessException("主机正在同步数据中，请等待完成后继续。")
        }

        //条数不宜过大,过大可能导致客户端请求失败
        val maxLimit = 100
        return Constant.dbService.selectList(
            String::class,
            "select id from $tbName where id > ? and id < ? order by id asc limit ?",
            lastId,
            aopId,
            maxLimit
        ).joinToString(separator = ",") { it }
    }

    /**
     * 获取表数据
     * @param tbName 表名
     * @param ids 要取的数据id列表
     */
    @GetMapping("/get_table_data")
    @ResponseBody
    fun getTableData(tbName: String, ids: String): List<*> {
        if (SyncByTable.isRuning || SyncByLog.isRunning) {
            throw BusinessException("主机正在同步数据中，请等待完成后继续。")
        }
        return Constant.dbService.selectList(
            "select * from $tbName where id in ($ids)"
        )
    }


    /**
     * 文件下载
     * @param request 客户端请求
     * @param response 往客户端返回内容
     * @param id 文件ID
     */
    @GetMapping("/download/{md5}")
    fun download(
        request: HttpServletRequest,
        response: HttpServletResponse,
        @PathVariable md5: String
    ) {
        if (SyncByTable.isRuning || SyncByLog.isRunning) {
            throw BusinessException("主机正在同步数据中，请等待完成后继续。")
        }
        val localFileDto = this.localFileDao.selectByFileMd5(md5)
        if (localFileDto == null) {
            response.status = HttpStatus.NOT_FOUND.value()
            return
        }
        response.reset() //清除buffer缓存
        DfsFileUtil.download(localFileDto, request, response)
    }
}
