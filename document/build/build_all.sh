#!/bin/bash

#遇到错误立即停止
set -e

# 进入项目根目录
cd "$(dirname "$0")/../.."

./document/build/build_dfs.sh

#Windows、macOS版:在Linux上用cargo-zigbuild交叉编译,没有macOS SDK时跳过macOS版
./document/build/build_dfs_cross.sh
./document/build/build_github.sh
./document/build/build_dfs_docker.sh
