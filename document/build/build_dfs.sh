#!/bin/bash

#遇到错误立即停止
set -e

#前提条件 安装 QEMU binfmt,目的:在amd64位linux系统上编译arm64位linux二进制文件
docker run --privileged --rm tonistiigi/binfmt --install all

# 当前 sh 文件所在目录
script_dir="$(cd "$(dirname "$0")" && pwd)"

# 项目目录
project_root="$(cd "$script_dir/../.." && pwd)"

#先创建文件夹
mkdir -p $project_root/build

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' $project_root/DairoDFS/Cargo.toml)

#挂载到编译容器的目录一定是宿主机上的绝对路径
#在Jenkins容器里调用docker时,当前目录是Jenkins容器里的路径,需要通过环境变量指定宿主机上的路径
host_project_dir=${HOST_PROJECT_DIR:-$project_root}
host_cargo_dir=${HOST_CARGO_DIR:-$HOME/.cargo}

docker_rust_build(){
  #不同架构使用不同的编译目录,避免每次都要重新编译
  docker run --platform "linux/$1" --rm \
    -v $host_project_dir:/home/rust/src \
    -v $host_cargo_dir/registry:/usr/local/cargo/registry \
    -e CARGO_TARGET_DIR=/home/rust/src/target/linux-$1 \
    -w /home/rust/src rust:1.99.1 \
    sh -c "cargo build --release --package DairoDFS"
}

#打包amd64位linux二进制文件
docker_rust_build amd64
cp $project_root/target/linux-amd64/release/DairoDFS $project_root/build/DairoDFS-linux-amd64-$version


#打包arm64位linux二进制文件
docker_rust_build arm64
cp $project_root/target/linux-arm64/release/DairoDFS $project_root/build/DairoDFS-linux-arm64-$version
