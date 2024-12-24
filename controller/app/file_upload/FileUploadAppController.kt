package cn.dairo.dfs.controller.app.file_upload

import cn.dairo.dfs.code.ErrorCode
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.dao.LocalFileDao
import cn.dairo.dfs.dao.dto.DfsFileDto
import cn.dairo.dfs.dao.dto.LocalFileDto
import cn.dairo.dfs.exception.BusinessException
import cn.dairo.dfs.extension.*
import cn.dairo.dfs.service.DfsFileService
import cn.dairo.dfs.util.DfsFileUtil
import cn.dairo.dfs.util.DfsFileUtil.dfsContentType
import cn.dairo.dfs.util.file_handle.DfsFileHandleUtil
import io.swagger.v3.oas.annotations.Operation
import io.swagger.v3.oas.annotations.Parameter
import jakarta.servlet.http.HttpServletRequest
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.beans.factory.annotation.Value
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.*
import org.springframework.web.multipart.MultipartFile
import java.io.File
import java.io.FileOutputStream

/**
 * 文件上传Controller
 */
@Controller
@RequestMapping("/app/file_upload")
class FileUploadAppController : AppBase() {

    /**
     * 文件夹数据操作Service
     */
    @Autowired
    private lateinit var dfsFileService: DfsFileService

    /**
     * 本地存储文件数据操作Dao
     */
    @Autowired
    private lateinit var localFileDao: LocalFileDao

    @Value("\${data.path}")
    private lateinit var dataPath: String

    /**
     * 记录当前正在上传的文件
     * 避免同一个文件同时上传导致文件数据混乱
     * md5 -> 上传时间戳
     */
    private val uploadingFileMap = HashMap<String, Long>()

    @Operation(summary = "浏览器文件上传")
    @PostMapping
    @ResponseBody
    fun upload(
        @RequestParam("file") mulFile: MultipartFile,
        folder: String,
        contentType: String?
    ) {

        //得到文件实际路径
        val file = mulFile.getField("part")?.getField("fileItem")?.getField("tempFile") as File

        //文件名
        val name = mulFile.originalFilename!!
        val path = folder + "/" + name

        //检查文件路径是否合法
        DfsFileUtil.checkPath(folder)

        //文件MD5
        val md5 = file.md5

        //将文件存放到指定目录
        val localFileDto = this.dfsFileService.saveToLocalFile(md5, file.inputStream())
        this.addDfsFile(super.loginId, localFileDto, path, contentType)

        //开启生成缩略图线程
        DfsFileHandleUtil.start()
    }

    @Operation(summary = "以流的方式上传文件")
    @PostMapping("/by_stream/{md5}")
    @ResponseBody
    fun byStream(request: HttpServletRequest, @PathVariable md5: String) {
        synchronized(this.uploadingFileMap) {
            if (this.uploadingFileMap.containsKey(md5)) {
                throw ErrorCode.FILE_UPLOADING
            }
            this.uploadingFileMap[md5] = System.currentTimeMillis()
        }

        try {//保存到文件
            val file = File(this.dataPath + "/temp/" + md5)

            //文件输出流
            FileOutputStream(file, true).use {
                request.inputStream.transferTo(it)
//                val stream = request.inputStream
//                val data = ByteArray(64 * 1024)
//                var len: Int
//                while (stream.read(data).also { len = it } != -1) {
//                    sleep(10)
//                    it.write(data, 0, len)
//                }
            }

            //计算文件的MD5
            val fileMd5 = file.md5
            if (md5 != fileMd5) {
                file.delete()
                throw BusinessException("文件校验失败")
            }

            //将文件存放到指定目录
            this.dfsFileService.saveToLocalFile(md5, file.inputStream())
            file.delete()

            //开启生成缩略图线程
            DfsFileHandleUtil.start()
        } finally {
            synchronized(this.uploadingFileMap) {
                this.uploadingFileMap.remove(md5)
            }
        }
    }

    @Operation(summary = "获取文件已经上传大小")
    @PostMapping("/get_uploaded_size")
    @ResponseBody
    fun getUploadedSize(@Parameter(name = "文件的MD5") @RequestParam("md5", required = true) md5: String): Long {
        if (this.uploadingFileMap.containsKey(md5)) {
            throw ErrorCode.FILE_UPLOADING
        }
        val file = File(dataPath + "/temp/" + md5)
        if (!file.exists()) {
            return 0
        }
        return file.length()
    }

    /**
     * 通过MD5上传
     * @param md5 文件md5
     * @param path 文件路径
     */
    @PostMapping("/by_md5")
    @ResponseBody
    fun byMd5(md5: String, path: String, contentType: String?) {

        val localFileDto = this.localFileDao.selectByFileMd5(md5)
            ?: throw ErrorCode.NO_EXISTS

        //添加到DFS文件
        this.addDfsFile(super.loginId, localFileDto, path, contentType)

        //删除上传的临时文件
        File(dataPath + "/temp/" + md5).delete()

        //开启生成缩略图线程
        DfsFileHandleUtil.start()
    }


    /**
     * 添加到DFS文件
     * @param userId 会员id
     * @param localFileDto 本地文件Dto
     * @param path DFS文件路径
     * @param fileContentType 文件类型
     */
    private fun addDfsFile(userId: Long, localFileDto: LocalFileDto, path: String, fileContentType: String?) {

        //文件名
        val name = path.fileName

        //上级文件夹名
        val folder = path.fileParent

        //获取文件夹ID
        val folderId = this.dfsFileService.getIdByFolder(userId, folder, true)
        val contentType = if (!fileContentType.isNullOrBlank()) {
            fileContentType
        } else {
            name.fileExt.dfsContentType
        }

        val localFile = File(localFileDto.path!!)
        val fileDto = DfsFileDto()
        fileDto.userId = userId
        fileDto.localId = localFileDto.id
        fileDto.name = name
        fileDto.contentType = contentType
        fileDto.size = localFile.length()
        fileDto.parentId = folderId
        this.dfsFileService.addFile(fileDto, true)
    }
}
