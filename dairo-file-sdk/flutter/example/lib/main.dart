import 'dart:io';

import 'package:dairo_file_sdk/dairo_file_sdk.dart';
import 'package:flutter/material.dart';

/// 示例:输入下载地址,显示下载进度和本机HTTP地址
Future<void> main() async {
  await RustLib.init();
  final root = Directory('${Directory.systemTemp.path}/dairo-file-sdk-example');
  final sdk = FileSdk(config: SdkConfig(rootDir: root.path));
  runApp(MaterialApp(home: DemoPage(sdk: sdk)));
}

class DemoPage extends StatefulWidget {
  const DemoPage({super.key, required this.sdk});

  final FileSdk sdk;

  @override
  State<DemoPage> createState() => _DemoPageState();
}

class _DemoPageState extends State<DemoPage> {
  final _url = TextEditingController();
  FetchTask? _task;
  String _message = '';

  void _download() {
    _task?.cancel();
    final options = Options(name: Uri.tryParse(_url.text)?.pathSegments.lastOrNull);
    final task = widget.sdk.startFetch(url: _url.text, options: options);
    _task = task;
    task.events().listen((event) {
      setState(() {
        _message = switch (event) {
          FetchEvent_Progress(:final size, :final downloaded) => '下载中:$downloaded/${size ?? '未知'}',
          FetchEvent_Success(:final path) => '完成:$path',
          FetchEvent_Error(:final error) => '失败:$error',
        };
      });
    });
  }

  void _openLocal() {
    try {
      final local = widget.sdk.open(url: _url.text, options: const Options());
      setState(() => _message = '本机地址:$local');
    } on SdkError catch (e) {
      setState(() => _message = '失败:$e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('dairo_file_sdk')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            TextField(controller: _url, decoration: const InputDecoration(labelText: '下载地址')),
            const SizedBox(height: 16),
            Wrap(spacing: 8, children: [
              ElevatedButton(onPressed: _download, child: const Text('下载')),
              ElevatedButton(onPressed: () => _task?.cancel(), child: const Text('取消')),
              ElevatedButton(onPressed: _openLocal, child: const Text('本机地址')),
            ]),
            const SizedBox(height: 16),
            SelectableText(_message),
          ],
        ),
      ),
    );
  }
}
