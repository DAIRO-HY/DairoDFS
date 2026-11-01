#!/bin/bash

# 进入项目根目录
cd "$(dirname "$0")/../.."

#github登录票据(不要输出到日志)
github_token=$GITHUB_TOKEN

#项目名
projectName="DairoDFS"

repo="DAIRO-HY/$projectName"
branch="main"

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./DairoDFS/Cargo.toml)

tag_name="$version"

#---------------------------------------创建标签----------------------------------------
echo "正在创建标签 $tag_name ..."

#删除本地已经存在的标签
git tag -d $tag_name

#删除远程标签
git push origin --delete tag $tag_name

git tag $tag_name
git push origin $tag_name


#查询Release:latest的id
get_release_id(){
  release_json=$(curl -L \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer $github_token" \
    -H "X-GitHub-Api-Version: 2026-03-10" \
    https://api.github.com/repos/${repo}/releases)
  id=$(echo "$release_json" | jq -r ".[] | select(.name==\"latest\") | .id")

  #返回值
  echo $id
}

# 创建github的release
create_release(){
  create_release_api_response=$(curl -L \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $github_token" \
  -H "X-GitHub-Api-Version: 2026-03-10" \
  https://api.github.com/repos/${repo}/releases \
  -d "{\"tag_name\":\"${tag_name}\",\"target_commitish\":\"${branch}\",\"name\":\"latest\",\"body\":\"本次发布版本:${tag_name}\",\"draft\":false,\"prerelease\":false,\"generate_release_notes\":false}"
  )
  echo "创建Release结果:${create_release_api_response}" >> push_github_tag.log

  release_id=$(echo "$create_release_api_response" | grep -o '"id": [0-9]*' | head -n 1 | grep -o '[0-9]*')
  echo $release_id
}

# 修改github的release
update_release(){
  create_release_api_response=$(curl -L \
      -X PATCH \
      -H "Accept: application/vnd.github+json" \
      -H "Authorization: Bearer $github_token" \
      -H "X-GitHub-Api-Version: 2026-03-10" \
      https://api.github.com/repos/${repo}/releases/$1 \
      -d "{\"tag_name\":\"${tag_name}\",\"target_commitish\":\"${branch}\",\"name\":\"latest\",\"body\":\"本次发布版本:${tag_name}\",\"draft\":false,\"prerelease\":false}")
  echo "修改Release结果:${create_release_api_response}" >> push_github_tag.log
}

#---------------------------------------创建或修改Release:latest----------------------------------
release_id=$(get_release_id)
echo release_id:$release_id
if [ -n "$release_id" ]; then
  echo "正在修改Release:latest..."
  update_release $release_id
else
  echo "正在创建Release:latest..."
  release_id=$(create_release)
fi

#查询某个文件的id
get_asset_id_by_name(){
  assets_json=$(curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $github_token" \
  -H "X-GitHub-Api-Version: 2026-03-10" \
  https://api.github.com/repos/${repo}/releases/${release_id}/assets)
  id=$(echo "$assets_json" | jq -r ".[] | select(.name==\"$1\") | .id")

  echo "查询某个文件:$1 的结果:${assets_json}" >> push_github_tag.log

  #返回值
  echo $id
}

#删除某个文件
delete_asset(){
  echo "准备删除文件ID:$1"
  curl -L \
    -X DELETE \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer $github_token" \
    -H "X-GitHub-Api-Version: 2026-03-10" \
    https://api.github.com/repos/${repo}/releases/assets/$1
}

#上传文件
upload_file(){
  filename=$(basename $1)
  echo "正在上传编译好的二进制文件:${filename}..."
  upload_file_api_response=$(curl -L \
      -X POST \
      -H "Accept: application/vnd.github+json" \
      -H "Authorization: Bearer $github_token" \
      -H "X-GitHub-Api-Version: 2026-03-10" \
      -H "Content-Type: application/octet-stream" \
      "https://uploads.github.com/repos/${repo}/releases/${release_id}/assets?name=${filename}" \
      --data-binary "@$1")
  echo "上传文件$1结果:${upload_file_api_response}" >> push_github_tag.log
}

# 上传文件,Release里已经存在同名文件时,先删除再上传
check_and_upload_file(){
  file_name="./build/$1"
  if [ ! -f $file_name ]; then
      echo "文件不存在:$file_name" >> push_github_tag.log
      return
  fi
  asset_id=$(get_asset_id_by_name $1)
  echo asset_id:$asset_id
  if [ -n "$asset_id" ]; then
    echo "正在删除已经存在的文件$file_name..."
    delete_asset $asset_id
  fi
  upload_file $file_name
}

#---------------------------------------上传编译好的二进制文件----------------------------------
check_and_upload_file "DairoDFS-linux-amd64-$version"
check_and_upload_file "DairoDFS-linux-arm64-$version"
check_and_upload_file "DairoDFS-mac-darwin-aarch64-$version"
check_and_upload_file "DairoDFS-mac-darwin-amd64-$version"
check_and_upload_file "DairoDFS-win-amd64-$version.exe"
