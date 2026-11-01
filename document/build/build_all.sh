#!/bin/bash

#遇到错误立即停止
set -e

# 进入项目根目录
cd "$(dirname "$0")/../.."

./document/build/build_dfs.sh
./document/build/build_github.sh
./document/build/build_dfs_docker.sh
