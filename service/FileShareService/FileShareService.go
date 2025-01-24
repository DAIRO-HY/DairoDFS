package FileShareService

import (
    "DairoDFS/controller/app/files/form"
    "DairoDFS/dao/DfsFileDao"
    "DairoDFS/dao/dto"
    "DairoDFS/exception"
    "DairoDFS/service/DfsFileService"
)

/**
 * 文件分享操作Service
 */

    /**
     * 分享文件
     * @param form 分享表单
     */
    func Share(userId int64, form form.ShareForm) int64 {
        if len(form.Names) == 0 {
            throw BusinessException("请选择要分享的路径")
        }
        //endDate: Date? = if (form.endDateTime == 0L) {//永久
        //    null
        //} else {
        //    Date(form.endDateTime!!)
        //}

        //得到缩略图ID
        thumbId = this.findThumb(userId, form.folder, form.names!!)

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
    func findThumb(userId int64, folder string, names []string) (int64,error) {

        //得到分享的父文件夹ID
        folderId,err := DfsFileService.GetIdByFolder(userId, folder,false)
        if err != nil{
            return 0,err
        }
        if folderId == 0{
            return 0, exception.NO_FOLDER()
        }

        //取出当前目录下的所有文件，用来查找缩略图
        subFiles := DfsFileDao.SelectSubFile(userId, folderId)

        //文件名对应的文件信息
        name2file := make(map[string]dto.DfsFileThumbDto)
        for _,it:=range subFiles{
            if it.HasThumb{
                name2file[it.Name] = it
            }
        }

        //查找缩略图
        for _,name := range names {
            thumbFile,isExists := name2file[name]
            if isExists {//如果有缩略图
                return thumbFile.Id,nil
            }
        }
        return 0,nil
    }

    /**
     * 判断这是不是只是一个文件夹
     */
    func isFolder(userId int64, folder string, names []string) (bool,error) {
        if len(names) > 1 {
            return false,nil
        }

        //得到分享的文件ID
        fileId,err := DfsFileService.GetIdByFolder(userId, folder + "/" + names[0],false)
        if err != nil{
            return false,err
        }
        if fileId == 0{
            return false,exception.NO_FOLDER()
        }
        fileDto,_ := DfsFileDao.SelectOne(fileId)
        if fileDto.LocalId == 0 {//这是一个文件夹
            return true,nil
        }
        return false,nil
    }

    /**
     * 获取分享文件的标题
     */
    func getTitle(names []string) string {
        if len(names) == 1 {
            return names[0]
        }
        return names[0] + "等多个文件"
    }
