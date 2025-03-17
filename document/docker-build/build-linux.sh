#!/bin/bash

#github登录票据
github_token=$GITHUB_TOKEN

#docker用户名
docker_user=$DOCKER_USER
docker_pwd=$DOCKER_PWD

#项目名
projectName="DairoDFS"

#Docker镜像名
dockerImageName="dairo-dfs"

repo="DAIRO-HY/$projectName"
branch="release"

#最终编译好的二进制文件
exec_name=dairo-dfs-linux-amd64
exec_file="./$exec_name"


#--------------------------------------配置编译环境-------------------------------------
if [ ! -d "/opt/go-1.24.1" ]; then
    cd /opt

    #下载go
    curl -L -o go1.24.1.linux-amd64.tar.gz https://go.dev/dl/go1.24.1.linux-amd64.tar.gz

    #解压
    tar -xzvf go1.24.1.linux-amd64.tar.gz

    #删除安装包
    rm go1.24.1.linux-amd64.tar.gz

    mv go go-1.24.1
else
  echo "go-1.24.1 is exists"
fi

# 追加到PATH环境变量
export PATH="/opt/go-1.24.1/bin:${PATH}"

#--------------------------------------获取代码-----------------------------------------
cd /home
if [ -d $projectName ]; then
    cd $projectName

    #删除所有新添加的文件
    git clean -f

    #取消所有更改
    git reset --hard
    git pull
else
    CLONE_URL="https://${github_token}@github.com/${repo}.git"
    git clone --branch $branch $CLONE_URL
    cd $projectName
fi

#---------------------------------------编译-----------------------------------------
CGO_ENABLED=1 go build -ldflags="-s -w" -o $exec_file
if [ ! -e $exec_file ]; then
    echo "编译失败"
    exit 1
fi


#---------------------------------------创建标签----------------------------------------
# 获取版本号
version=$(grep -oP '(?<=VERSION = ")[^"]+' application/Application.go)
echo "获取到版本号:$version"

#删除本地已经存在的标签
git tag -d $version

#删除远程标签
git push origin --delete tag $version

git tag $version
git push origin $version

release_message="本次发布版本:$version"
create_release_api_response=$(curl -L -X POST "https://api.github.com/repos/$repo/releases" \
                        -H "Accept: application/vnd.github.v3+json" \
                        -H "Authorization: Bearer $github_token" \
                        -H "X-GitHub-Api-Version: 2022-11-28" \
                        -d "{\"tag_name\":\"$version\",\"name\":\"$version\",\"body\":\"$release_message\"}")
echo "创建标签结果:${create_release_api_response}"

#通过正则匹配ReleaseId, [head -n 1]功能是从匹配到的多个字符串中去第一个字符串
release_id=$(echo "$create_release_api_response" | grep -oP '(?<="id": )[^,]+' | head -n 1)
echo "本地发布ID:${release_id}"


#---------------------------------------上传编译好的二进制文件----------------------------------
upload_file_api_response=$(curl -s -X POST \
                            -H "Accept: application/vnd.github+json" \
                            -H "Authorization: Bearer ${github_token}" \
                            -H "X-GitHub-Api-Version: 2022-11-28" \
                            -H "Content-Type: application/octet-stream" \
                            --data-binary "@${exec_file}" \
                            "https://uploads.github.com/repos/${repo}/releases/${release_id}/assets?name=${exec_name}")

echo "上传文件结果:${upload_file_api_response}"


#---------------------------------------上传Docker镜像-----------------------------------------
mv $exec_file ./document/docker/
cd ./document/docker/
docker build -t $docker_user/$dockerImageName:$version .
docker login -u $docker_user --password $docker_pwd
docker push $docker_user/$dockerImageName:$version
docker logout

echo "---------------------------------------docker镜像推送完成--------------------------------------"
