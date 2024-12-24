package cn.dairo.dfs.controller.app.folder_selector

import cn.dairo.dfs.code.ErrorCode
import cn.dairo.dfs.controller.app.folder_selector.form.FolderForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.dao.DfsFileDao
import cn.dairo.dfs.extension.*
import cn.dairo.dfs.service.DfsFileService
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.*

/**
 * 选择文件夹Controller
 */
@Controller
@RequestMapping("/app/folder_selector")
class FolderSelectorController : AppBase() {

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
     * 重置密码
     */
    @PostMapping("/get_list")
    @ResponseBody
    fun getList(folder: String): List<FolderForm> {
        val userId = super.loginId
        val folderId = this.dfsFileService.getIdByFolder(userId, folder)
            ?: throw ErrorCode.NO_FOLDER
        val list = ArrayList<FolderForm>()
        this.dfsFileDao.selectSubFile(userId, folderId).forEach {
            if (it.isFile) {
                return@forEach
            }
            val form = FolderForm()
            form.name = it.name
            form.size = it.size.toDataSize
            form.date = it.date?.format()
            form.fileFlag = it.isFile
            list.add(form)

        }
        return list
    }
}
