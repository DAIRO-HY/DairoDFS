#!/bin/bash

#遇到错误立即停止
set -e

# 进入项目根目录
cd "$(dirname "$0")/../.."

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./DairoDFS/Cargo.toml)

mkdir -p ./build/amd64
mkdir -p ./build/arm64
cp ./build/DairoDFS-linux-amd64-$version ./build/amd64/DairoDFS
cp ./build/DairoDFS-linux-arm64-$version ./build/arm64/DairoDFS

#docker用户名
docker_user=$DOCKER_USER
docker_pwd=$DOCKER_PASSWORD

#---------------------------------------上传Docker镜像-----------------------------------------
echo "正在打包Docker镜像..."
cp ./document/build/dfs_dockerfile/Dockerfile ./

#构建时只需要build目录下的程序,避免把整个项目(包括target目录)发送给docker
printf '*\n!build/amd64\n!build/arm64\n' > .dockerignore

echo "$docker_pwd" | docker login -u $docker_user --password-stdin

#多平台构建需要使用docker-container驱动的builder
docker buildx inspect dfs-builder >/dev/null 2>&1 || docker buildx create --name dfs-builder --driver docker-container
docker buildx inspect dfs-builder --bootstrap
docker buildx build --builder dfs-builder --platform linux/amd64,linux/arm64 \
  --build-arg RUNTIME_IMAGE=$docker_user/dairo-dfs-runtime:latest \
  -t $docker_user/dairo-dfs:$version --push .
docker logout

rm Dockerfile .dockerignore

echo "---------------------------------------docker镜像推送完成--------------------------------------"
