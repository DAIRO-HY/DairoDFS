package files

/** 文件列表页面 **/
//@get:/app/files
//@templates:app/files.html
func Html() {}

//@Operation(summary = "获取文件列表")
//    @PostMapping("/get_list")
//    @ResponseBody
//    fun getList(
//        @Parameter(name = "目标文件夹") @RequestParam("folder", required = true, defaultValue = "") folder: String
//    ): List<FileForm> {
//        val userId = super.loginId
//        val folderId = this.dfsFileService.getIdByFolder(userId, folder)
//        if (folderId == null) {
//            if (folder == "") {//新用户在为上传任何文件时,根目录不存在
//                return ArrayList()
//            }
//            throw ErrorCode.NO_FOLDER
//        }
//        val list = this.dfsFileDao.selectSubFile(userId, folderId).map {
//            FileForm().apply {
//                this.id = it.id!!
//                this.name = it.name!!
//                this.size = it.size!!
//                this.date = it.date!!.format()
//                this.fileFlag = it.isFile
//                this.thumb = if (it.hasThumb) "/app/files/thumb/${it.id}" else null
//            }
//        }
//        return list
//    }
//
//    @Operation(summary = "获取扩展文件的所有key值")
//    @PostMapping("/get_extra_keys")
//    @ResponseBody
//    fun getExtraKeys(
//        @Parameter(name = "文件id") @RequestParam("id", required = true) id: Long
//    ): List<String> {
//        return this.dfsFileDao.selectExtraNames(id)
//    }
//
//    @Operation(summary = "创建文件夹")
//    @PostMapping("/create_folder")
//    @ResponseBody
//    fun createFolder(
//        @Parameter(description = "文件夹名") @RequestParam(
//            "folder",
//            required = true,
//            defaultValue = ""
//        ) folder: String
//    ) {
//        val userId = super.loginId
//        val existsFileId = this.dfsFileDao.selectIdByPath(userId, folder.toDfsFileNameList)
//        if (existsFileId != null) {
//            throw ErrorCode.EXISTS
//        }
//        this.dfsFileService.mkdirs(userId, folder)
//    }
//
//    @Operation(summary = "删除文件")
//    @PostMapping("/delete")
//    @ResponseBody
//    fun delete(
//        @Parameter(description = "要删除的文件路径数组") @RequestParam(
//            "paths",
//            required = true
//        ) paths: List<String>
//    ) {
//        val userId = super.loginId
//        paths.forEach {
//            this.dfsFileService.setDelete(userId, it)
//        }
//    }
//
//    @Operation(summary = "重命名")
//    @PostMapping("/rename")
//    @ResponseBody
//    fun rename(
//        @Parameter(description = "源路径") @RequestParam("sourcePath", required = true) sourcePath: String,
//        @Parameter(description = "新名称") @RequestParam("name", required = true) name: String
//    ) {
//        if (name.contains('/')) {
//            throw BusinessException("文件名不能包含/")
//        }
//        val userId = super.loginId
//        this.dfsFileService.rename(userId, sourcePath, name)
//
//    }
//
//    @Operation(summary = "文件复制")
//    @PostMapping("/copy")
//    @ResponseBody
//    fun copy(
//        @Parameter(description = "源路径") @RequestParam("sourcePaths", required = true) sourcePaths: List<String>,
//        @Parameter(description = "目标文件夹") @RequestParam("targetFolder", required = true) targetFolder: String,
//        @Parameter(description = "是否覆盖目标文件") @RequestParam("isOverWrite", required = true) isOverWrite: Boolean
//    ) {
//        val userId = super.loginId
//        this.dfsFileService.copy(userId, sourcePaths, targetFolder, isOverWrite)
//    }
//
//    @Operation(summary = "文件移动")
//    @PostMapping("/move")
//    @ResponseBody
//    fun move(
//        @Parameter(description = "源路径") @RequestParam("sourcePaths", required = true) sourcePaths: List<String>,
//        @Parameter(description = "目标文件夹") @RequestParam("targetFolder", required = true) targetFolder: String,
//        @Parameter(description = "是否覆盖目标文件") @RequestParam("isOverWrite", required = true) isOverWrite: Boolean
//    ) {
//        val userId = super.loginId
//        this.dfsFileService.move(userId, sourcePaths, targetFolder, isOverWrite)
//    }
//
//    /**
//     * 分享文件
//     * @param form 分享表单
//     */
//    @Operation(summary = "分享文件")
//    @PostMapping("/share")
//    @ResponseBody
//    fun share(@Validated form: ShareForm): Long {
//        return this.fileShareService.share(super.loginId, form)
//    }
//
//    @Operation(summary = "文件或文件夹属性")
//    @PostMapping("/get_property")
//    @ResponseBody
//    fun getProperty(
//        @Parameter(description = "选择的路径列表") @RequestParam("paths", required = true) paths: List<String>
//    ): FilePropertyForm {
//        val userId = super.loginId
//        val form = FilePropertyForm()
//        if (paths.size > 1) {//多个文件时
//            val totalForm = ComputeSubTotalForm()
//            paths.forEach {
//                val fileId = this.dfsFileDao.selectIdByPath(userId, it.toDfsFileNameList) ?: throw ErrorCode.NO_EXISTS
//                val dfsFile = this.dfsFileDao.selectOne(fileId)!!
//                if (dfsFile.isFolder) {
//                    totalForm.folderCount += 1
//                    computeSubTotal(totalForm, userId, dfsFile.id!!)
//                } else {
//                    totalForm.fileCount += 1
//                    totalForm.size += dfsFile.size!!
//                }
//            }
//            form.size = totalForm.size.toDataSize
//            form.fileCount = totalForm.fileCount
//            form.folderCount = totalForm.folderCount
//        } else {//但文件时
//            val path = if (paths.isEmpty()) {//根目录时数组为空
//                ""
//            } else {
//                paths[0]
//            }
//            val fileId = this.dfsFileDao.selectIdByPath(userId, path.toDfsFileNameList) ?: throw ErrorCode.NO_EXISTS
//            val dfsFile = this.dfsFileDao.selectOne(fileId)!!
//            form.name = dfsFile.name
//            form.date = dfsFile.date!!.format()
//            form.path = path
//            form.isFile = dfsFile.isFile
//            if (dfsFile.isFile) {//文件时
//                form.size = dfsFile.size.toDataSize
//                form.contentType = dfsFile.contentType
//                val historyList = this.dfsFileDao.selectHistory(userId, fileId).map {
//                    FilePropertyHistoryForm().apply {
//                        this.id = it.id
//                        this.size = it.size.toDataSize
//                        this.date = it.date!!.format()
//                    }
//                }
//                form.historyList = historyList
//            } else {//文件夹时
//                val totalForm = ComputeSubTotalForm()
//                computeSubTotal(totalForm, userId, dfsFile.id!!)
//                form.fileCount = totalForm.fileCount
//                form.folderCount = totalForm.folderCount
//                form.size = totalForm.size.toDataSize
//            }
//        }
//        return form
//    }
//
//    /**
//     * 计算文件大小
//     */
//    private fun computeSubTotal(form: ComputeSubTotalForm, userId: Long, folderId: Long) {
//        this.dfsFileDao.selectSubFile(userId, folderId).forEach {
//            if (it.isFolder) {
//                form.folderCount += 1
//                computeSubTotal(form, userId, it.id!!)
//            } else {
//                form.fileCount += 1
//                form.size += it.size!!
//            }
//        }
//    }
//
//    @Operation(summary = "修改文件类型")
//    @PostMapping("/set_content_type")
//    @ResponseBody
//    fun setContentType(
//        @Parameter(description = "文件路径") @RequestParam("path", required = true) path: String,
//        @Parameter(description = "文件类型") @RequestParam("contentType", required = true) contentType: String
//    ) {
//        val userId = super.loginId
//        val fileId = this.dfsFileDao.selectIdByPath(userId, path.toDfsFileNameList) ?: throw ErrorCode.NO_EXISTS
//        this.dfsFileDao.setContentType(fileId, contentType)
//    }
//
//    /**
//     * 文件下载
//     * @param request 客户端请求
//     * @param response 往客户端返回内容
//     * @param id 文件ID
//     */
//    @GetMapping("/download_history/{id}/{name}")
//    fun downloadByHistory(
//        request: HttpServletRequest, response: HttpServletResponse, @PathVariable id: Long, @PathVariable name: String
//    ) {
//        val userId = super.loginId
//        val dfsFile = this.dfsFileDao.selectOne(id)
//        if (dfsFile == null) {
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//        if (dfsFile.userId != userId) {
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//        if (dfsFile.name != name) {
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//        DfsFileUtil.download(id, request, response)
//    }
//
//    /**
//     * 文件预览
//     * @param request 客户端请求
//     * @param response 往客户端返回内容
//     * @param dfsId dfs文件ID
//     */
//    @GetMapping("/preview/{dfsId}")
//    fun preview(
//        request: HttpServletRequest,
//        response: HttpServletResponse,
//        @PathVariable dfsId: Long,
//        @Parameter(description = "要下载的附属文件名") @RequestParam("extra", required = false) extra: String?,
//    ) {
//        val userId = super.loginId
//        val dfsDto = this.dfsFileDao.selectOne(dfsId)
//        if (dfsDto == null) {//文件不存在
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//        if (dfsDto.userId != userId) {//没有权限
//            throw ErrorCode.NOT_ALLOW
//        }
//        if (extra == null) {//下载源文件
//            DfsFileUtil.download(dfsDto, request, response)
//            return
//        }
//        val lowerName = dfsDto.name!!.lowercase()
//        if (lowerName.endsWith("psd") || lowerName.endsWith("psb")) {
//            val previewDto = this.dfsFileDao.selectExtra(dfsId, extra)
//            DfsFileUtil.download(previewDto, request, response)
//        } else if (lowerName.endsWith("cr3") || lowerName.endsWith("cr2")) {
//            val previewDto = this.dfsFileDao.selectExtra(dfsId, extra)
//            DfsFileUtil.download(previewDto, request, response)
//        } else if (lowerName.endsWith("cr3") || lowerName.endsWith("cr2")) {
//            val previewDto = this.dfsFileDao.selectExtra(dfsId, extra)
//            DfsFileUtil.download(previewDto, request, response)
//        } else if (lowerName.endsWith(".mp4")
//            || lowerName.endsWith(".mov")
//            || lowerName.endsWith(".avi")
//            || lowerName.endsWith(".mkv")
//            || lowerName.endsWith(".flv")
//            || lowerName.endsWith(".rm")
//            || lowerName.endsWith(".rmvb")
//            || lowerName.endsWith(".3gp")
//        ) {
//            //视频文件预览
//            val previewDto = this.dfsFileDao.selectExtra(dfsId, extra)
//            if (previewDto == null) {
//                //没有对应的画质
//            }
//            DfsFileUtil.download(previewDto, request, response)
//        } else {
//            DfsFileUtil.download(dfsDto, request, response)
//        }
//    }
//
//    /**
//     * 文件下载
//     * @param request 客户端请求
//     * @param response 往客户端返回内容
//     * @param name 文件名
//     * @param folder 所在文件夹
//     */
//    @SuppressWarnings("这里应该改成文件id访问，防止客户端缓存冲突")
//    @GetMapping("/download/{name}")
//    fun download(
//        request: HttpServletRequest, response: HttpServletResponse, @PathVariable name: String, folder: String
//    ) {
//        val userId = super.loginId
//        val path = folder + "/" + name
//        val fileId = this.dfsFileDao.selectIdByPath(userId, path.toDfsFileNameList)
//        if (fileId == null) {//文件不存在
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//        DfsFileUtil.download(fileId, request, response)
//    }
//
//    /**
//     * 缩略图下载
//     * @param request 客户端请求
//     * @param response 往客户端返回内容
//     * @param id 文件ID
//     */
//    @GetMapping("/thumb/{id}")
//    fun thumb(
//        request: HttpServletRequest,
//        response: HttpServletResponse,
//        @PathVariable id: Long
//    ) {
//        val dfsDto = this.dfsFileDao.selectOne(id)
//        if (dfsDto == null) {//文件不存在
//            response.status = HttpStatus.NOT_FOUND.value()
//            return
//        }
//        if (dfsDto.userId != super.loginId) {//没有权限
//            throw ErrorCode.NOT_ALLOW
//        }
//
//        //获取缩率图附属文件
//        val thumb = this.dfsFileDao.selectExtra(dfsDto.id!!, "thumb")
//        DfsFileUtil.download(thumb, request, response)
//    }
//}
