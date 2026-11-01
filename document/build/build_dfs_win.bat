@echo off
chcp 65001 >nul
cd %USERPROFILE%
if not exist .\develop (
    mkdir develop
)
cd develop
if not exist .\DairoDFS (
    echo "is not exists DairoDFS, clone from github"
    git clone https://github.com/DAIRO-HY/DairoDFS.git
)
cd DairoDFS

rem 获取最新的代码
git reset --hard
git pull
cargo +1.99.1 build --release --package DairoDFS
