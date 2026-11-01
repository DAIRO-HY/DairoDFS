use hyperoute_codegen::{Embed, ParamBinding, ParamNaming, StaticDir};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() {

    // 生成 DAO 相关的代码块
    generate_dao().await;
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = manifest_dir.join("src");

    // 扫描controller、filter下的注解，并挂载静态资源，生成路由代码
    let routes = hyperoute_codegen::Scanner::new()
        .dir(src_dir.join("web/controller"), "crate::web::controller")
        .dir(src_dir.join("web/filter"), "crate::web::filter")
        // 普通参数同时接收驼峰（folderId）和Rust原名（folder_id）
        .param_naming(ParamNaming::CamelCase)
        // 参数同时从URL和表单请求体中获取,PUT/DELETE等请求的 ids=1&ids=2 这类数组参数也能直接绑定到Vec
        .param_binding(ParamBinding::Merged)
        .static_dir(
            StaticDir::new("/", manifest_dir.join("assets/resources"))
                // 始终打包进二进制，发布时只有一个文件
                .embed(Embed::Always)
                // html里的 {{template include/head}} 先合并成完整页面
                .html_include(true)
                // 浏览器每次都带ETag来验证，没变就是304，部署后能立即拿到新资源
                .cache_control("no-cache")
                .gzip(true),
        )
        .scan();
    let code = hyperoute_codegen::generate_router_source(&routes);
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("routes.rs");
    fs::write(&out_path, code).unwrap();

    // 追踪输入文件变化
    println!("cargo:rerun-if-changed=src/web/controller");
    println!("cargo:rerun-if-changed=src/web/filter");
    println!("cargo:rerun-if-changed=assets/resources/");
}

// 生成 DAO 相关的代码块
async fn generate_dao() {

    // -------------------------------------Dao自动生成------------------START

    //编译期专用的数据库,只用来生成DAO和支持sqlx宏编译检查sql语句,每次构建都按ddl重新建表
    //放在OUT_DIR下,不和开发数据目录里的真实数据混在一起
    //必须用绝对路径:build.rs的工作目录是当前crate目录,而sqlx宏执行时的工作目录是工作空间根目录,用相对路径两边会指向不同的文件
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let db_path = out_dir.join("dairo-dfs-build.sqlite");
    lib_db::init_build(&db_path).await;

    //告诉sqlx宏使用同一个数据库文件
    println!("cargo:rustc-env=DATABASE_URL=sqlite://{}?mode=rwc", db_path.display());

    //开发时(debug编译)程序默认使用工作空间根目录下的data目录
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let data_dir = manifest_dir.parent().unwrap().join("data");
    println!("cargo:rustc-env=DAIRO_DFS_DEV_DATA_DIR={}", data_dir.display());

    // 字段名 -> Rust类型 的覆盖规则(本项目自定义的字段命名约定)
    let type_overrides = HashMap::from([]);

    // 生成 DAO 相关的代码块
    sqlx_dsl_dao::generate(lib_db::get(), "./assets/dsl", &type_overrides).await;

    // -------------------------------------Dao自动生成------------------END

    println!("cargo:rerun-if-changed=assets/ddl/");
    println!("cargo:rerun-if-changed=assets/dsl/");
}
