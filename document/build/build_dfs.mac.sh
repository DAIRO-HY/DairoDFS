#!/bin/bash

#遇到错误立即停止
set -e

# 进入项目根目录
cd "$(dirname "$0")/../.."

#先创建文件夹
mkdir -p ./build

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./DairoDFS/Cargo.toml)

#安装两个平台的编译目标
rustup target add x86_64-apple-darwin aarch64-apple-darwin

#编译到macOS intel平台
cargo build --release --package DairoDFS --target x86_64-apple-darwin
cp ./target/x86_64-apple-darwin/release/DairoDFS ./build/DairoDFS-mac-darwin-amd64-$version


#编译到macOS M平台
cargo build --release --package DairoDFS --target aarch64-apple-darwin
cp ./target/aarch64-apple-darwin/release/DairoDFS ./build/DairoDFS-mac-darwin-aarch64-$version
