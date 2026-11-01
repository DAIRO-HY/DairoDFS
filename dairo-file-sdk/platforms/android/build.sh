#!/bin/bash
# 编译Android的AAR
# 1. 用cargo-ndk编译各个ABI的动态库(arm64-v8a、armeabi-v7a、x86_64)
# 2. 用uniffi生成Kotlin代码
# 3. 用Gradle打包成AAR,输出到 platforms/android/build/dairo-file-sdk-{版本}.aar
# 需要:Android NDK、cargo-ndk(cargo install cargo-ndk)、rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
set -e

android_dir=$(cd "$(dirname "$0")" && pwd)
project_root=$(cd "$android_dir/../../.." && pwd)
cd "$project_root"

version=$(sed -n 's/^version = "\(.*\)"/\1/p' dairo-file-sdk/ffi/Cargo.toml)
jni_dir="$android_dir/sdk/build/rustJniLibs"
kotlin_dir="$android_dir/sdk/build/generated/uniffi"

echo "编译动态库..."
rm -rf "$jni_dir"
#只输出动态库(同时输出多种类型时不会做LTO),使用体积优先的sdk配置
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 --platform 24 -o "$jni_dir" rustc --profile sdk -p dairo-file-sdk-ffi --crate-type cdylib

echo "生成Kotlin代码..."
#发布版本去掉了调试符号,uniffi的元数据也被去掉了,所以用本机的调试版本生成(导出的接口相同)
rm -rf "$kotlin_dir"
cargo build -q -p dairo-file-sdk-ffi
host_lib=$(ls target/debug/libdairo_file_sdk_ffi.dylib target/debug/libdairo_file_sdk_ffi.so 2>/dev/null | head -1)
cargo run -q -p uniffi-bindgen -- generate \
  --library "$host_lib" --language kotlin --no-format --out-dir "$kotlin_dir"

echo "打包AAR..."
cd "$android_dir"

#Gradle需要知道Android SDK的位置
if [ ! -f local.properties ]; then
  sdk_dir=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}
  echo "sdk.dir=$sdk_dir" > local.properties
fi
./gradlew -q :sdk:assembleRelease
mkdir -p build
cp sdk/build/outputs/aar/sdk-release.aar "build/dairo-file-sdk-$version.aar"
echo "完成:$android_dir/build/dairo-file-sdk-$version.aar"
