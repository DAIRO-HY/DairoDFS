# dairo-file-sdk

用Rust编写的文件下载缓存SDK，提供给Android、iOS（原生）和Windows、macOS、Linux（Flutter）使用。

- **下载和缓存**：同一个文件的多个请求合并成一次下载；断点续传（App重启之后也能继续）；缓存key去掉了域名，服务器切换域名之后缓存依然有效，通过命名空间区分不同的服务器。
- **本机HTTP**：播放器、图片组件可以直接使用SDK提供的`http://127.0.0.1`地址，支持Range，边下边播，拖动进度时优先下载拖动到的位置。
- **管理**：缓存分组统计和清理、按保存天数和容量上限自动清理、永久下载的暂停/继续/导出、事件回调。
- **上传到DFS**：秒传、断点续传、由App提供数据（相册等）、按账号区分任务。

下载和缓存不依赖DFS，可以用于任意HTTP资源；上传只支持DFS。

## 目录

| 目录 | 说明 |
|---|---|
| `core/` | Rust核心库（crate名`dairo-file-sdk`） |
| `ffi/` | Kotlin/Swift绑定（uniffi） |
| `platforms/android/` | Android库工程，`build.sh`编译AAR |
| `platforms/ios/` | Swift Package，`build.sh`编译xcframework |
| `flutter/` | Flutter插件（Windows、macOS、Linux），flutter_rust_bridge 2.12.0 |

## 编译

```bash
./dairo-file-sdk/build.sh android   # 编译Android的AAR
./dairo-file-sdk/build.sh ios       # 编译iOS/macOS的Swift Package(只能在Mac上编译)
./dairo-file-sdk/build.sh flutter   # 重新生成Flutter插件的Dart代码并打包
./dairo-file-sdk/build.sh all       # 编译Android和iOS(默认)
```

脚本会先检查编译环境，缺少的Rust编译目标会自动安装，其他缺少的工具会提示安装方法：

| 平台 | 需要 |
|---|---|
| 共同 | Rust（https://rustup.rs） |
| Android | JDK 17及以上、Android SDK（环境变量`ANDROID_HOME`，默认`~/Library/Android/sdk`）、Android NDK（不指定`ANDROID_NDK_HOME`时使用最新版本）、`cargo install cargo-ndk` |
| iOS | Mac、Xcode |
| Flutter | Flutter（Dart 3.4及以上），`flutter_rust_bridge_codegen` 2.12.0（没有安装时脚本会自动安装） |

编译好的文件在`dairo-file-sdk/dist`目录：

| 文件 | 说明 |
|---|---|
| `android/dairo-file-sdk-{版本}.aar` | Android库 |
| `ios/DairoFileSDK-{版本}.zip` | 完整的Swift Package，解压之后在Xcode里通过"Add Package Dependencies → Add Local"添加 |
| `ios/DairoFileSDKFFI-{版本}.xcframework.zip` | 只有静态库，发布到网上时`Package.swift`可以改成`.binaryTarget(name:url:checksum:)`引用 |
| `ios/DairoFileSDKFFI-{版本}.xcframework.zip.checksum` | 上面压缩包的checksum |
| `flutter/dairo_file_sdk-{版本}.zip` | Flutter插件（包含核心库的源码，可以单独使用），解压之后在`pubspec.yaml`里用`path`引用 |

版本号取自`ffi/Cargo.toml`。第一次编译需要十几分钟，之后有缓存会快很多。

## Android

### 接入

AAR依赖JNA和kotlinx-coroutines：

```kotlin
dependencies {
    implementation(files("libs/dairo-file-sdk-0.1.0.aar"))
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3")
}
```

播放器访问SDK的本机HTTP服务（`http://127.0.0.1`）需要允许明文HTTP，`res/xml/network_security_config.xml`：

```xml
<network-security-config>
    <domain-config cleartextTrafficPermitted="true">
        <domain includeSubdomains="false">127.0.0.1</domain>
    </domain-config>
</network-security-config>
```

并在`AndroidManifest.xml`的`application`上加`android:networkSecurityConfig="@xml/network_security_config"`。

### 使用

```kotlin
//整个App共用一个实例,数据目录建议用noBackupFilesDir(不会被系统清理,也不会备份)
val sdk = FileSdk(SdkConfig(rootDir = File(context.noBackupFilesDir, "dairo-file-sdk").absolutePath))

//下载并缓存,返回本地文件路径
val path = sdk.fetchAsync(url, Options(namespace = clusterId, headers = mapOf("Authorization" to "Bearer $token")))

//交给播放器
val localUrl = sdk.open(url, Options(group = "music"))
```

## iOS

开发时也可以直接在Xcode里通过"Add Local"添加`dairo-file-sdk/platforms/ios`目录（需要先编译一次）。

### 接入

播放器访问SDK的本机HTTP服务需要在`Info.plist`里允许本机网络：

```xml
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsLocalNetworking</key>
    <true/>
</dict>
```

数据目录建议用`Application Support`，并排除iCloud备份：

```swift
var root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
    .appendingPathComponent("dairo-file-sdk")
try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
var values = URLResourceValues()
values.isExcludedFromBackup = true
try root.setResourceValues(values)

let sdk = try FileSdk(config: SdkConfig(rootDir: root.path))
```

App从后台回到前台时调用`sdk.checkServer()`，本机HTTP服务失效时会重新启动（优先使用原来的端口，之前的地址依然有效）。

## Flutter（Windows、macOS、Linux）

插件里的Rust代码在App编译时自动编译（cargokit），编译App的电脑需要安装Rust（https://rustup.rs），Windows还需要Visual Studio的C++工具。

```yaml
dependencies:
  dairo_file_sdk:
    path: ../DairoDFS/dairo-file-sdk/flutter
```

```dart
import 'package:dairo_file_sdk/dairo_file_sdk.dart';

await RustLib.init();   //使用之前调用一次
final dir = await getApplicationSupportDirectory();   //path_provider
final sdk = FileSdk(config: SdkConfig(rootDir: '${dir.path}/dairo-file-sdk'));

//下载并缓存
final path = await sdk.fetch(url: url, options: Options(namespace: clusterId, headers: {'Authorization': 'Bearer $token'}));

//带进度下载,可以取消
final task = sdk.startFetch(url: url, options: const Options());
task.events().listen((event) { /* FetchEvent_Progress、FetchEvent_Success、FetchEvent_Error */ });

//事件(下载、上传的状态和进度)
sdk.events().listen((event) {});

//列表里的缩略图(同步方法,可以在build里使用)
Image.network(sdk.localUrl(url: thumbUrl, options: const Options(group: 'thumb')));
```

和Android、iOS的区别：上传ID使用字符串；上传只支持本地文件路径。

macOS的App默认开启了沙盒，需要在`macos/Runner/DebugProfile.entitlements`和`Release.entitlements`里允许网络访问（client）和本机HTTP服务（server）：

```xml
<key>com.apple.security.network.client</key>
<true/>
<key>com.apple.security.network.server</key>
<true/>
```

修改了`flutter/rust/src/api`下的Rust接口之后，执行`./dairo-file-sdk/build.sh flutter`重新生成Dart代码。

## 注意

- `delete`、`deleteUploads`、`export`等方法会读写文件或者等待任务停止，不要在主线程调用。
- 回调（`FetchListener`、`EventListener`、`UploadSourceProvider`）在SDK的后台线程里调用，更新界面时要切换到主线程。
