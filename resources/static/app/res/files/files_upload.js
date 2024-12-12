/**
 * 全部文件大小
 */
let sizeTotal = 0

/**
 * 已经上的文件大小,不包含正在上传和上传失败的文件
 */
let finishSizeTotal = 0

/**
 * 记录上次上传的大小,用来计算网速
 */
let preUploadedSize = 0

/**
 * 记录上次统计的时间,用来计算网速
 */
let preDatetime = new Date().getTime()

/**
 * 标记是否正在上传中
 */
let isUploading = false

/**
 * 是否正在显示上传列表
 */
let isShowList = false

/**
 * 存放正在上传的文件列表
 */
const uploadList = []

/**
 * 打开文件上传窗口
 */
function openUpload(folder) {
    $("#uploadFile").attr("data-folder", folder)
    $("#uploadFile").click()
}

/**
 * 显示或隐藏文件上传列表
 */
function onShowUploadFileListClick() {
    if ($("#uploadListDiv").is(":visible")) {
        $("#uploadListDiv").hide()
        $("#openListIcon").show()
        isShowList = false

        //移除上传列表,下次重绘
        $("#uploadingTable").empty()
    } else {
        $("#uploadListDiv").show()
        $("#openListIcon").hide()
        isShowList = true

        //重绘上传列表
        drawUploadList()
    }
}

/**
 * 文件发生变化时执行文件上传
 */
function onFileChange() {
    const folder = $("#uploadFile").attr("data-folder")
    $.each($("#uploadFile")[0].files, (_, file) => {
        file.progress = "等待上传"
        file.isFinish = false
        file.isFail = false
        file.folder = folder
        file.dataSize = file.size.toDataSize()
        uploadList.push(file)
    })

    //清楚已经选择
    $("#uploadFile").val("")

    //显示上传进度信息
    $("#uploadInfoDiv").show()

    //计算总的上传情况
    computeUploadTotal()
    if (isShowList) {
        drawUploadList()
    }
    loopUpload()
}

/**
 * 计算总的上传情况
 */
function computeUploadTotal() {
    let sizeTotalTemp = 0
    let finishSizeTotalTemp = 0
    uploadList.forEach((item) => {
        sizeTotalTemp += item.size
        if (item.isFinish) {
            finishSizeTotalTemp += item.size
        }
    })
    sizeTotal = sizeTotalTemp
    finishSizeTotal = finishSizeTotalTemp
}

/**
 * 显示正在上传的文件
 */
function drawUploadList() {
    $("#uploadingTable").createTable({
        data: uploadList, iconBtns: {
            "bi-x": function (item) {
                location.href = "/app/user_edit?id=" + item.id
            }
        },
        columns: [{
            data: "name", title: "文件名"
        }, {
            data: "dataSize", title: "大小", className: "text-end", width: 50
        }, {
            data: "progress", title: "进度", className: "text-end", width: 60
        }]
    })
}

/**
 * 失败的全部重试点击事件
 */
function onRetryClick() {
    $("#retryBtn").hide()
    uploadList.forEach((item) => {
        if (item.isFail) {
            item.isFail = false
            item.isFinish = false
            item.progress = "等待上传"
        }
    })

    //计算总的上传情况
    computeUploadTotal()
    loopUpload()
}

/**
 * 循环上传文件
 */
function loopUpload() {
    const fileIndex = uploadList.findIndex(item => !item.isFinish)
    if (fileIndex === -1) {//文件已经全部上传完成,但可能有上传失败的
        const failFileList = uploadList.filter(item => item.isFail)
        if (failFileList.length > 0) {//有上传的失败的文件
            $("#uploadProgressText").text(`成功:${uploadList.length - failFileList.length},失败:${failFileList.length}`)
        } else {
            $("#uploadProgressText").text("上传完成")
        }
        return
    }
    if (isUploading) {
        return
    }
    isUploading = true

    //计算总的上传情况
    computeUploadTotal()
    const file = uploadList[fileIndex]
    const token = sessionStorage.getItem("token")
    const formData = new FormData()
    formData.append("file", file)
    formData.append("token", token)
    formData.append("folder", file.folder)
    $.ajax({
        method: "POST",
        url: "/app/file_upload",
        dataType: "TEXT",
        data: formData,
        processData: false,
        contentType: false,
        xhr: function () {
            const xhr = new XMLHttpRequest()
            xhr.upload.addEventListener("progress", e => {
                if (e.lengthComputable) {
                    const progressTxt = ((e.loaded / e.total) * 100).toFixed(0) + "%"

                    //更新文件的上传进度
                    file.progress = progressTxt
                    computeAndUpdateTotalProgress(e.loaded)
                    reshowProgressUploadTable(fileIndex, progressTxt)
                }
            }, false)
            return xhr
        },
        success: function () {
            file.isFinish = true
            isUploading = false
            reload()
            loopUpload()
        },
        error: function () {
            $("#retryDiv").show()
            reshowProgressUploadTable(fileIndex, "失败")
            file.isFinish = true
            file.isFail = true
            isUploading = false
            loopUpload()
        }
    })
}

/**
 * 计算并更新总的进度
 */
function computeAndUpdateTotalProgress(loaded) {
    const uploadedTotal = finishSizeTotal + loaded
    const now = new Date().getTime()
    const totalProgress = (uploadedTotal / sizeTotal) * 100
    const speed = (uploadedTotal - preUploadedSize) / (now - preDatetime) * 1000

    //更新进度条
    $("#uploadProgressBar").css("width", totalProgress.toFixed(2) + "%")

    //显示网速
    $("#uploadProgressText").text(speed.toDataSize(1) + "/S")
    preUploadedSize = uploadedTotal
    preDatetime = now
}

/**
 * 更新文件列表的进度
 * @param index 当前文件所在的索引
 * @param progressTxt 要更新的文字
 */
function reshowProgressUploadTable(index, progressTxt) {
    if (!isShowList) {
        return
    }
    $("#uploadingTable").setCell(index, 2, progressTxt)
}