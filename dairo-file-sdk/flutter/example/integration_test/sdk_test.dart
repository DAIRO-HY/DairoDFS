import 'dart:async';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:dairo_file_sdk/dairo_file_sdk.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

/// 访问电脑上的DFS,参数通过--dart-define传入:DFS_SERVER、DFS_TOKEN、FILE_ID、FILE_MD5
/// flutter test integration_test -d macos --dart-define=DFS_SERVER=http://127.0.0.1:8031 ...
const server = String.fromEnvironment('DFS_SERVER');
const token = String.fromEnvironment('DFS_TOKEN');
const fileId = String.fromEnvironment('FILE_ID');
const fileMd5 = String.fromEnvironment('FILE_MD5');

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  late FileSdk sdk;
  final events = StreamController<Event>.broadcast();
  const url = '$server/api/files/download/$fileId/test.bin';
  const options = Options(headers: {'Authorization': 'Bearer $token'}, name: 'test.bin');

  setUpAll(() async {
    await RustLib.init();
    final root = Directory('${Directory.systemTemp.path}/dairo-file-sdk-test-${DateTime.now().microsecondsSinceEpoch}');
    sdk = FileSdk(config: SdkConfig(rootDir: root.path));
    sdk.events().listen(events.add);
    await sdk.setDfsAccount(account: 'test', server: server, token: token);
  });

  String md5Of(List<int> data) => md5.convert(data).toString();

  /// 等待上传结束
  Future<UploadState> waitUpload(String id) async {
    final event = await events.stream
        .where((it) => it is Event_UploadStateChanged && it.id == id && it.state != UploadState.waiting && it.state != UploadState.uploading)
        .first
        .timeout(const Duration(seconds: 30));
    return (event as Event_UploadStateChanged).state;
  }

  testWidgets('fetch and cache', (_) async {
    final path = await sdk.fetch(url: url, options: options);
    expect(md5Of(await File(path).readAsBytes()), fileMd5);
    expect(sdk.cachedPath(url: url, options: options), path);
    final info = await sdk.get_(id: sdk.cacheId(url: url, options: options));
    expect(info!.state, DownloadState.finish);
    expect(info.size, 5242880);
  });

  testWidgets('fetch with progress and cancel', (_) async {
    final task = sdk.startFetch(url: url, options: const Options(headers: {'Authorization': 'Bearer $token'}, cacheKey: 'progress-test'));
    final result = await task.events().firstWhere((it) => it is! FetchEvent_Progress).timeout(const Duration(seconds: 30));
    expect(result, isA<FetchEvent_Success>());
    expect(md5Of(await File((result as FetchEvent_Success).path).readAsBytes()), fileMd5);
    task.cancel();
  });

  testWidgets('local http with range', (_) async {
    //播放器、图片组件通过本机HTTP地址访问
    final local = sdk.open(url: url, options: const Options(headers: {'Authorization': 'Bearer $token'}, cacheKey: 'local-http-test', name: 'test.bin'));
    expect(local, startsWith('http://127.0.0.1:'));
    final client = HttpClient();
    final request = await client.getUrl(Uri.parse(local));
    request.headers.set('Range', 'bytes=1000000-1000099');
    final response = await request.close();
    expect(response.statusCode, 206);
    expect(response.headers.value('content-range'), 'bytes 1000000-1000099/5242880');
    final part = await response.fold<List<int>>([], (all, it) => all..addAll(it));
    client.close();

    final path = await sdk.fetch(url: url, options: options);
    expect(part, (await File(path).readAsBytes()).sublist(1000000, 1000100));
  });

  testWidgets('errors', (_) async {
    expect(() => sdk.open(url: 'abc', options: const Options()), throwsA(isA<SdkError_InvalidUrl>()));
    try {
      await sdk.fetch(url: url, options: const Options(cacheKey: 'no-token'));
      fail('应该失败');
    } on SdkError_UrlExpired catch (e) {
      expect(e.status, 401);
    }
  });

  testWidgets('upload file', (_) async {
    final data = List<int>.generate(3000000, (i) => i * 7 % 251);
    final file = File('${Directory.systemTemp.path}/upload-${DateTime.now().microsecondsSinceEpoch}.bin');
    await file.writeAsBytes(data);
    final ids = await sdk.addUploads(items: [UploadItem(path: file.path, dfsPath: '/sdk-test/flutter-${DateTime.now().millisecondsSinceEpoch}.bin')]);
    expect(await waitUpload(ids[0]), UploadState.finish);
    expect((await sdk.getUpload(id: ids[0]))!.md5, md5Of(data));
  });

  testWidgets('cache management', (_) async {
    await sdk.fetch(url: url, options: const Options(headers: {'Authorization': 'Bearer $token'}, cacheKey: 'music-1', group: 'music'));
    final stats = await sdk.stats(mode: CacheMode.cache);
    expect(stats.firstWhere((it) => it.group == 'music').count, 1);
    expect(await sdk.clearCache(group: 'music'), 1);
    expect((await sdk.list(query: const CacheQuery(group: 'music'))), isEmpty);
  });
}
