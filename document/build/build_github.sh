#!/bin/bash

#创建版本标签和GitHub Release,上传build目录下编译好的程序(和DairoMUSIC-Search相同)
#Release里已经存在同名文件时,先删除再上传,所以可以重复执行
#需要环境变量:GITHUB_TOKEN
#可选:ONLY_UPLOAD=true  不重新创建标签,只上传文件(比如上传失败后补传,不需要重新编译)

#遇到错误立即停止
set -e

# 进入项目根目录
cd "$(dirname "$0")/../.."

#github登录票据(不要输出到日志)
github_token=$GITHUB_TOKEN

#项目名
projectName="DairoDFS"

repo="DAIRO-HY/$projectName"
branch=${BRANCH:-main}

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./DairoDFS/Cargo.toml)

#调用GitHub接口,结果写到日志文件,不输出到控制台
mkdir -p ./build
log_file=./build/push_github.log

github_api(){
  curl -sL \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer $github_token" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "$@"
}

#---------------------------------------创建标签----------------------------------------
if [ "$ONLY_UPLOAD" != "true" ]; then
  echo "正在创建标签 $version ..."

  #删除本地、远程已经存在的标签(第一次发布时不存在,忽略错误)
  git tag -d $version || true
  git push origin --delete tag $version || true
  git tag $version
  git push origin $version
fi

#---------------------------------------获取或创建Release----------------------------------
#按标签名查找,不按Release名称查找(原来所有版本的Release都叫latest,按名称会查到多个)
#重新创建标签后,GitHub会把原来的Release变成草稿(按标签查询不到草稿),所以从列表里按标签名查找,找到后改回正式发布
release_id=$(github_api "https://api.github.com/repos/${repo}/releases?per_page=100" \
  | jq -r ".[] | select(.tag_name==\"${version}\") | .id" | head -n 1)
if [ -n "$release_id" ]; then
  echo "正在更新Release $version ..."
  response=$(github_api -X PATCH https://api.github.com/repos/${repo}/releases/${release_id} \
    -d "{\"tag_name\":\"${version}\",\"target_commitish\":\"${branch}\",\"name\":\"${version}\",\"draft\":false,\"prerelease\":false}")
  echo "更新Release结果:${response}" >> $log_file
else
  echo "正在创建Release $version ..."
  response=$(github_api -X POST https://api.github.com/repos/${repo}/releases \
    -d "{\"tag_name\":\"${version}\",\"target_commitish\":\"${branch}\",\"name\":\"${version}\",\"body\":\"本次发布版本:${version}\"}")
  echo "创建Release结果:${response}" >> $log_file
  release_id=$(echo "$response" | jq -r '.id // empty')
fi
if [ -z "$release_id" ]; then
  echo "创建Release失败,详情见 $log_file"
  exit 1
fi
echo "Release ID:${release_id}"

#---------------------------------------上传编译好的程序----------------------------------
upload(){
  file="./build/$1"
  if [ ! -f "$file" ]; then
    echo "文件不存在,跳过:$file"
    return
  fi

  #已经存在同名文件时先删除
  asset_id=$(github_api https://api.github.com/repos/${repo}/releases/${release_id}/assets?per_page=100 \
    | jq -r ".[] | select(.name==\"$1\") | .id")
  if [ -n "$asset_id" ]; then
    echo "正在删除已经存在的文件:$1"
    github_api -X DELETE https://api.github.com/repos/${repo}/releases/assets/${asset_id}
  fi
  echo "正在上传:$1"
  response=$(github_api -X POST -H "Content-Type: application/octet-stream" \
    --data-binary "@$file" \
    "https://uploads.github.com/repos/${repo}/releases/${release_id}/assets?name=$1")
  echo "上传文件$1结果:${response}" >> $log_file

  #上传失败时停止,不要当成发布成功
  if [ -z "$(echo "$response" | jq -r '.id // empty' 2>/dev/null)" ]; then
    echo "上传失败:$1,详情见 $log_file"
    exit 1
  fi
}

upload "DairoDFS-linux-amd64-$version"
upload "DairoDFS-linux-arm64-$version"
upload "DairoDFS-mac-darwin-aarch64-$version"
upload "DairoDFS-mac-darwin-amd64-$version"
upload "DairoDFS-win-amd64-$version.exe"
upload "DairoDFS-win-arm64-$version.exe"
