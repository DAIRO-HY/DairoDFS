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
 * 打开文件夹上传窗口
 * 浏览器的选择窗口一次只能选一个文件夹,要一次上传多个文件夹可以拖到文件列表里
 */
function openUploadFolder(folder) {
    $("#uploadFolder").attr("data-folder", folder)
    $("#uploadFolder").click()
}

/**
 * 系统自动生成的文件,上传文件夹时跳过
 */
const IGNORE_FILE_NAMES = [".DS_Store", "Thumbs.db", "desktop.ini"]

/**
 * 文件发生变化时执行文件上传
 */
function onFileChange() {
    const files = Array.from($("#uploadFile")[0].files)
    addUpload($("#uploadFile").attr("data-folder"), files.map(file => ({file: file, path: file.name})))

    //清除已经选择
    $("#uploadFile").val("")
}

/**
 * 选择文件夹之后执行上传
 * webkitRelativePath包含选择的文件夹名,例如"照片/2024/a.jpg"
 */
function onFolderChange() {
    const files = Array.from($("#uploadFolder")[0].files)
    addUpload($("#uploadFolder").attr("data-folder"), files.map(file => ({file: file, path: file.webkitRelativePath || file.name})))

    //清除已经选择
    $("#uploadFolder").val("")
}

/**
 * 拖拽文件或文件夹到文件列表时上传,可以同时拖入多个文件夹
 */
async function onDropUpload(folder, dataTransfer) {
    const entries = Array.from(dataTransfer.items || [])
        .filter(item => item.kind === "file")
        .map(item => item.webkitGetAsEntry ? item.webkitGetAsEntry() : null)
    if (entries.length === 0 || entries.some(it => it == null)) {//浏览器不支持读取文件夹,只上传文件
        addUpload(folder, Array.from(dataTransfer.files).map(file => ({file: file, path: file.name})))
        return
    }
    const fileList = []
    const emptyFolders = []
    try {
        for (const entry of entries) {
            await readEntry(entry, "", fileList, emptyFolders)
        }
    } catch (e) {
        alert("读取文件夹失败:" + e)
        return
    }

    //空文件夹没有文件可以上传,需要单独创建
    for (const path of emptyFolders) {
        await createFolderIfAbsent(folder + "/" + path)
    }
    if (fileList.length === 0) {
        reload()
        return
    }
    addUpload(folder, fileList)
}

/**
 * 递归读取拖入的文件夹
 * - parent 相对于拖入位置的父路径
 * - fileList 读取到的文件
 * - emptyFolders 读取到的空文件夹
 */
async function readEntry(entry, parent, fileList, emptyFolders) {
    const path = parent === "" ? entry.name : parent + "/" + entry.name
    if (entry.isFile) {
        const file = await new Promise((resolve, reject) => entry.file(resolve, reject))
        fileList.push({file: file, path: path})
        return
    }
    if (!entry.isDirectory) {
        return
    }
    const reader = entry.createReader()
    let count = 0

    //readEntries每次只返回一部分(Chrome每次最多100个),要一直读到返回空为止
    while (true) {
        const children = await new Promise((resolve, reject) => reader.readEntries(resolve, reject))
        if (children.length === 0) {
            break
        }
        count += children.length
        for (const child of children) {
            await readEntry(child, path, fileList, emptyFolders)
        }
    }
    if (count === 0) {
        emptyFolders.push(path)
    }
}

/**
 * 创建文件夹,已经存在时不报错
 */
function createFolderIfAbsent(folder) {
    return new Promise(resolve => {
        $.ajax({
            method: "POST",
            url: "/api/files/create_folder",
            data: {folder: folder},
            complete: () => resolve()
        })
    })
}

/**
 * 添加到上传列表并开始上传
 * - folder 上传到的文件夹
 * - items 要上传的文件,path是相对于folder的路径,包含文件名
 */
function addUpload(folder, items) {
    items.forEach(({file, path}) => {
        const names = path.split("/").filter(it => it !== "")
        if (IGNORE_FILE_NAMES.includes(names[names.length - 1]) && names.length > 1) {//只在上传文件夹时跳过
            return
        }
        file.uploadName = names.pop()
        file.progress = "等待上传"
        file.isFinish = false
        file.isFail = false
        file.folder = names.length === 0 ? folder : folder + "/" + names.join("/")
        file.path = path
        file.dataSize = file.size.toDataSize()
        uploadList.push(file)
    })
    if (uploadList.length === 0) {
        return
    }

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
        columns: [{
            data: "path", title: "文件名", render: $.fn.dataTable.render.text()
        }, {
            data: "dataSize", title: "大小", className: "text-end", width: 50
        }, {
            data: "progress", title: "进度", className: "text-end", width: 60
        }],
        data: uploadList
    })
}

/**
 * 失败的全部重试点击事件
 */
function onRetryClick() {
    $("#retryDiv").hide()
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
        reload()
        return
    }
    if (isUploading) {
        return
    }
    isUploading = true

    //计算总的上传情况
    computeUploadTotal()
    const file = uploadList[fileIndex]
    const formData = new FormData()
    formData.append("folder", file.folder)
    //必须指定文件名:选择文件夹上传时,Chrome默认把相对路径(例如"ee/ff/gg.txt")当作文件名发送,会和folder里的路径重复
    formData.append("file", file, file.uploadName)
    $.ajax({
        method: "POST",
        url: "/api/file_upload",
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
            reloadLater()
            loopUpload()
        },
        error: function (res) {
            if (res.status === 401) {
                window.location.href = "/login.html"
                return
            }
            let errorMsg = ""
            try {
                errorMsg = JSON.parse(res.responseText).msg
            }catch{
                errorMsg = res.responseText
            }
            $("#retryDiv").show()
            reshowProgressUploadTable(fileIndex, errorMsg)
            file.isFinish = true
            file.isFail = true
            file.progress = errorMsg
            isUploading = false
            loopUpload()
        }
    })
}

/**
 * 刷新文件列表的定时器
 */
let reloadTimer = null

/**
 * 上传过程中最多2秒刷新一次文件列表,上传文件夹时文件很多,每个文件都刷新太频繁
 */
function reloadLater() {
    if (reloadTimer != null) {
        return
    }
    reloadTimer = setTimeout(() => {
        reloadTimer = null
        reload()
    }, 2000)
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