// SDK调试页面:查看运行状态,自检、App请求ping、强制重启本机HTTP服务
// 用法:NavigationLink(destination: FileSdkDebugView(sdk: sdk)) { Text("SDK调试") }

#if os(iOS)
import SwiftUI
import UIKit

public struct FileSdkDebugView: View {
    private let sdk: FileSdk

    @State private var info: DebugInfo?
    @State private var groups: [GroupStats] = []
    @State private var groupsTime = ""
    @State private var loadingGroups = false
    @State private var selfCheckText = ""
    @State private var pingText = ""
    @State private var restartText = ""
    @State private var checking = false
    @State private var pinging = false

    /// 页面显示时每秒刷新一次(缓存统计要查数据库,只在打开页面和点击刷新时查询)
    private let timer = Timer.publish(every: 1, on: .main, in: .common).autoconnect()

    public init(sdk: FileSdk) {
        self.sdk = sdk
    }

    public var body: some View {
        List {
            Section(header: Text("操作")) {
                Button(checking ? "自检中..." : "自检(SDK内部请求ping)") { selfCheck() }
                    .disabled(checking)
                resultText(selfCheckText)
                Button(pinging ? "请求中..." : "App请求ping") { appPing() }
                    .disabled(pinging)
                resultText(pingText)
                Button("强制重启本机服务") { restart() }
                    .foregroundColor(.red)
                resultText(restartText)
                Button("复制全部信息") { UIPasteboard.general.string = report() }
            }
            if let info = info {
                Section(header: Text("本机HTTP服务")) {
                    row("状态", info.serverRunning ? "运行中" : "未运行")
                    row("端口", info.serverPort.map { String($0) } ?? "-")
                    row("本次启动", info.serverStartTime.map { "\(Self.time($0)) (\(Self.duration(since: $0)))" } ?? "-")
                    row("启动次数", "\(info.serverStartCount)")
                    row("手动重启次数", "\(info.serverRestartCount)")
                    row("当前连接数", "\(info.serverConnectionCount)")
                    row("请求数", "\(info.serverRequestCount)")
                    row("返回数据量", Self.size(info.serverBytesSent))
                    row("状态码", "2xx:\(info.serverStatus2xx) 4xx:\(info.serverStatus4xx) 5xx:\(info.serverStatus5xx) 其他:\(info.serverStatusOther)")
                    row("接收连接失败", "\(info.acceptErrorCount)")
                    if let error = info.lastAcceptError {
                        row("最后一次失败", error)
                    }
                }
                Section(header: Text("网络")) {
                    row("请求数", "\(info.netRequestCount)")
                    row("失败次数", "\(info.netErrorCount)")
                    row("下载数据量", Self.size(info.netBytesReceived))
                    row("上传数据量", Self.size(info.netBytesSent))
                    row("下载任务数", "\(info.downloadingCount)")
                    row("上传任务数", "\(info.uploadingCount)")
                }
                Section(header: Text("缓存")) {
                    Button(loadingGroups ? "查询中..." : "刷新缓存统计") { refreshGroups() }
                        .disabled(loadingGroups)
                    if !groupsTime.isEmpty {
                        Text("查询时间:\(groupsTime)").font(.caption).foregroundColor(.secondary)
                    }
                    if groups.isEmpty {
                        Text("没有缓存").foregroundColor(.secondary)
                    }
                    ForEach(groups, id: \.group) { it in
                        row(it.group, "\(it.count)个 \(Self.size(it.size))")
                    }
                }
                Section(header: Text("SDK")) {
                    row("启动时间", Self.time(info.sdkStartTime))
                    row("已运行", Self.duration(since: info.sdkStartTime))
                }
                Section(header: Text("事件(最近100条)")) {
                    if info.events.isEmpty {
                        Text("没有事件").foregroundColor(.secondary)
                    }
                    ForEach(Array(info.events.enumerated()), id: \.offset) { _, it in
                        VStack(alignment: .leading, spacing: 2) {
                            Text(Self.time(it.time)).font(.caption).foregroundColor(.secondary)
                            Text(it.msg).font(.footnote)
                        }
                    }
                }
            }
        }
        .listStyle(GroupedListStyle())
        .navigationBarTitle("SDK调试", displayMode: .inline)
        .onAppear {
            refresh()
            refreshGroups()
        }
        .onReceive(timer) { _ in refresh() }
    }

    private func row(_ title: String, _ value: String) -> some View {
        HStack(alignment: .top) {
            Text(title)
            Spacer()
            Text(value).foregroundColor(.secondary).multilineTextAlignment(.trailing)
        }
        .font(.footnote)
    }

    @ViewBuilder
    private func resultText(_ text: String) -> some View {
        if !text.isEmpty {
            Text(text).font(.footnote).foregroundColor(.secondary)
        }
    }

    private func refresh() {
        info = sdk.debugInfo()
    }

    /// 查询缓存统计,缓存多的时候查询比较慢,在后台线程查询
    private func refreshGroups() {
        loadingGroups = true
        DispatchQueue.global().async {
            let result = (try? sdk.stats(mode: nil)) ?? []
            DispatchQueue.main.async {
                groups = result
                groupsTime = Self.time(Self.now())
                loadingGroups = false
            }
        }
    }

    /// SDK自己向本机服务发送ping请求
    private func selfCheck() {
        checking = true
        selfCheckText = ""
        Task {
            let result = await sdk.selfCheck()
            var text = result.ok ? "正常" : "失败"
            text += ",端口:\(result.port.map { String($0) } ?? "无")"
            text += ",TCP:\(result.connected ? "已连接" : "未连接")"
            text += ",状态码:\(result.status.map { String($0) } ?? "无")"
            text += ",耗时:\(result.elapsedMs)ms"
            if let error = result.error {
                text += "\n\(error)"
            }
            await MainActor.run {
                selfCheckText = "\(Self.time(Self.now())) \(text)"
                checking = false
                refresh()
            }
        }
    }

    /// App用自己的URLSession请求本机服务的ping,和App加载图片的方式一样
    private func appPing() {
        pinging = true
        pingText = ""
        let start = Date()
        let finish: (String) -> Void = { text in
            let elapsed = Int(Date().timeIntervalSince(start) * 1000)
            DispatchQueue.main.async {
                pingText = "\(Self.time(Self.now())) \(text),耗时:\(elapsed)ms"
                pinging = false
                refresh()
            }
        }
        let base: String
        do {
            //localBase()在服务没有运行时会启动服务
            base = try sdk.localBase()
        } catch {
            finish("获取地址失败:\(error)")
            return
        }
        guard let url = URL(string: base + "/ping") else {
            finish("地址不正确")
            return
        }
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 3
        config.timeoutIntervalForResource = 3
        let session = URLSession(configuration: config)
        session.dataTask(with: url) { data, response, error in
            let port = url.port.map { String($0) } ?? "-"
            if let error = error as NSError? {
                let reason = error.code == NSURLErrorTimedOut ? "超时" : error.localizedDescription
                finish("失败,端口:\(port),\(reason)(\(error.code))")
            } else if let response = response as? HTTPURLResponse {
                let body = data.flatMap { String(data: $0, encoding: .utf8) } ?? ""
                finish("\(response.statusCode == 200 ? "正常" : "失败"),端口:\(port),状态码:\(response.statusCode)\n\(body)")
            } else {
                finish("失败,端口:\(port),没有响应")
            }
            session.finishTasksAndInvalidate()
        }.resume()
    }

    private func restart() {
        let old = sdk.debugInfo().serverPort
        do {
            let port = try sdk.restartServer()
            let oldText = old.map { String($0) } ?? "未运行"
            restartText = "\(Self.time(Self.now())) 重启完成,端口:\(port)(原端口:\(oldText))"
        } catch {
            restartText = "\(Self.time(Self.now())) 重启失败:\(error)"
        }
        refresh()
    }

    /// 全部信息,复制给开发者分析
    private func report() -> String {
        let info = sdk.debugInfo()
        var lines = [
            "SDK启动:\(Self.time(info.sdkStartTime)) 已运行:\(Self.duration(since: info.sdkStartTime))",
            "服务:\(info.serverRunning ? "运行中" : "未运行") 端口:\(info.serverPort.map { String($0) } ?? "-") 本次启动:\(info.serverStartTime.map { Self.time($0) } ?? "-")",
            "启动次数:\(info.serverStartCount) 手动重启:\(info.serverRestartCount) 当前连接:\(info.serverConnectionCount)",
            "请求数:\(info.serverRequestCount) 返回:\(Self.size(info.serverBytesSent)) 2xx:\(info.serverStatus2xx) 4xx:\(info.serverStatus4xx) 5xx:\(info.serverStatus5xx) 其他:\(info.serverStatusOther)",
            "接收连接失败:\(info.acceptErrorCount) \(info.lastAcceptError ?? "")",
            "网络请求:\(info.netRequestCount) 失败:\(info.netErrorCount) 下载:\(Self.size(info.netBytesReceived)) 上传:\(Self.size(info.netBytesSent)) 下载任务:\(info.downloadingCount) 上传任务:\(info.uploadingCount)",
            "自检:\(selfCheckText)",
            "App ping:\(pingText)",
            "重启:\(restartText)",
            "事件:",
        ]
        lines += info.events.map { "\(Self.time($0.time)) \($0.msg)" }
        return lines.joined(separator: "\n")
    }

    private static func now() -> Int64 {
        Int64(Date().timeIntervalSince1970 * 1000)
    }

    private static let timeFormatter: DateFormatter = {
        let it = DateFormatter()
        it.dateFormat = "MM-dd HH:mm:ss"
        return it
    }()

    private static func time(_ millis: Int64) -> String {
        timeFormatter.string(from: Date(timeIntervalSince1970: TimeInterval(millis) / 1000))
    }

    private static func duration(since millis: Int64) -> String {
        let seconds = max(0, (now() - millis) / 1000)
        let (d, h, m, s) = (seconds / 86400, seconds % 86400 / 3600, seconds % 3600 / 60, seconds % 60)
        if d > 0 {
            return "\(d)天\(h)小时\(m)分"
        }
        if h > 0 {
            return "\(h)小时\(m)分\(s)秒"
        }
        return m > 0 ? "\(m)分\(s)秒" : "\(s)秒"
    }

    private static func size(_ bytes: UInt64) -> String {
        ByteCountFormatter.string(fromByteCount: Int64(bytes), countStyle: .binary)
    }
}
#endif
