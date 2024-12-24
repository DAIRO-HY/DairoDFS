package cn.dairo.dfs.controller.app.files.service

import cn.dairo.dfs.code.ErrorCode
import cn.dairo.dfs.controller.app.files.form.ShareForm
import cn.dairo.dfs.dao.DfsFileDao
import cn.dairo.dfs.dao.LocalFileDao
import cn.dairo.dfs.dao.ShareDao
import cn.dairo.dfs.dao.dto.DfsFileDto
import cn.dairo.dfs.dao.dto.LocalFileDto
import cn.dairo.dfs.dao.dto.ShareDto
import cn.dairo.dfs.exception.BusinessException
import cn.dairo.dfs.extension.*
import cn.dairo.dfs.service.DfsFileService
import cn.dairo.dfs.util.DBID
import cn.dairo.dfs.util.DfsFileUtil
import cn.dairo.dfs.util.file_handle.DfsFileHandleUtil
import io.swagger.v3.oas.annotations.Operation
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.stereotype.Service
import org.springframework.validation.annotation.Validated
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.ResponseBody
import java.io.File
import java.io.InputStream
import java.util.*
import kotlin.collections.ArrayList
import kotlin.collections.LinkedHashMap

/**
 * 文件分享操作Service
 */
@Service
class FileShareService {

    /**
     * 文件夹数据操作Service
     */
    @Autowired
    private lateinit var dfsFileService: DfsFileService

    /**
     * 文件数据操作Dao
     */
    @Autowired
    private lateinit var dfsFileDao: DfsFileDao

    /**
     * 分享文件数据操作Dao
     */
    @Autowired
    private lateinit var shareDao: ShareDao

    /**
     * 分享文件
     * @param form 分享表单
     */
    fun share(userId: Long, form: ShareForm): Long {
        if (form.names.isNullOrEmpty()) {
            throw BusinessException("请选择要分享的路径")
        }
        val endDate: Date? = if (form.endDateTime == 0L) {//永久
            null
        } else {
            Date(form.endDateTime!!)
        }

        //得到缩略图ID
        val thumbId = this.findThumb(userId, form.folder, form.names!!)

        //判断这是不是只是一个文件夹
        val folderFlag = this.isFolder(userId, form.folder, form.names!!)

        //获取分享文件的标题
        val title = this.getTitle(form.names!!)

        val shareDto = ShareDto()
        shareDto.title = title
        shareDto.userId = userId
        shareDto.endDate = endDate?.time
        shareDto.pwd = form.pwd
        shareDto.names = form.names!!.joinToString(separator = "|") { it }
        shareDto.folder = form.folder
        shareDto.folderFlag = folderFlag
        shareDto.thumb = thumbId
        shareDto.fileCount = form.names!!.size
        shareDto.date = Date()
        shareDto.id = DBID.id
        this.shareDao.add(shareDto)
        return shareDto.id!!
    }

    /**
     * 去查找缩略图
     */
    private fun findThumb(userId: Long, folder: String, names: List<String>): Long? {

        //得到分享的父文件夹ID
        val folderId = this.dfsFileService.getIdByFolder(userId, folder) ?: throw ErrorCode.NO_FOLDER

        //取出当前目录下的所有文件，用来查找缩略图
        val name2file = this.dfsFileDao.selectSubFile(userId, folderId).filter { it.hasThumb }.associateBy { it.name }

        //查找缩略图
        for (name in names) {
            if (name2file.containsKey(name)) {//如果有缩略图
                val nameFile = name2file[name]!!
                return nameFile.id
            }
        }
        return null
    }

    /**
     * 判断这是不是只是一个文件夹
     */
    private fun isFolder(userId: Long, folder: String, names: List<String>): Boolean {
        if (names.size > 1) {
            return false
        }

        //得到分享的文件ID
        val fileId = this.dfsFileService.getIdByFolder(userId, folder + "/" + names[0]) ?: throw ErrorCode.NO_FOLDER

        val fileDto = this.dfsFileDao.selectOne(fileId)!!
        if (fileDto.localId == 0L) {//这是一个文件夹
            return true
        }
        return false
    }

    /**
     * 获取分享文件的标题
     */
    private fun getTitle(names: List<String>): String {
        if (names.size == 1) {
            return names[0]
        }
        return names[0] + "等多个文件"
    }
}
