package cn.dairo.dfs.controller.app.sync

import cn.dairo.dfs.controller.app.sync.form.SyncServerForm
import cn.dairo.dfs.extension.toJson
import cn.dairo.dfs.sync.bean.SyncServerInfo
import org.springframework.context.annotation.Configuration
import org.springframework.web.socket.CloseStatus
import org.springframework.web.socket.TextMessage
import org.springframework.web.socket.WebSocketSession
import org.springframework.web.socket.config.annotation.EnableWebSocket
import org.springframework.web.socket.config.annotation.WebSocketConfigurer
import org.springframework.web.socket.config.annotation.WebSocketHandlerRegistry
import org.springframework.web.socket.handler.TextWebSocketHandler

/**
 * 同步信息Socket
 */
@Configuration
@EnableWebSocket
class SyncWebSocketHandler : TextWebSocketHandler(), WebSocketConfigurer {

    /**
     * 最后一次链接的session
     */
    private var session: WebSocketSession? = null

    override fun registerWebSocketHandlers(registry: WebSocketHandlerRegistry) {
        registry.addHandler(this, "/ws/app/sync")//.setAllowedOrigins("*")
    }

    /**
     * 客户端建立链接时
     */
    override fun afterConnectionEstablished(session: WebSocketSession) {
        this.session = session
    }

    override fun afterConnectionClosed(session: WebSocketSession, status: CloseStatus) {
        this.session = null
    }

    /**
     * 发送消息
     */
    fun send(info: SyncServerInfo) = try {
        this.session?.also {
            val form = SyncServerForm()
            form.url = info.url
            form.state = info.state
            form.msg = info.msg
            form.no = info.no
            form.syncCount = info.syncCount
            form.lastHeartTime = info.lastHeartTime
            form.lastTime = info.lastTime
            it.sendMessage(TextMessage(form.toJson))
        }
    } catch (e: Exception) {
    }
}