#!/bin/bash

#在编译环境容器(document/docker-build/Dockerfile)里执行:拉取代码 -> 编译 -> 发布到GitHub -> 推送Docker镜像
#运行时需要:
#  -v /var/run/docker.sock:/var/run/docker.sock  使用宿主机的docker
#  -v dairo-dfs-src:/home/DairoDFS               源码所在的命名卷
#  -e GITHUB_TOKEN -e DOCKER_USER -e DOCKER_PASSWORD
#可选:
#  -e BRANCH=release     拉取的分支
#  -e PUBLISH=false      只编译,不创建标签、不上传GitHub、不推送Docker镜像

#遇到错误立即停止
set -e

#github登录票据(不要输出到日志)
github_token=$GITHUB_TOKEN

#项目名
projectName="DairoDFS"

repo="DAIRO-HY/$projectName"
branch=${BRANCH:-release}

#源码和cargo依赖缓存都放在Docker命名卷里
#编译时会通过宿主机的docker再启动rust容器,挂载目录必须是宿主机能识别的路径,命名卷在两个容器里都能直接使用,不需要知道宿主机上的路径
#卷会一直保留,target目录里的编译结果下次可以复用,只编译有变化的部分
src_volume=${SRC_VOLUME:-dairo-dfs-src}
cargo_volume=${CARGO_VOLUME:-dairo-dfs-cargo-registry}

#--------------------------------------获取代码-----------------------------------------
cd /home/$projectName
if [ -d .git ]; then

    #取消所有更改
    git reset --hard

    #删除所有新添加的文件(target、build目录在.gitignore里,不会被删除)
    git clean -fd
    git checkout $branch
    git pull
else
    echo "正在克隆 $repo 分支 $branch ..."
    git clone --branch $branch "https://${github_token}@github.com/${repo}.git" .
fi

#---------------------------------------编译、发布-----------------------------------------
export HOST_PROJECT_DIR=$src_volume
export HOST_CARGO_REGISTRY=$cargo_volume
chmod +x document/build/*.sh
if [ "$PUBLISH" = "false" ]; then
    ./document/build/build_dfs.sh
else
    ./document/build/build_all.sh
fi

echo "---------------------------------------完成--------------------------------------"
ls -l ./build/
