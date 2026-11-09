import XCTest
@testable import DairoFileSDK

/// 调试接口:不需要DFS
final class DebugTests: XCTestCase {

    func makeSdk() throws -> FileSdk {
        let root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("dairo-file-sdk-debug-test-\(UUID().uuidString)")
        return try FileSdk(config: SdkConfig(rootDir: root.path))
    }

    /// App用URLSession请求ping
    func ping(_ base: String) async throws -> (Int, [String: Any]) {
        let (data, response) = try await URLSession.shared.data(from: URL(string: base + "/ping")!)
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]
        return ((response as! HTTPURLResponse).statusCode, json)
    }

    func testPingSelfCheckRestart() async throws {
        let sdk = try makeSdk()
        let base = try sdk.localBase()
        let port = UInt16(URL(string: base)!.port!)

        let (status, json) = try await ping(base)
        XCTAssertEqual(status, 200)
        XCTAssertEqual(json["ok"] as? Bool, true)
        XCTAssertEqual(json["port"] as? Int, Int(port))

        let result = await sdk.selfCheck()
        XCTAssertTrue(result.ok, "\(result)")
        XCTAssertEqual(result.port, port)
        XCTAssertEqual(result.status, 200)

        XCTAssertEqual(try sdk.restartServer(), port)
        let (status2, _) = try await ping(base)
        XCTAssertEqual(status2, 200)

        let info = sdk.debugInfo()
        XCTAssertTrue(info.serverRunning)
        XCTAssertEqual(info.serverPort, port)
        XCTAssertEqual(info.serverRestartCount, 1)
        XCTAssertEqual(info.serverStartCount, 2)
        XCTAssertGreaterThanOrEqual(info.serverRequestCount, 3)
        XCTAssertGreaterThanOrEqual(info.serverStatus2xx, 3)
        XCTAssertGreaterThan(info.serverBytesSent, 0)
        XCTAssertTrue(info.events.contains { $0.msg.hasPrefix("停止:手动重启") }, "\(info.events)")
        XCTAssertTrue(info.events.contains { $0.msg.hasPrefix("自检正常") }, "\(info.events)")

        //停止之后自检失败,不会自动启动
        sdk.stopServer()
        let stopped = await sdk.selfCheck()
        XCTAssertFalse(stopped.ok)
        XCTAssertFalse(stopped.running)
        XCTAssertEqual(stopped.error, "服务没有运行")
        XCTAssertFalse(sdk.debugInfo().serverRunning)
    }

    func testLocalSource() throws {
        let sdk = try makeSdk()
        let options = Options(group: "image")
        guard case .http(let url) = try sdk.localSource(url: "http://example.com/a.jpg", options: options) else {
            return XCTFail("没有缓存时应该返回本机地址")
        }
        XCTAssertTrue(url.hasPrefix("http://127.0.0.1:"))
        let path = try sdk.putData(url: "http://example.com/a.jpg", options: options, data: Data([1, 2, 3]))
        guard case .file(let file) = try sdk.localSource(url: "http://example.com/a.jpg", options: options) else {
            return XCTFail("已经缓存时应该返回文件路径")
        }
        XCTAssertEqual(file, path)
        XCTAssertEqual(try Data(contentsOf: URL(fileURLWithPath: file)), Data([1, 2, 3]))
    }
}
