#!/bin/bash
# 编译iOS/macOS的Swift Package
# 1. 编译静态库:iOS真机(arm64)、iOS模拟器(arm64+x86_64)、macOS(arm64+x86_64)
# 2. 用uniffi生成Swift代码
# 3. 打包成DairoFileSDKFFI.xcframework,和生成的Swift代码一起组成Swift Package(Package.swift)
# 需要:Xcode、rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin
set -e

ios_dir=$(cd "$(dirname "$0")" && pwd)
project_root=$(cd "$ios_dir/../../.." && pwd)
cd "$project_root"

lib=libdairo_file_sdk_ffi.a
work="$ios_dir/build"
rm -rf "$work"
mkdir -p "$work"

export IPHONEOS_DEPLOYMENT_TARGET=13.0
export MACOSX_DEPLOYMENT_TARGET=10.15
for target in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin; do
  echo "编译 $target ..."
  #只输出静态库(同时输出多种类型时不会做LTO),使用体积优先的sdk配置
  cargo rustc --profile sdk -p dairo-file-sdk-ffi --target $target --crate-type staticlib
done

echo "合并模拟器和macOS的静态库..."
mkdir -p "$work/ios" "$work/ios-sim" "$work/macos"
cp "target/aarch64-apple-ios/sdk/$lib" "$work/ios/$lib"
lipo -create "target/aarch64-apple-ios-sim/sdk/$lib" "target/x86_64-apple-ios/sdk/$lib" -output "$work/ios-sim/$lib"
lipo -create "target/aarch64-apple-darwin/sdk/$lib" "target/x86_64-apple-darwin/sdk/$lib" -output "$work/macos/$lib"

echo "生成Swift代码..."
#发布版本去掉了调试符号,uniffi的元数据也被去掉了,所以用本机的调试版本生成(导出的接口相同)
cargo build -q -p dairo-file-sdk-ffi
cargo run -q -p uniffi-bindgen -- generate \
  --library target/debug/libdairo_file_sdk_ffi.dylib --language swift --out-dir "$work/swift"
mkdir -p "$work/headers"
cp "$work/swift/DairoFileSDKFFI.h" "$work/headers/"
cp "$work/swift/DairoFileSDKFFI.modulemap" "$work/headers/module.modulemap"

echo "打包xcframework..."
rm -rf "$ios_dir/DairoFileSDKFFI.xcframework"
xcodebuild -create-xcframework \
  -library "$work/ios/$lib" -headers "$work/headers" \
  -library "$work/ios-sim/$lib" -headers "$work/headers" \
  -library "$work/macos/$lib" -headers "$work/headers" \
  -output "$ios_dir/DairoFileSDKFFI.xcframework"
cp "$work/swift/DairoFileSDK.swift" "$ios_dir/Sources/DairoFileSDK/DairoFileSDK.swift"
echo "完成:$ios_dir(Swift Package)"
