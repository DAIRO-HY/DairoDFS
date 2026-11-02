use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::sub_account_grant_dao::SubAccountGrant;
use crate::dao::{dfs_file_dao, sub_account_grant_dao, user_dao, user_token_dao};
use crate::util::{file_util, id_util, time_util};
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use crate::{biz_error, biz_errorf};
use sqlx_context::DbContext;
use std::collections::{HashMap, HashSet};

/// 子账户的权限,可以组合
pub mod perm {
    // 浏览:列表、属性、缩略图、相册
    pub const VIEW: i64 = 1;

    // 打开/下载:预览、播放、下载原文件
    pub const OPEN: i64 = 2;

    // 上传:上传文件、新建文件夹
    pub const UPLOAD: i64 = 4;

    // 修改:重命名、移动、复制、修改文件类型
    pub const MODIFY: i64 = 8;

    // 删除:删除到回收站、从回收站恢复
    pub const DELETE: i64 = 16;

    // 彻底删除:从回收站彻底删除
    pub const PURGE: i64 = 32;

    // 分享:创建分享链接、管理自己的分享
    pub const SHARE: i64 = 64;

    // 所有权限
    pub const ALL: i64 = VIEW | OPEN | UPLOAD | MODIFY | DELETE | PURGE | SHARE;

    // 授权文件夹的上级文件夹:只能浏览,并且只显示通往授权文件夹的子文件夹(不保存到数据库)
    pub const PASS: i64 = 1 << 30;

    /// 权限名称,用于提示
    pub fn name(perm: i64) -> &'static str {
        match perm {
            VIEW => "浏览",
            OPEN => "打开/下载",
            UPLOAD => "上传",
            MODIFY => "修改",
            DELETE => "删除",
            PURGE => "彻底删除",
            SHARE => "分享",
            _ => "操作",
        }
    }
}

/// 当前登录用户对主账户文件的访问范围,主账户不受限制
/// 权限从授权的文件夹往下继承,有多层授权时离文件最近的授权生效
pub struct Scope {
    // 是否子账户
    is_sub: bool,

    // 授权的文件夹ID -> 权限
    grants: HashMap<i64, i64>,

    // 授权文件夹的所有上级文件夹ID
    ancestors: HashSet<i64>,

    // 文件夹ID -> 继承的权限,避免重复查询
    cache: HashMap<i64, i64>,
}

impl Scope {
    /// 获取登录用户的访问范围
    pub async fn load(ctx: &mut DbContext, login: &LoginUser) -> Result<Self, BizError> {
        let mut scope = Scope {
            is_sub: login.is_sub,
            grants: HashMap::new(),
            ancestors: HashSet::new(),
            cache: HashMap::new(),
        };
        if !login.is_sub {
            return Ok(scope);
        }
        for grant in sub_account_grant_dao::select_by_sub_user_id(&mut *ctx, login.id).await? {
            let Ok(folder) = dfs_file_dao::select_one(&mut *ctx, grant.folderId).await else {
                continue;
            };
            if folder.userId != login.owner_id || folder.deleteDate.is_some() {
                continue;
            }
            scope.grants.insert(folder.id, grant.perms);
            let mut parent_id = folder.parentId;
            while parent_id != 0 && scope.ancestors.insert(parent_id) {
                parent_id = dfs_file_dao::select_one(&mut *ctx, parent_id).await?.parentId;
            }
        }
        Ok(scope)
    }

    /// 是否子账户(主账户不受限制)
    pub fn is_sub(&self) -> bool {
        self.is_sub
    }

    /// 获取对文件或文件夹的权限,没有任何权限时返回0,授权文件夹的上级文件夹返回PASS
    pub async fn perms(&mut self, ctx: &mut DbContext, file: &DfsFile) -> Result<i64, BizError> {
        if !self.is_sub {
            return Ok(perm::ALL);
        }
        if let Some(perms) = self.grants.get(&file.id) {
            return Ok(*perms);
        }
        let perms = self.inherited(ctx, file.parentId).await?;
        if perms != 0 {
            return Ok(perms);
        }
        if self.ancestors.contains(&file.id) {
            return Ok(perm::PASS);
        }
        Ok(0)
    }

    /// 从上级继承的权限(从id开始往上找最近的授权)
    async fn inherited(&mut self, ctx: &mut DbContext, id: i64) -> Result<i64, BizError> {
        let mut path = Vec::new();
        let mut cur = id;
        let perms = loop {
            if cur == 0 {
                break 0;
            }
            if let Some(perms) = self.grants.get(&cur).or_else(|| self.cache.get(&cur)) {
                break *perms;
            }
            path.push(cur);
            cur = match dfs_file_dao::select_one(&mut *ctx, cur).await {
                Ok(it) => it.parentId,
                Err(sqlx::Error::RowNotFound) => 0,
                Err(e) => return Err(e.into()),
            };
        };
        for it in path {
            self.cache.insert(it, perms);
        }
        Ok(perms)
    }

    /// 检查对文件或文件夹本身的权限,用于浏览、打开、修改文件类型、分享等
    pub async fn check(&mut self, ctx: &mut DbContext, file: &DfsFile, need: i64) -> Result<(), BizError> {
        let perms = self.perms(ctx, file).await?;
        Self::check_perms(perms, need)
    }

    /// 检查所在文件夹的权限,用于重命名、移动、删除等改变文件位置的操作
    /// 授权的文件夹本身不能被子账户重命名、移动或删除
    pub async fn check_in_parent(&mut self, ctx: &mut DbContext, file: &DfsFile, need: i64) -> Result<(), BizError> {
        if !self.is_sub {
            return Ok(());
        }
        if self.perms(ctx, file).await? == 0 {
            return Err(biz_error!("文件夹或文件不存在"));
        }
        let perms = self.inherited(ctx, file.parentId).await?;
        if self.grants.contains_key(&file.id) && perms & need == 0 {
            return Err(biz_errorf!("不能{}授权的文件夹", perm::name(need)));
        }
        Self::check_perms(perms, need)
    }

    /// 检查在某个路径下添加文件的权限(路径不存在时检查最近一级已经存在的上级文件夹),用于上传、新建文件夹、复制和移动的目标位置
    pub async fn check_path(&mut self, ctx: &mut DbContext, owner_id: i64, path: &str, need: i64) -> Result<(), BizError> {
        if !self.is_sub {
            return Ok(());
        }
        let mut parent_id = 0;
        let mut last = None;
        for name in file_util::to_sub_names(path)? {
            let Some(it) = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, owner_id, parent_id, name).await? else {
                break;
            };
            parent_id = it.id;
            last = Some(it);
        }
        let Some(last) = last else {
            return Err(biz_errorf!("没有{}权限", perm::name(need)));
        };
        self.check(ctx, &last, need).await
    }

    /// 文件夹列表里是否显示这个子文件或子文件夹
    /// - parent_perms 所在文件夹的权限
    pub fn visible(&self, parent_perms: i64, child_id: i64) -> bool {
        if parent_perms & perm::VIEW != 0 {
            return true;
        }
        parent_perms & perm::PASS != 0 && (self.grants.contains_key(&child_id) || self.ancestors.contains(&child_id))
    }

    fn check_perms(perms: i64, need: i64) -> Result<(), BizError> {
        if perms == 0 {
            //看不到的文件,当作不存在
            return Err(biz_error!("文件夹或文件不存在"));
        }
        if perms & need != need {
            return Err(biz_errorf!("没有{}权限", perm::name(need)));
        }
        Ok(())
    }
}

/// 补全依赖的权限:有任何权限都能浏览;彻底删除需要删除权限;分享出去别人就能下载,所以需要打开权限
pub fn normalize_perms(perms: i64) -> i64 {
    let mut perms = perms & perm::ALL;
    if perms == 0 {
        return 0;
    }
    perms |= perm::VIEW;
    if perms & perm::PURGE != 0 {
        perms |= perm::DELETE;
    }
    if perms & perm::SHARE != 0 {
        perms |= perm::OPEN;
    }
    perms
}

/// 检查子账户的密码是否符合要求:8到32位,同时包含字母和数字
pub fn check_pwd(pwd: &str) -> Result<(), BizError> {
    let len = pwd.chars().count();
    if !(8..=32).contains(&len) {
        return Err(BizError::field("pwd", "密码长度必须在8到32位之间"));
    }
    if !pwd.chars().any(|it| it.is_ascii_alphabetic()) || !pwd.chars().any(|it| it.is_ascii_digit()) {
        return Err(BizError::field("pwd", "密码必须同时包含字母和数字"));
    }
    if pwd.chars().any(|it| it.is_whitespace()) {
        return Err(BizError::field("pwd", "密码不能包含空格"));
    }
    Ok(())
}

/// 获取主账户的一个子账户,不是该主账户的子账户或者已经删除时返回错误
pub async fn select_own(ctx: &mut DbContext, parent_id: i64, id: i64) -> Result<user_dao::User, BizError> {
    match user_dao::select_one(&mut *ctx, id).await {
        Ok(it) if it.parentId == parent_id && it.state != 2 => Ok(it),
        _ => Err(biz_error!("子账户不存在")),
    }
}

/// 重新设置子账户的授权
/// - grants 文件夹ID和权限
pub async fn set_grants(ctx: &mut DbContext, sub_user_id: i64, grants: &[(i64, i64)]) -> Result<(), sqlx::Error> {
    sub_account_grant_dao::delete_by_sub_user_id(&mut *ctx, sub_user_id).await?;
    for (folder_id, perms) in grants {
        let grant = SubAccountGrant {
            id: id_util::id(),
            subUserId: sub_user_id,
            folderId: *folder_id,
            perms: *perms,
            date: time_util::current_millis(),
        };
        sub_account_grant_dao::insert(&mut *ctx, &grant).await?;
    }
    Ok(())
}

/// 删除子账户:清除登录记录和授权,已经上传的文件仍然属于主账户
pub async fn delete(ctx: &mut DbContext, id: i64) -> Result<(), sqlx::Error> {
    user_dao::set_sub_deleted(&mut *ctx, id).await?;
    sub_account_grant_dao::delete_by_sub_user_id(&mut *ctx, id).await?;
    user_token_dao::delete_by_user_id(&mut *ctx, id).await?;
    Ok(())
}
