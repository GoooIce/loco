//! Loco 应用程序模块
//!
//! 这个模块包含了构建 Web 服务器应用程序的核心组件和 trait。
//! 它定义了应用程序的核心结构、生命周期钩子和类型安全的数据存储。
use std::{
    any::{Any, TypeId},
    net::SocketAddr,
    sync::Arc,
};

use async_trait::async_trait;
use axum::Router as AxumRouter;
use dashmap::DashMap;
#[cfg(feature = "with-db")]
use {sea_orm::DatabaseConnection, std::path::Path};

use crate::{
    bgworker::{self, Queue},
    boot::{shutdown_signal, BootResult, ServeParams, StartMode},
    cache::{self},
    config::Config,
    controller::{
        middleware::{self, MiddlewareLayer},
        AppRoutes,
    },
    environment::Environment,
    mailer::EmailSender,
    storage::Storage,
    task::Tasks,
    Result,
};

/// 类型安全的异构应用程序数据存储
///
/// 用于在应用程序中存储不同类型的数据，使用 DashMap 实现线程安全访问。
#[derive(Default, Debug)]
pub struct SharedStore {
    // 使用 DashMap 实现并发访问，具有细粒度锁定机制
    storage: DashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl SharedStore {
    /// Insert a value of type T into the shared store
    ///
    /// # Example
    /// ```
    /// # use loco_rs::app::SharedStore;
    /// let shared_store = SharedStore::default();
    ///
    /// #[derive(Debug)]
    /// struct TestService {
    ///     name: String,
    ///     value: i32,
    /// }
    ///
    /// let service = TestService {
    ///     name: "test".to_string(),
    ///     value: 100,
    /// };
    ///
    /// shared_store.insert(service);
    /// assert!(shared_store.contains::<TestService>());
    /// ```
    pub fn insert<T: 'static + Send + Sync>(&self, val: T) {
        self.storage.insert(TypeId::of::<T>(), Box::new(val));
    }

    /// Remove a value of type T from the shared store
    ///
    /// Returns `Some(T)` if the value was present and removed, `None`
    /// otherwise.
    ///
    /// # Example
    /// ```
    /// # use loco_rs::app::SharedStore;
    /// let shared_store = SharedStore::default();
    ///
    /// struct TestService {
    ///     name: String,
    ///     value: i32,
    /// }
    ///
    /// let service = TestService {
    ///     name: "test".to_string(),
    ///     value: 100,
    /// };
    ///
    /// shared_store.insert(service);
    /// assert!(shared_store.contains::<TestService>());
    ///
    /// // Remove and get the value
    /// let removed_service_opt = shared_store.remove::<TestService>();
    /// assert!(removed_service_opt.is_some(), "Service should be present");
    /// // Assert fields individually instead of comparing the whole struct
    /// if let Some(removed_service) = removed_service_opt {
    ///      assert_eq!(removed_service.name, "test");
    ///      assert_eq!(removed_service.value, 100);
    /// }
    /// // Ensure it's gone
    /// assert!(!shared_store.contains::<TestService>());
    ///
    /// // Trying to remove again returns None
    /// assert!(shared_store.remove::<TestService>().is_none());
    /// ```
    #[must_use]
    pub fn remove<T: 'static + Send + Sync>(&self) -> Option<T> {
        self.storage
            .remove(&TypeId::of::<T>())
            .map(|(_, v)| v) // Extract the Box<dyn Any>
            .and_then(|any| any.downcast::<T>().ok()) // Downcast to Box<T>
            .map(|boxed| *boxed) // Dereference the Box<T> to get T
    }

    /// Get a reference to a value of type T from the shared store.
    ///
    /// Returns `None` if the value doesn't exist.
    /// The reference is valid for as long as the returned `RefGuard` is held.
    /// If you need to clone the value, you can do so directly from the
    /// returned reference, or use the `get` method instead.
    ///
    /// # Example
    /// ```
    /// # use loco_rs::app::SharedStore;
    /// let shared_store = SharedStore::default();
    ///
    /// #[derive(Clone)]
    /// struct TestService {
    ///     name: String,
    ///     value: i32,
    /// }
    ///
    /// let service = TestService {
    ///     name: "test".to_string(),
    ///     value: 100,
    /// };
    ///
    /// shared_store.insert(service);
    ///
    /// // Get a reference to the service
    /// let service_ref = shared_store.get_ref::<TestService>().expect("Service not found");
    /// // Access fields directly
    /// assert_eq!(service_ref.name, "test");
    /// assert_eq!(service_ref.value, 100);
    ///
    /// // Clone if needed (the field itself)
    /// let name_clone = service_ref.name.clone();
    /// assert_eq!(name_clone, "test");
    ///
    /// // Compute values from the reference
    /// let name_len = service_ref.name.len();
    /// assert_eq!(name_len, 4);
    /// ```
    #[must_use]
    pub fn get_ref<T: 'static + Send + Sync>(&self) -> Option<RefGuard<'_, T>> {
        let type_id = TypeId::of::<T>();
        self.storage.get(&type_id).map(|r| RefGuard::<T> {
            inner: r,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Get a clone of a value of type T from the shared store.
    /// Requires T to implement Clone.
    ///
    /// Returns `None` if the value doesn't exist.
    /// This method clones the stored value.
    /// If cloning is not desired or T does not implement Clone,
    /// use `get_ref` instead.
    ///
    /// # Example
    /// ```
    /// # use loco_rs::app::SharedStore;
    /// let shared_store = SharedStore::default();
    ///
    /// #[derive(Clone)]
    /// struct TestService {
    ///     name: String,
    ///     value: i32,
    /// }
    ///
    /// let service = TestService {
    ///     name: "test".to_string(),
    ///     value: 100,
    /// };
    ///
    /// shared_store.insert(service);
    ///
    /// // Get a clone of the service
    /// let service_clone_opt = shared_store.get::<TestService>();
    /// assert!(service_clone_opt.is_some(), "Service not found");
    /// // Assert fields individually
    /// if let Some(ref service_clone) = service_clone_opt {
    ///     assert_eq!(service_clone.name, "test");
    ///     assert_eq!(service_clone.value, 100);
    /// }
    /// ```
    #[must_use]
    pub fn get<T: 'static + Send + Sync + Clone>(&self) -> Option<T> {
        self.get_ref::<T>().map(|guard| (*guard).clone())
    }

    /// Check if the shared store contains a value of type T
    ///
    /// # Example
    /// ```
    /// # use loco_rs::app::SharedStore;
    /// let shared_store = SharedStore::default();
    ///
    /// struct TestService {
    ///     name: String,
    ///     value: i32,
    /// }
    ///
    /// let service = TestService {
    ///     name: "test".to_string(),
    ///     value: 100,
    /// };
    ///
    /// shared_store.insert(service);
    /// assert!(shared_store.contains::<TestService>());
    /// assert!(!shared_store.contains::<String>());
    /// ```
    #[must_use]
    pub fn contains<T: 'static + Send + Sync>(&self) -> bool {
        self.storage.contains_key(&TypeId::of::<T>())
    }
}

// A wrapper around DashMap's Ref type that erases the exact type
// but provides deref to the target type
pub struct RefGuard<'a, T: 'static + Send + Sync> {
    inner: dashmap::mapref::one::Ref<'a, TypeId, Box<dyn Any + Send + Sync>>,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<T: 'static + Send + Sync> std::ops::Deref for RefGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // This is safe because we only create a RefGuard for a specific type
        // after looking it up by its TypeId
        #[allow(clippy::coerce_container_to_any)]
        self.inner
            .value()
            .downcast_ref::<T>()
            .expect("Type mismatch in RefGuard")
    }
}

/// Represents the application context for a web server.
///
/// Loco 应用上下文结构体
///
/// 此结构体封装了 Web
/// 服务器运行所需的各种组件和配置。
/// 它通常用于存储和管理在整个应用程序生命周期中可访问的共享资源和设置。
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct AppContext {
    /// 应用运行的环境信息
    pub environment: Environment,
    #[cfg(feature = "with-db")]
    /// 应用程序使用的数据库连接
    pub db: DatabaseConnection,
    /// 队列提供者，用于后台任务处理
    pub queue_provider: Option<Arc<bgworker::Queue>>,
    /// 应用配置设置
    pub config: Config,
    /// 可选的邮件发送组件，可用来发送电子邮件
    pub mailer: Option<EmailSender>,
    /// 应用程序的可选存储实例
    pub storage: Arc<Storage>,
    /// 应用程序缓存实例
    pub cache: Arc<cache::Cache>,
    /// 用于存储任意应用程序数据的共享存储器
    pub shared_store: Arc<SharedStore>,
}

/// A trait that defines hooks for customizing and extending the behavior of a
/// web server application.
///
/// Web 服务器应用程序的钩子 trait。
///
/// 应用程序用户需实现此 trait
/// 来根据特定需求和使用场景自定义应用程序的路由、工作进程连接、
/// 任务注册以及数据库操作。
#[async_trait]
pub trait Hooks: Send {
    /// 定义应用程序的综合版本号
    #[must_use]
    fn app_version() -> String {
        "dev".to_string()
    }

    /// 定义 crate 的名称
    ///
    /// 示例代码:
    /// ```rust
    /// fn app_name() -> &'static str {
    ///     env!("CARGO_CRATE_NAME")
    /// }
    /// ```
    fn app_name() -> &'static str;

    /// 根据指定的模式和环境初始化并启动应用程序。
    ///
    /// 启动初始化过程可能会根据是否使用 DB migrator 而有所不同。
    ///
    /// # 示例代码
    ///
    /// 带有数据库的场景:
    /// ```rust,ignore
    /// async fn boot(mode: StartMode, environment: &str, config: Config) -> Result<BootResult> {
    ///     create_app::<Self, Migrator>(mode, environment, config).await
    /// }
    /// ````
    ///
    /// 不带数据库的场景:
    /// ```rust,ignore
    /// async fn boot(mode: StartMode, environment: &str, config: Config) -> Result<BootResult> {
    ///     create_app::<Self>(mode, environment, config).await
    /// }
    /// ````
    ///
    ///
    /// # 错误处理
    /// 如果无法启动应用程序则返回错误
    async fn boot(mode: StartMode, environment: &Environment, config: Config)
        -> Result<BootResult>;

    /// 启动 Axum Web 应用服务器，在指定的地址和端口上监听。
    ///
    /// # 返回值
    /// 成功时返回 Result<()>，服务器启动失败则返回错误。
    async fn serve(app: AxumRouter, ctx: &AppContext, serve_params: &ServeParams) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(&format!(
            "{}:{}",
            serve_params.binding, serve_params.port
        ))
        .await?;

        let cloned_ctx = ctx.clone();
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("shutting down...");
            Self::on_shutdown(&cloned_ctx).await;
        })
        .await?;

        Ok(())
    }

    /// 覆盖并返回 `Ok(true)` 以提供自定义的日志和 tracing 系统。
    /// 当返回 `Ok(true)` 时，Loco 将不会初始化自己的日志器，
    /// 因此您需要设置完整的 tracing 和 logging 系统。
    ///
    /// # 错误处理
    /// 失败时返回错误
    fn init_logger(_ctx: &AppContext) -> Result<bool> {
        Ok(false)
    }

    /// 根据给定环境加载应用程序的配置设置。
    ///
    /// 此函数负责基于当前环境检索应用程序的配置信息。
    async fn load_config(env: &Environment) -> Result<Config> {
        env.load()
    }

    /// 返回应用程序的初始 Axum router，允许用户控制 Axum router 的构建过程。
    /// 这里可以安装 fallback handler，在中间件或其它路由之前添加。
    ///
    /// # 错误处理
    /// 当 router 无法创建时返回 [`Result`]
    async fn before_routes(_ctx: &AppContext) -> Result<AxumRouter<AppContext>> {
        Ok(AxumRouter::new())
    }

    /// 在 Loco router 构建完成后调用此函数。
    /// 此函数允许用户配置自定义的 Axum 逻辑，例如 layers，
    /// 这些逻辑与 Axum 兼容。
    ///
    /// # 错误处理
    /// Axum router 的错误情况
    async fn after_routes(router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        Ok(router)
    }

    /// 提供初始化器列表。
    /// 初始化器可用于无缝地向应用程序添加功能或初始化某些方面。
    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![])
    }

    /// 提供中间件列表。
    #[must_use]
    fn middlewares(ctx: &AppContext) -> Vec<Box<dyn MiddlewareLayer>> {
        middleware::default_middleware_stack(ctx)
    }

    /// 在应用程序运行之前调用此函数。
    /// 您现在可以在应用程序运行前编写一些自定义的资源加载或其它操作。
    async fn before_run(_app_context: &AppContext) -> Result<()> {
        Ok(())
    }

    /// 定义应用程序的路由配置。
    fn routes(_ctx: &AppContext) -> AppRoutes;

    /// 提供初始化后更改 Loco [`AppContext`] 的选项。
    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        Ok(ctx)
    }

    /// 使用提供的 [`Processor`] 和 [`AppContext`]
    /// 连接自定义工作进程到应用程序。
    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()>;

    /// 使用提供的 [`Tasks`] 对象注册自定义任务。
    fn register_tasks(tasks: &mut Tasks);

    /// 如需截断数据库，则执行此功能。用户应该实现此函数。
    /// 截断通过 [`crate::config::Database`] 中的 dangerously_truncate 设置控制,
    /// 默认为 false。截断在测试前需要清空数据库时非常有用。
    #[cfg(feature = "with-db")]
    async fn truncate(_ctx: &AppContext) -> Result<()>;

    /// 使用初始数据填充数据库。
    #[cfg(feature = "with-db")]
    async fn seed(_ctx: &AppContext, path: &Path) -> Result<()>;

    /// 当应用程序准备关闭时调用此函数。
    /// 此函数允许用户在应用程序完全停止前执行任何必要的清理或最终操作。
    async fn on_shutdown(_ctx: &AppContext) {}
}

/// An initializer.
/// Initializers should be kept in `src/initializers/`
///
/// Initializers can provide health checks by implementing the `check` method.
/// These checks will be run during the `cargo loco doctor` command to validate
/// the initializer's configuration and test its connections.
#[async_trait]
// <snip id="initializers-trait">
pub trait Initializer: Sync + Send {
    /// The initializer name or identifier
    fn name(&self) -> String;

    /// Occurs after the app's `before_run`.
    /// Use this to for one-time initializations, load caches, perform web
    /// hooks, etc.
    async fn before_run(&self, _app_context: &AppContext) -> Result<()> {
        Ok(())
    }

    /// Occurs after the app's `after_routes`.
    /// Use this to compose additional functionality and wire it into an Axum
    /// Router
    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        Ok(router)
    }

    /// Perform health checks for this initializer.
    /// This method is called during the doctor command to validate the
    /// initializer's configuration. Return `None` if no check is needed, or
    /// `Some(Check)` if a check should be performed.
    async fn check(&self, _app_context: &AppContext) -> Result<Option<crate::doctor::Check>> {
        Ok(None)
    }
}
// </snip>

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_cfg::app::get_app_context;

    struct TestService {
        name: String,
        value: i32,
    }

    #[derive(Clone)]
    struct CloneableTestService {
        name: String,
        value: i32,
    }

    #[test]
    fn test_extensions_insert_and_get() {
        // Setup
        let shared_store = SharedStore::default();

        shared_store.insert(42i32);
        assert_eq!(shared_store.get::<i32>().expect("Value should exist"), 42);

        let service = TestService {
            name: "test".to_string(),
            value: 100,
        };

        shared_store.insert(service);

        let service_ref_opt = shared_store.get_ref::<TestService>();
        assert!(service_ref_opt.is_some(), "Service ref should exist");
        if let Some(service_ref) = service_ref_opt {
            assert_eq!(service_ref.name, "test");
            assert_eq!(service_ref.value, 100);
            let name_clone = service_ref.name.clone();
            assert_eq!(name_clone, "test");
        } else {
            panic!("Should have gotten Some(service_ref)");
        }
    }

    #[test]
    fn test_extensions_get_without_clone() {
        let shared_store = SharedStore::default();

        let service = TestService {
            name: "test_direct".to_string(),
            value: 100,
        };
        shared_store.insert(service);

        let service_ref_opt = shared_store.get_ref::<TestService>();
        assert!(service_ref_opt.is_some(), "Service ref should exist");
        if let Some(service_ref) = service_ref_opt {
            assert_eq!(service_ref.name, "test_direct");
            assert_eq!(service_ref.value, 100);
        } else {
            panic!("Should have gotten Some(service_ref)");
        }

        let name_len_opt = shared_store.get_ref::<TestService>().map(|r| r.name.len());
        assert!(
            name_len_opt.is_some(),
            "Service ref should exist for len check"
        );
        assert_eq!(name_len_opt.unwrap(), 11);

        let value_opt = shared_store.get_ref::<TestService>().map(|r| r.value);
        assert!(
            value_opt.is_some(),
            "Service ref should exist for value check"
        );
        assert_eq!(value_opt.unwrap(), 100);
    }

    #[test]
    fn test_extensions_remove() {
        let shared_store = SharedStore::default();

        shared_store.insert(42i32);
        assert!(shared_store.contains::<i32>());
        assert_eq!(shared_store.remove::<i32>(), Some(42));
        assert!(!shared_store.contains::<i32>());
        assert_eq!(shared_store.remove::<i32>(), None);

        let service = TestService {
            name: "rem".to_string(),
            value: 50,
        };
        shared_store.insert(service);
        assert!(shared_store.contains::<TestService>());
        let removed_opt = shared_store.remove::<TestService>();
        assert!(removed_opt.is_some());
        if let Some(removed) = removed_opt {
            assert_eq!(removed.name, "rem");
            assert_eq!(removed.value, 50);
        } else {
            panic!("Removed option should be Some");
        }
        assert!(!shared_store.contains::<TestService>());
        assert!(shared_store.remove::<TestService>().is_none());
    }

    #[test]
    fn test_extensions_contains() {
        let shared_store = SharedStore::default();

        shared_store.insert(42i32);
        shared_store.insert(TestService {
            name: "contains".to_string(),
            value: 1,
        });

        assert!(shared_store.contains::<i32>());
        assert!(shared_store.contains::<TestService>());
        assert!(!shared_store.contains::<String>());
        assert!(!shared_store.contains::<CloneableTestService>());
    }

    #[test]
    fn test_extensions_get_cloned() {
        let shared_store = SharedStore::default();

        shared_store.insert(42i32);
        assert_eq!(shared_store.get::<i32>(), Some(42));
        assert!(shared_store.contains::<i32>());

        let service = CloneableTestService {
            name: "cloned_test".to_string(),
            value: 200,
        };
        shared_store.insert(service.clone());

        let service_clone_opt = shared_store.get::<CloneableTestService>();
        assert!(service_clone_opt.is_some(), "Cloned service should exist");
        if let Some(ref service_clone) = service_clone_opt {
            assert_eq!(service_clone.name, "cloned_test");
            assert_eq!(service_clone.value, 200);
        } else {
            panic!("Should have gotten Some(service_clone)");
        }

        assert!(shared_store.contains::<CloneableTestService>());
        let original_ref_opt = shared_store.get_ref::<CloneableTestService>();
        assert!(original_ref_opt.is_some(), "Original ref should exist");
        if let Some(original_ref) = original_ref_opt {
            assert_eq!(original_ref.name, "cloned_test");
            assert_eq!(original_ref.value, 200);
        } else {
            panic!("Should have gotten Some(original_ref)");
        }

        assert_eq!(shared_store.get::<String>(), None);
        assert!(shared_store.get::<CloneableTestService>().is_some());
        // The following line correctly fails to compile because TestService
        // doesn't impl Clone, which is required by the `get` method.
        // let non_existent_clone = shared_store.get::<TestService>();
    }

    #[tokio::test]
    async fn test_app_context_extensions() {
        let ctx = get_app_context().await;

        let service_cloneable = CloneableTestService {
            name: "app_context_test_cloneable".to_string(),
            value: 42,
        };
        ctx.shared_store.insert(service_cloneable.clone());

        let ref_opt = ctx.shared_store.get_ref::<CloneableTestService>();
        assert!(ref_opt.is_some(), "Cloneable service ref should exist");
        if let Some(service_ref) = ref_opt {
            assert_eq!(service_ref.name, "app_context_test_cloneable");
            assert_eq!(service_ref.value, 42);
        } else {
            panic!("Should have gotten Some(service_ref)");
        }

        let clone_opt = ctx.shared_store.get::<CloneableTestService>();
        assert!(clone_opt.is_some(), "Should get cloned service");
        if let Some(service_clone) = clone_opt {
            assert_eq!(service_clone.name, "app_context_test_cloneable");
            assert_eq!(service_clone.value, 42);
        } else {
            panic!("Should have gotten Some(service_clone)");
        }

        assert!(ctx.shared_store.contains::<CloneableTestService>());
        assert!(!ctx.shared_store.contains::<String>());

        let removed_cloneable_opt = ctx.shared_store.remove::<CloneableTestService>();
        assert!(removed_cloneable_opt.is_some());
        if let Some(removed) = removed_cloneable_opt {
            assert_eq!(removed.name, "app_context_test_cloneable");
            assert_eq!(removed.value, 42);
        } else {
            panic!("Removed cloneable option should be Some");
        }
        assert!(!ctx.shared_store.contains::<CloneableTestService>());

        let service_non_cloneable = TestService {
            name: "app_context_test_non_cloneable".to_string(),
            value: 99,
        };
        ctx.shared_store.insert(service_non_cloneable);

        let non_clone_ref_opt = ctx.shared_store.get_ref::<TestService>();
        assert!(
            non_clone_ref_opt.is_some(),
            "Non-cloneable service ref should exist"
        );
        if let Some(service_ref) = non_clone_ref_opt {
            assert_eq!(service_ref.name, "app_context_test_non_cloneable");
            assert_eq!(service_ref.value, 99);
        } else {
            panic!("Should have gotten Some(service_ref)");
        }

        assert!(ctx.shared_store.contains::<TestService>());

        let removed_non_cloneable_opt = ctx.shared_store.remove::<TestService>();
        assert!(removed_non_cloneable_opt.is_some());
        if let Some(removed) = removed_non_cloneable_opt {
            assert_eq!(removed.name, "app_context_test_non_cloneable");
            assert_eq!(removed.value, 99);
        } else {
            panic!("Removed non-cloneable option should be Some");
        }
        assert!(!ctx.shared_store.contains::<TestService>());
    }
}
