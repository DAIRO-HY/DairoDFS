// 仿java
String.prototype.startsWith = function (str) {
    if (str == null || str == "" || this.length == 0
        || str.length > this.length)
        return false;
    if (this.substr(0, str.length) == str)
        return true;
    else
        return false;
    return true;
};

// 仿java
String.prototype.endsWith = function (str) {
    if (str == null || str == "" || this.length == 0
        || str.length > this.length)
        return false;
    if (this.substring(this.length - str.length) == str)
        return true;
    else
        return false;
    return true;
};

/**
 * 数据流量单位换算
 */
Number.prototype.toDataSize = function (fraction = 2) {
    if (this == null) {
        return "0B"
    }
    const value = this
    if (value >= 1024 * 1024 * 1024 * 1024) {
        return (this / (1024 * 1024 * 1024 * 1024)).toFixed(fraction) + "TB"
    }
    if (value >= 1024 * 1024 * 1024) {
        return (this / (1024 * 1024 * 1024)).toFixed(fraction) + "GB"
    }
    if (value >= 1024 * 1024) {
        return (this / (1024 * 1024)).toFixed(fraction) + "MB"
    }
    if (value >= 1024) {
        return (this / (1024)).toFixed(fraction) + "KB"
    }
    return this.toFixed(fraction) + "B"
}

/**
 * 日期格式化扩展
 * @param pattern
 * @returns {string}
 */
Date.prototype.dateFormat = function (pattern = "yyyy-MM-dd hh:mm:ss") {
    const o = {
        "M+": this.getMonth() + 1, // month
        "d+": this.getDate(), // day
        "h+": this.getHours(), // hour
        "m+": this.getMinutes(), // minute
        "s+": this.getSeconds(), // second
        "q+": Math.floor((this.getMonth() + 3) / 3), // quarter
        "S": this.getMilliseconds()
        // millisecond
    };

    if (/(y+)/.test(pattern)) {
        pattern = pattern.replace(RegExp.$1, (this.getFullYear() + "")
            .substr(4 - RegExp.$1.length));
    }

    for (var k in o) {
        if (new RegExp("(" + k + ")").test(pattern)) {
            pattern = pattern.replace(RegExp.$1, RegExp.$1.length == 1 ? o[k] :
                ("00" + o[k]).substr(("" + o[k]).length));
        }
    }
    return pattern;
}

$(function () {
    if ($(".navbar").length > 0) {
        initTopBar();
    }
});

/**
 * 退出登录
 */
function logout() {
    $.ajaxByData("/api/login/logout").success(() => {
        localStorage.removeItem("name")
        localStorage.removeItem("isAdmin")
        window.location.href = "/login.html"
    }).delete()
}

/**
 * 获取url参数
 * @param key 参数名
 * @param def 参数不存在时的默认值
 * @returns {string}
 */
function getParam(key, def = "") {

    // 获取当前页面的 URL
    const urlParams = new URLSearchParams(window.location.search);

    // 获取单个参数值
    const value = urlParams.get(key);
    if (value == null) {
        return def
    }
    return value
}

/**
 * HTML转义,把文件名等用户输入的内容拼接到HTML里时使用
 * @param str 要转义的字符串
 * @returns {string}
 */
function escapeHtml(str) {
    return String(str)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;")
}
