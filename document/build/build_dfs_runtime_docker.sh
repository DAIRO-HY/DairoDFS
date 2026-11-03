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

#builder容器和构建过程中的RUN命令都不会继承docker的代理设置(比如Docker Desktop通过代理访问镜像仓库),需要手动传入
#否则拉取基础镜像会报network is unreachable,apt等命令也无法联网
proxy=$(docker info --format '{{.HTTPSProxy}}')
proxy_opts=""
proxy_args=""
if [ -n "$proxy" ]; then
  [[ "$proxy" == *://* ]] || proxy="http://$proxy"
  proxy_opts="--driver-opt env.http_proxy=$proxy --driver-opt env.https_proxy=$proxy"

  #http_proxy、https_proxy是docker预定义的构建参数,只在构建时生效,不会保存到镜像里
  proxy_args="--build-arg http_proxy=$proxy --build-arg https_proxy=$proxy"
fi

#多平台构建需要使用docker-container驱动的builder
docker buildx inspect dfs-builder >/dev/null 2>&1 || docker buildx create --name dfs-builder --driver docker-container $proxy_opts
docker buildx inspect dfs-builder --bootstrap

docker buildx build --builder dfs-builder --platform linux/amd64,linux/arm64 $proxy_args \
  -t $docker_user/dairo-dfs-runtime:latest --push ./document/build/dfs_runtime_dockerfile
docker logout

echo "---------------------------------------运行时镜像推送完成--------------------------------------"
