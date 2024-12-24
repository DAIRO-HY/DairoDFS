package cn.dairo.dfs.controller.app.advanced

import cn.dairo.dfs.config.Constant
import cn.dairo.dfs.controller.base.AppBase
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody
import java.io.PrintStream

/**
 * 数据同步状态
 */
@Controller
@RequestMapping("/app/advanced")
class AdvancedAppController : AppBase() {

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute() = "app/advanced"

    /**
     * 页面数据初始化
     */
    @PostMapping("/exec_sql")
    @ResponseBody
    fun execSql(sql: String): Any {
        Constant.dbService.use {
            if (sql.trim().lowercase().startsWith("select")) {//如果是查询语句

                //限制返回条数
                return it.selectList(sql).take(10000)
            } else {
                return it.exec(sql)
            }
        }
    }
}
