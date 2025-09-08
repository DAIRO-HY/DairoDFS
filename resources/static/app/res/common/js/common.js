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
    $.ajaxByData("/app/login/logout").success(() => {
        window.location.href = "/app/login"
    }).post()
}

/**
 * 重置账户
 */
function reinit() {
    $.ajaxByData("/app/index/reinit").success(() => {
        window.location.href = "/app/login"
    }).post()
}

/**
 * 获取url参数
 * @param key
 * @returns {string}
 */
function getParam(key) {

    // 获取当前页面的 URL
    const urlParams = new URLSearchParams(window.location.search);

    // 获取单个参数值
    const value = urlParams.get(key);
    if (value == null) {
        return ""
    }
    return value
}

function getCookie(name) {
    // 将 cookie 字符串拆分为数组
    const cookieArray = document.cookie.split('; ');

    // 遍历数组查找指定名称的 cookie
    for (let i = 0; i < cookieArray.length; i++) {
        const cookie = cookieArray[i];
        const [cookieName, cookieValue] = cookie.split('=');

        // 如果找到匹配的 cookie 名称，返回其值
        if (cookieName === name) {
            return decodeURIComponent(cookieValue);
        }
    }

    // 如果没有找到，返回 null
    return null;
}