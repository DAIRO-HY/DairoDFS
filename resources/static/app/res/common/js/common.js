// 仿java
String.prototype.startWith = function (str) {
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
String.prototype.endWith = function (str) {
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

function dateFormat(date, pattern = "yyyy-MM-dd hh:mm:ss") {
    const o = {
        "M+": date.getMonth() + 1, // month
        "d+": date.getDate(), // day
        "h+": date.getHours(), // hour
        "m+": date.getMinutes(), // minute
        "s+": date.getSeconds(), // second
        "q+": Math.floor((date.getMonth() + 3) / 3), // quarter
        "S": date.getMilliseconds()
        // millisecond
    };

    if (/(y+)/.test(pattern)) {
        pattern = pattern.replace(RegExp.$1, (date.getFullYear() + "")
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