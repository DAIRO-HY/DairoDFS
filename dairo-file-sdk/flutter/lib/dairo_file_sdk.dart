/// 文件下载缓存SDK(Rust),支持Windows、macOS、Linux
///
/// 使用之前先调用一次[RustLib.init]:
/// ```dart
/// await RustLib.init();
/// final sdk = FileSdk(config: SdkConfig(rootDir: dir));
/// ```
library dairo_file_sdk;

export 'src/rust/api/sdk.dart';
export 'src/rust/api/types.dart';
export 'src/rust/frb_generated.dart' show RustLib;
