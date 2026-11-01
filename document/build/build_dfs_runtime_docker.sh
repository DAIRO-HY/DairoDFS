#!/bin/bash

#运行时镜像:编译安装LibRaw、ImageMagick、ExifTool,很少变化,只在需要更新这些工具时执行
#在amd64机器上通过QEMU模拟编译arm64镜像,耗时会比较长

#遇到错误立即停止
set -e

# 进入项目根目录
cd "$(dirname "$0")/../.."

#docker用户名
docker_user=$DOCKER_USER
docker_pwd=$DOCKER_PASSWORD

#前提条件 安装 QEMU binfmt,目的:在amd64位linux系统上构建arm64位镜像
docker run --privileged --rm tonistiigi/binfmt --install all

echo "$docker_pwd" | docker login -u $docker_user --password-stdin

#多平台构建需要使用docker-container驱动的builder
docker buildx inspect dfs-builder >/dev/null 2>&1 || docker buildx create --name dfs-builder --driver docker-container
docker buildx inspect dfs-builder --bootstrap

docker buildx build --builder dfs-builder --platform linux/amd64,linux/arm64 \
  -t $docker_user/dairo-dfs-runtime:latest --push ./document/build/dfs_runtime_dockerfile
docker logout

echo "---------------------------------------运行时镜像推送完成--------------------------------------"
