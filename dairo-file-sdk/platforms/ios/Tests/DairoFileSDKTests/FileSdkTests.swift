import CryptoKit
import XCTest
@testable import DairoFileSDK

/// 访问电脑上的DFS,通过环境变量传入参数(在模拟器上运行时环境变量要加上TEST_RUNNER_前缀)
/// DFS_SERVER(例如 http://127.0.0.1:8031)、DFS_TOKEN、FILE_ID、FILE_MD5
final class FileSdkTests: XCTestCase {

    static let env = ProcessInfo.processInfo.environment
    static let server = env["DFS_SERVER"] ?? ""
    static let token = env["DFS_TOKEN"] ?? ""
    static let fileId = env["FILE_ID"] ?? ""
    static let fileMd5 = env["FILE_MD5"] ?? ""

    /// 收到的事件
    static let events = EventCollector()
    static let sdk: FileSdk = {
        let root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("dairo-file-sdk-test-\(UUID().uuidString)")
        let sdk = try! FileSdk(config: SdkConfig(rootDir: root.path))
        sdk.setListener(listener: events)
        sdk.setDfsAccount(account: "test", server: server, token: token)
        return sdk
    }()

    var sdk: FileSdk { Self.sdk }
    var url: String { "\(Self.server)/api/files/download/\(Self.fileId)/test.bin" }
    var options: Options { Options(headers: ["Authorization": "Bearer \(Self.token)"], name: "test.bin") }

    override func setUpWithError() throws {
        try XCTSkipIf(Self.server.isEmpty, "没有设置DFS_SERVER")
    }

    func md5(_ data: Data) -> String {
        Insecure.MD5.hash(data: data).map { String(format: "%02x", $0) }.joined()
    }

    func testFetchAsyncAndCache() async throws {
        let path = try await sdk.fetchAsync(url: url, options: options)
        XCTAssertEqual(md5(try Data(contentsOf: URL(fileURLWithPath: path))), Self.fileMd5)
        XCTAssertEqual(try sdk.cachedPath(url: url, options: options), path)
        let info = try sdk.get(id: try sdk.cacheId(url: url, options: options))!
        XCTAssertEqual(info.state, .finish)
        XCTAssertEqual(info.size, 5242880)
    }

    func testFetchWithCallback() throws {
        let done = expectation(description: "下载完成")
        let listener = CallbackListener { result in
            if case .success(let path) = result,
               let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
               self.md5(data) == Self.fileMd5 {
                done.fulfill()
            }
        }
        var other = options
        other.cacheKey = "callback-test"
        _ = sdk.fetch(url: url, options: other, listener: listener)
        wait(for: [done], timeout: 30)
    }

    func testLocalHttpWithRange() async throws {
        //播放器、图片组件通过本机HTTP地址访问
        var other = options
        other.cacheKey = "local-http-test"
        let local = try sdk.open(url: url, options: other)
        XCTAssertTrue(local.hasPrefix("http://127.0.0.1:"))
        var request = URLRequest(url: URL(string: local)!)
        request.setValue("bytes=1000000-1000099", forHTTPHeaderField: "Range")
        let (part, response) = try await URLSession.shared.data(for: request)
        let http = response as! HTTPURLResponse
        XCTAssertEqual(http.statusCode, 206)
        XCTAssertEqual(http.value(forHTTPHeaderField: "Content-Range"), "bytes 1000000-1000099/5242880")

        let path = try await sdk.fetchAsync(url: url, options: options)
        let data = try Data(contentsOf: URL(fileURLWithPath: path))
        XCTAssertEqual(part, data.subdata(in: 1000000..<1000100))
    }

    func testErrors() async throws {
        XCTAssertThrowsError(try sdk.open(url: "abc", options: Options())) { error in
            guard case SdkError.InvalidUrl = error else {
                return XCTFail("\(error)")
            }
        }
        do {
            _ = try await sdk.fetchAsync(url: url, options: Options(cacheKey: "no-token"))
            XCTFail()
        } catch SdkError.UrlExpired(let status, _) {
            XCTAssertEqual(status, 401)
        }
    }

    func testUploadFile() throws {
        let data = Data((0..<3_000_000).map { UInt8($0 * 7 % 251) })
        let file = FileManager.default.temporaryDirectory.appendingPathComponent("upload.bin")
        try data.write(to: file)
        let item = UploadItem(source: .file(path: file.path), dfsPath: "/sdk-test/ios-\(Date().timeIntervalSince1970).bin")
        let id = try sdk.addUploads(items: [item])[0]
        XCTAssertEqual(Self.events.waitUploadResult(id: id), .finish)
        XCTAssertEqual(try sdk.getUpload(id: id)?.md5, md5(data))
    }

    func testUploadCustomSource() throws {
        //由App提供数据,例如相册里的照片
        let data = Data((0..<1_500_000).map { UInt8($0 * 13 % 251) })
        let provider = MemoryProvider(data: data)
        sdk.setUploadSourceProvider(provider: provider)
        let item = UploadItem(source: .custom(id: "photo-1"), dfsPath: "/sdk-test/photo-\(Date().timeIntervalSince1970).jpg")
        let id = try sdk.addUploads(items: [item])[0]
        XCTAssertEqual(Self.events.waitUploadResult(id: id), .finish)
        XCTAssertEqual(provider.closed, ["photo-1"])
    }
}

final class EventCollector: EventListener, @unchecked Sendable {
    private let lock = NSCondition()
    private var events: [Event] = []

    func onEvent(event: Event) {
        lock.lock()
        events.append(event)
        lock.broadcast()
        lock.unlock()
    }

    /// 等待上传结束(完成、跳过或者失败)
    func waitUploadResult(id: Int64) -> UploadState? {
        let deadline = Date().addingTimeInterval(30)
        lock.lock()
        defer { lock.unlock() }
        while Date() < deadline {
            for event in events {
                if case .uploadStateChanged(let it, let state, _) = event, it == id, state != .waiting, state != .uploading {
                    return state
                }
            }
            lock.wait(until: deadline)
        }
        return nil
    }
}

final class CallbackListener: FetchListener, @unchecked Sendable {
    let callback: (Result<String, SdkError>) -> Void

    init(_ callback: @escaping (Result<String, SdkError>) -> Void) {
        self.callback = callback
    }

    func onProgress(size: UInt64?, downloaded: UInt64) {}

    func onSuccess(path: String) {
        callback(.success(path))
    }

    func onError(error: SdkError) {
        callback(.failure(error))
    }
}

final class MemoryProvider: UploadSourceProvider, @unchecked Sendable {
    let data: Data
    var closed: [String] = []

    init(data: Data) {
        self.data = data
    }

    func size(id: String) throws -> UInt64 {
        UInt64(data.count)
    }

    func read(id: String, offset: UInt64, len: UInt64) throws -> Data {
        let end = min(Int(offset + len), data.count)
        return data.subdata(in: Int(offset)..<end)
    }

    func close(id: String) {
        closed.append(id)
    }
}
