package cn.dairo.dfs.controller.app.about

import cn.dairo.dfs.controller.base.AppBase
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.RequestMapping

/**
 * 关于
 */
@Controller
@RequestMapping("/app/about")
class AboutAppController : AppBase() {

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute() = "app/about"
}
