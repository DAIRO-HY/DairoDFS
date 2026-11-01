#!/bin/bash
# 编译SDK,先检查编译环境,编译好的文件统一放到 dairo-file-sdk/dist 目录
#
# 用法:
#   ./dairo-file-sdk/build.sh android   编译Android的AAR
#   ./dairo-file-sdk/build.sh ios       编译iOS/macOS的Swift Package(只能在Mac上编译)
#   ./dairo-file-sdk/build.sh flutter   重新生成Flutter插件的Dart代码并打包(Rust代码在App编译时由cargokit自动编译)
#   ./dairo-file-sdk/build.sh all       编译Android和iOS(默认)
#
# 输出:
#   dist/android/dairo-file-sdk-{版本}.aar
#   dist/ios/DairoFileSDK-{版本}.zip                     Swift Package,解压之后在Xcode里通过"Add Local"添加
#   dist/ios/DairoFileSDKFFI-{版本}.xcframework.zip      只有静态库,发布到网上时Package.swift可以用url+checksum引用
#   dist/ios/DairoFileSDKFFI-{版本}.xcframework.zip.checksum
#   dist/flutter/dairo_file_sdk-{版本}.zip               Flutter插件,解压之后在pubspec.yaml里用path引用
set -e

sdk_dir=$(cd "$(dirname "$0")" && pwd)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$sdk_dir/ffi/Cargo.toml")
dist="$sdk_dir/dist"
target=${1:-all}

fail() {
  echo "错误:$1" >&2
  exit 1
}

#安装缺少的Rust编译目标
add_rust_targets() {
  local installed
  installed=$(rustup target list --installed)
  for it in "$@"; do
    if ! echo "$installed" | grep -qx "$it"; then
      echo "安装Rust编译目标 $it ..."
      rustup target add "$it"
    fi
  done
}

check_common() {
  command -v cargo >/dev/null || fail "没有安装Rust,请先安装:https://rustup.rs"
  command -v rustup >/dev/null || fail "没有找到rustup,请通过 https://rustup.rs 安装Rust"
}

check_android() {
  command -v cargo-ndk >/dev/null || fail "没有安装cargo-ndk,请执行:cargo install cargo-ndk"
  command -v java >/dev/null || fail "没有安装JDK,请安装JDK 17"
  local java_version
  java_version=$(java -version 2>&1 | sed -n 's/.*version "\([0-9]*\).*/\1/p' | head -1)
  [ "${java_version:-0}" -ge 17 ] || fail "需要JDK 17及以上,当前版本:$java_version"

  local android_home=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}
  [ -d "$android_home" ] || fail "没有找到Android SDK,请设置环境变量ANDROID_HOME"
  export ANDROID_HOME=$android_home

  #没有指定NDK时使用最新安装的版本
  if [ -z "$ANDROID_NDK_HOME" ]; then
    local ndk
    ndk=$(ls -d "$android_home"/ndk/*/ 2>/dev/null | sort -V | tail -1)
    [ -n "$ndk" ] || fail "没有安装Android NDK,请在Android Studio的SDK Manager里安装NDK"
    export ANDROID_NDK_HOME=${ndk%/}
  fi
  echo "Android SDK:$ANDROID_HOME"
  echo "Android NDK:$ANDROID_NDK_HOME"
  add_rust_targets aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
}

check_ios() {
  [ "$(uname)" = "Darwin" ] || fail "iOS只能在Mac上编译"
  command -v xcodebuild >/dev/null || fail "没有安装Xcode"
  xcodebuild -version >/dev/null 2>&1 || fail "Xcode命令行工具不可用,请执行:sudo xcode-select -s /Applications/Xcode.app"
  add_rust_targets aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin
}

#Flutter插件使用的flutter_rust_bridge版本,和flutter/rust/Cargo.toml、flutter/pubspec.yaml保持一致
frb_version=2.12.0

check_flutter() {
  command -v flutter >/dev/null || fail "没有安装Flutter"
  if [ "$(flutter_rust_bridge_codegen --version 2>/dev/null | awk '{print $2}')" != "$frb_version" ]; then
    echo "安装flutter_rust_bridge_codegen $frb_version ..."
    cargo install flutter_rust_bridge_codegen --version "$frb_version" --locked
  fi
}

build_flutter() {
  echo "========== 生成Flutter插件 =========="
  check_flutter
  local flutter_dir="$sdk_dir/flutter"
  cd "$flutter_dir"

  flutter pub get

  #example里的插件符号链接会指回插件本身,build_runner遍历时会出错,先删掉(编译example时会重新生成)
  rm -rf example/linux/flutter/ephemeral example/windows/flutter/ephemeral example/macos/Flutter/ephemeral .dart_tool/build
  flutter_rust_bridge_codegen generate
  flutter analyze lib

  #打包时把核心库一起放进去,解压之后可以单独使用(不依赖这个仓库)
  local stage="$dist/flutter/stage"
  rm -rf "$stage"
  mkdir -p "$stage/dairo_file_sdk"
  rsync -a --exclude ".dart_tool" --exclude "build" --exclude "ephemeral" --exclude "rust/target" --exclude "*.iml" --exclude ".idea" \
    ./ "$stage/dairo_file_sdk/"
  rsync -a --exclude "target" "$sdk_dir/core/" "$stage/dairo_file_sdk/rust/dairo-file-sdk-core/"
  sed -i.bak 's#path = "../../core"#path = "dairo-file-sdk-core"#' "$stage/dairo_file_sdk/rust/Cargo.toml"
  rm "$stage/dairo_file_sdk/rust/Cargo.toml.bak"
  cp "$sdk_dir/../Cargo.lock" "$stage/dairo_file_sdk/rust/Cargo.lock"

  local package="$dist/flutter/dairo_file_sdk-$version.zip"
  rm -f "$package"
  (cd "$stage" && zip -qry "$package" dairo_file_sdk)
  rm -rf "$stage"
  cd "$sdk_dir"
}

build_android() {
  echo "========== 编译Android =========="
  check_android
  "$sdk_dir/platforms/android/build.sh"
  mkdir -p "$dist/android"
  cp "$sdk_dir/platforms/android/build/dairo-file-sdk-$version.aar" "$dist/android/"
}

build_ios() {
  echo "========== 编译iOS =========="
  check_ios
  local ios_dir="$sdk_dir/platforms/ios"
  "$ios_dir/build.sh"
  mkdir -p "$dist/ios"

  #完整的Swift Package
  local package="$dist/ios/DairoFileSDK-$version.zip"
  rm -f "$package"
  (cd "$ios_dir" && zip -qry "$package" Package.swift Sources Tests DairoFileSDKFFI.xcframework)

  #只有静态库,发布到网上时Package.swift可以改成 .binaryTarget(name:url:checksum:)
  local framework="$dist/ios/DairoFileSDKFFI-$version.xcframework.zip"
  rm -f "$framework"
  (cd "$ios_dir" && zip -qry "$framework" DairoFileSDKFFI.xcframework)
  (cd "$ios_dir" && swift package compute-checksum "$framework") > "$framework.checksum"
}

check_common
case "$target" in
  android) build_android ;;
  ios) build_ios ;;
  flutter) build_flutter ;;
  all)
    build_android
    build_ios
    ;;
  *) fail "不支持的参数:$target,可以是 android、ios、flutter、all" ;;
esac

echo "========== 完成 =========="
find "$dist" -type f | sort | while read -r it; do
  echo "$(du -h "$it" | cut -f1)	${it#$sdk_dir/}"
done
