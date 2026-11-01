#!/bin/bash

#开发工具安装目录
develop_tool_dir=$(echo ~)/develop_tool

#先创建文件夹
mkdir $develop_tool_dir

apt update
yes|apt install build-essential

#--------------------------------------安装CARGO--------------------------------------START# 指定安装目录
export RUSTUP_HOME=$develop_tool_dir/.rustup
export CARGO_HOME=$develop_tool_dir/.cargo

# 加入PATH
export PATH=${CARGO_HOME}/bin:${PATH}
curl https://sh.rustup.rs -sSf | sh -s -- -y     --default-toolchain 1.99.1

cargo --version
#--------------------------------------安装CARGO--------------------------------------START
