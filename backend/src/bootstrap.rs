//! 应用启动引导。
//!
//! [`bootstrap`] 由 `App::on_startup()` 调用，执行时机固定为
//! 「全局资源（配置 / 数据库 / 缓存 / 上传下载目录）已初始化完成、插件尚未启动」，
//! 因此 `cfg()` / `db()` / `try_cache()` 等全局访问器在此均可安全使用。
//!
//! 本模块承载**业务侧**的初始化：
//!
//! 1. 输出启动摘要；
//! 2. 写入幂等种子数据：管理员账号 `admin` → 超级管理员角色绑定（见 [`seed_super_admin`]）；
//! 3. 装载定时任务（系统内置任务 + `sys_job` 中启用的任务，见 [`load_scheduled_jobs`]）；
//! 4. 预热内置字典缓存（见 [`warm_dict_cache`]）；
//! 5. 装载运行时参数快照（见 [`load_runtime_config`]），供订阅了参数的插件直接取用。
//!
//! 内置超级管理员角色、菜单与按钮权限点等**无需运行期计算的种子数据**已全部下沉到
//! `migrations/seed/S__*.sql` 种子迁移；后续新增功能只需追加迁移，无需改动本文件。
//!
//! 基础设施的初始化由框架在 `App::start()` 内完成，不要在此重复做。

use alun::prelude::*;

use crate::shared::constants;
use crate::system::role::model::SysRoleModel;
use crate::system::user::model::{SysUserModel, SysUserRoleModel};

/// 启动钩子：完成业务侧初始化并输出启动摘要。
///
/// # 异常情况
///
/// 钩子签名没有错误通道（`FnOnce() -> Future<Output = ()>`），因此这里不做可能失败的
/// 操作；需要失败即中止启动的校验必须放在 `App::start()` 之前的代码路径中。
/// 种子数据写入失败只记录 `tracing::error!` 并继续，不阻断服务启动。
pub async fn bootstrap() {
    let app_name = cfg().app_name.clone();
    let profile = cfg().profile.clone();

    tracing::info!(
        app = %app_name,
        %profile,
        db_driver = %cfg().database.r#type,
        "启动引导完成"
    );

    // 四步互不依赖，并发执行：原先串行会把各步等待累加进启动耗时；
    // 每步自身的失败处理（只记日志、不阻断启动）保持不变，
    // 仅启动日志的先后顺序不再固定。
    tokio::join!(
        seed_super_admin(),
        load_scheduled_jobs(),
        crate::support::dict::model::warm_dict_cache(),
        load_runtime_config(),
    );
}

/// 装载运行时参数快照（`sys_config` → 进程内 `RwLock<HashMap>`）。
///
/// 必须在插件启动**之前**执行：插件（如 [`crate::support::log::LogPlugin`]）通过
/// [`subscribe_runtime`] 订阅参数，注册时会立刻以当前快照回调一次；快照未装载则
/// 回调被跳过，只能等运行期首次写操作或 TTL 过期回源才补上，导致启动瞬间读到的是
/// 回退默认值而非库中配置。
///
/// # 异常情况
///
/// 装载失败只记录错误日志并继续（启动钩子没有错误通道）：读路径会回源数据库，
/// 快照缺失只损失性能，不影响正确性。
async fn load_runtime_config() {
    match crate::support::config::model::reload_runtime_snapshot().await {
        Ok(count) => tracing::info!(params = count, "运行时参数快照已装载"),
        Err(e) => tracing::error!(error = %e, "运行时参数快照装载失败，读路径将回源数据库"),
    }
    // 启动跨实例同步：其它实例改参数后本实例在轮询间隔内自动跟随，无需重启或手动
    // 调用重载接口（缓存未启用时内部自行跳过）
    crate::support::config::model::spawn_config_version_watcher();
}

/// 装载定时任务：注册系统内置任务，并从 `sys_job` 读取启用任务一并注册。
///
/// 调用 [`crate::task::scheduler`] 的 `reload()`：内部先停止调度器，再按
/// 「系统内置任务 + 库中启用任务」重建注册表并重新启动调度。
///
/// # 异常情况
///
/// 装载失败只记录错误日志并继续（启动钩子没有错误通道）：此时调度器仍会保留
/// 已经注册成功的任务，不影响服务对外提供接口。
async fn load_scheduled_jobs() {
    if let Err(e) = crate::task::scheduler().reload().await {
        tracing::error!("定时任务装载失败: {e:?}");
    }
}

/// 写入内置种子数据：管理员账号 `admin`，以及它与超级管理员角色的绑定关系。
///
/// # 与迁移的分工
///
/// 超级管理员**角色**由种子迁移 `migrations/seed/S__role_data.sql` 创建，此处不再
/// 写角色，只在角色就绪后查询其主键，完成「账号 → 角色」绑定。角色缺失属异常
/// （迁移未执行），此时只记错误日志并跳过绑定。
///
/// # 幂等性
///
/// 账号按 `username = admin` 判定、绑定按 `(user_id, role_id)` 计数判定，存在即
/// 跳过，因此重启不会产生重复数据。
///
/// # 初始口令
///
/// 管理员密码取自环境变量 [`constants::ADMIN_PASSWORD_ENV`]；未注入时回退到
/// [`constants::DEFAULT_ADMIN_PASSWORD`] 并打印提示，要求首次登录后立即修改。
///
/// # 异常情况
///
/// 任一步骤失败只记录错误日志并继续（启动钩子无错误通道）：账号写入失败、角色查询
/// 失败或角色缺失时均跳过绑定；绑定失败仅记录错误。数据库未初始化时直接返回。
async fn seed_super_admin() {
    let db = match try_db() {
        Some(db) => db,
        None => {
            tracing::error!("数据库未初始化，跳过种子数据写入");
            return;
        }
    };

    let user = match ensure_admin_user(db).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!(error = %e, "写入管理员账号失败，跳过角色绑定");
            return;
        }
    };

    // 内置超级管理员角色由种子迁移创建，此处只查询其主键用于绑定。
    let role = match SysRoleModel::find_first_by(
        db,
        "role_code",
        (constants::SUPER_ADMIN_ROLE_CODE,),
    )
    .await
    {
        Ok(Some(role)) => role,
        Ok(None) => {
            tracing::error!(
                role_code = constants::SUPER_ADMIN_ROLE_CODE,
                "内置超级管理员角色缺失（应由 migrations/seed/S__role_data.sql 创建），跳过管理员角色绑定"
            );
            return;
        }
        Err(e) => {
            tracing::error!(error = %e, "查询超级管理员角色失败，跳过角色绑定");
            return;
        }
    };

    if let Err(e) = ensure_admin_role_binding(db, &user.id, &role.id).await {
        tracing::error!(error = %e, "写入管理员角色绑定失败");
    }
}

/// 确保内置管理员账号存在，返回该账号（已存在则直接返回库中记录）。
///
/// # 参数
///
/// - `db`：数据库执行器。
///
/// # 返回值
///
/// 库中 `username = admin` 的用户模型；不存在时以初始口令创建后返回。
///
/// # 异常情况
///
/// 查询失败、密码哈希失败或插入失败时返回 [`DbError`]。
async fn ensure_admin_user(db: &Db) -> std::result::Result<SysUserModel, DbError> {
    if let Some(user) =
        SysUserModel::find_first_by(db, "username", (constants::ADMIN_USERNAME,)).await?
    {
        tracing::info!(
            username = constants::ADMIN_USERNAME,
            "管理员账号已存在，跳过创建"
        );
        return Ok(user);
    }

    // 初始口令：优先取环境变量；缺失时回退默认口令——默认口令使用**预生成
    // 哈希常量**（`DEFAULT_ADMIN_PASSWORD_HASH`），避免首次启动现场执行
    // Argon2 慢哈希（本机 debug 构建实测约 420ms，空库初始化的第二大头）。
    let password = match std::env::var(constants::ADMIN_PASSWORD_ENV) {
        Ok(custom) => {
            // 自定义口令必须现场随机加盐哈希；Argon2 为 CPU 密集慢哈希，
            // 放入阻塞线程池执行，避免占据异步执行器线程。
            tokio::task::spawn_blocking(move || {
                Crypto::hash_password(&custom).map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| DbError::InvalidValue(format!("管理员初始密码哈希任务失败: {e}")))?
            .map_err(|e| DbError::InvalidValue(format!("管理员初始密码哈希失败: {e}")))?
        }
        Err(_) => {
            tracing::info!(
                env = constants::ADMIN_PASSWORD_ENV,
                "未通过环境变量注入管理员初始密码，改用内置默认初始口令（预置哈希），请在首次登录后立即修改"
            );
            constants::DEFAULT_ADMIN_PASSWORD_HASH.to_string()
        }
    };

    let now = chrono::Utc::now();
    let mut user = SysUserModel {
        id: uuid::Uuid::now_v7(),
        username: constants::ADMIN_USERNAME.to_string(),
        password,
        real_name: Some(constants::SUPER_ADMIN_ROLE_NAME.to_string()),
        email: None,
        mobile: None,
        dept_id: None,
        status: constants::EnableStatus::enabled,
        login_fail_count: 0,
        is_locked: false,
        lock_time: None,
        last_login_at: None,
        remark: None,
        created_at: now,
        updated_at: now,
        create_by: None,
        update_by: None,
        deleted_at: None,
    };
    user.insert(db).await?;

    tracing::info!(
        username = constants::ADMIN_USERNAME,
        "已写入管理员账号种子数据"
    );
    Ok(user)
}

/// 确保管理员账号与超级管理员角色的绑定存在。
///
/// # 参数
///
/// - `db`：数据库执行器。
/// - `user_id`：管理员账号主键。
/// - `role_id`：超级管理员角色主键。
///
/// # 返回值
///
/// 绑定已存在或写入成功时返回 `Ok(())`。
///
/// # 异常情况
///
/// 计数查询或插入失败时返回 [`DbError`]。
async fn ensure_admin_role_binding(
    db: &Db,
    user_id: &uuid::Uuid,
    role_id: &uuid::Uuid,
) -> std::result::Result<(), DbError> {
    let existing =
        SysUserRoleModel::count_by(db, "user_id = ? AND role_id = ?", (user_id, role_id)).await?;
    if existing > 0 {
        tracing::info!(
            username = constants::ADMIN_USERNAME,
            "管理员角色绑定已存在，跳过创建"
        );
        return Ok(());
    }

    // 关联表关闭了时间戳 Hook，created_at 需显式赋值。
    let mut binding = SysUserRoleModel {
        id: uuid::Uuid::now_v7(),
        user_id: *user_id,
        role_id: *role_id,
        expire_time: None,
        created_at: chrono::Utc::now(),
    };
    binding.insert(db).await?;

    tracing::info!(
        username = constants::ADMIN_USERNAME,
        role_code = constants::SUPER_ADMIN_ROLE_CODE,
        "已写入管理员角色绑定种子数据"
    );
    Ok(())
}
