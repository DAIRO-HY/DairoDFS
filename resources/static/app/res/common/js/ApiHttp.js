let maskShowTimes123 = 0

class ApiHttp {
    constructor(url) {
        // const domain = "https://fly.hy-1.cn"
        const domain = ""
        this.url = domain + url
        this.param = {
            _clientFlag: 1,
        }

        //默认显示等待加载框
        this.isShowWaiting = true

        this.finishFunc = () => {
            //默认结束后什么也不做
        }
    }

    /**
     * 添加参数
     * @param key 参数名
     * @param value 参数值
     */
    add(key, value) {
        this.param[key] = value
        return this
    }

    /**
     * 添加参数
     * @param param 参数数据
     */
    addAll(param) {
        Object.assign(this.param, param);
        return this
    }

    /**
     * 设置请求成功回调函数
     * @param block 回调函数
     */
    success(block) {
        this.successFunc = block
        return this
    }

    /**
     * 设置请求失败回调函数(服务器端错误)
     * @param block 回调函数
     */
    fail(block) {
        this.failFunc = block
        return this
    }

    /**
     * 设置请求错误回调函数
     * @param block 回调函数
     */
    error(block) {
        this.errorFunc = block
        return this
    }

    /**
     * 设置请求完成回调函数
     * @param block 回调函数
     */
    finish(block) {
        this.finishFunc = block
        return this
    }

    /**
     * 不显示等待框
     */
    hide() {
        this.isShowWaiting = false
        return this
    }

    /**
     * 发起GET请求
     */
    get() {
        this.request("GET")
    }

    /**
     * 发起POST请求
     */
    post() {
        this.request("POST")
    }

    /**
     * 发起请求
     * @param method 请求方式
     */
    request(method) {
        this.addMask()
        let urlParam = "_clientFlag=0&_version=0&"
        for (let key in this.param) {
            const value = this.param[key]
            if (value == null || value === "") {
                continue
            }
            if (Array.isArray(value)) {//如果这是一个数组
                value.forEach(item => {
                    urlParam += key + "=" + encodeURIComponent(item) + "&"
                })
            } else {
                urlParam += key + "=" + encodeURIComponent(value) + "&"
            }
        }
        if (urlParam !== "") {//删除最后一个&
            urlParam = urlParam.substring(0, urlParam.length - 1)
        }

        $.ajax({
            url: this.url,
            method: method,
            data: urlParam,
            dataType: "TEXT",
            success: resText => {
                this.removeMask()
                let data = null
                try {
                    data = JSON.parse(resText)
                } catch {
                    data = resText
                }
                if (this.successFunc) {
                    this.successFunc(data)
                }
            },
            error: xhr => {
                this.removeMask()
                const resText = xhr.responseText
                if (resText === undefined) {
                    alert("网络连接失败")
                    return
                }
                let data = null
                try {
                    data = JSON.parse(resText)
                } catch {
                }
                if (data == null) {//数据解析失败
                    alert(resText)
                    return
                }
                if (data.code === undefined) {//非业务错误
                    alert(resText)
                    return
                }
                if (data.code === 5) {
                    window.location.href = "/app/login.html"
                    return
                }
                if (data.code === 2) {//单项目检查错误
                    const fieldError = data.data
                    this.addFiledError(fieldError)
                    return
                }
                if (this.failFunc) {
                    this.failFunc(data)
                } else {
                    alert(data.msg)
                }
            },
            complete: () => {
                if (this.finishFunc) {
                    this.finishFunc()
                }
            }
        })
    }

    /**
     * 添加验证失败消息
     * @param fieldError
     */
    addFiledError(fieldError) {
        for (let key in fieldError) {
            const messages = fieldError[key]
            const error = messages.join("；") + "。"
            const $input = $(`[name="${key}"]`)
            $input.addClass("is-invalid")
            const $parent = $input.parent()
            $parent.append(`<span class="text-danger" error-valid>${error}</span>`)
        }
        $(".is-invalid").first().focus()
    }

    addMask() {
        if (!this.isShowWaiting) {
            return
        }
        maskShowTimes123++
        if (maskShowTimes123 > 1) {
            return
        }
        const MASK_HTML =
            `<div class="ajax-mask">
            <div class="ajax-mask-animation"></div>
         </div>`
        $("body").append(MASK_HTML)
    }

    removeMask() {
        if (!this.isShowWaiting) {
            return
        }
        maskShowTimes123--
        if (maskShowTimes123 === 0) {
            $(".ajax-mask").remove()
        }
    }
}