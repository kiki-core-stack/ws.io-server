use std::{
    env,
    sync::LazyLock,
};

use anyhow::{
    Context,
    Result,
};
use axum::{
    Router,
    serve,
};
use dotenvy::from_filename;
use kikiutils::{
    global_allocator::GlobalAllocator,
    logger::{
        init_logger,
        options::LoggerInitOptions,
    },
    signal::wait_for_shutdown_signal,
    task::manager::TaskManager,
};
use tokio::{
    net::TcpListener,
    select,
};
use tokio_util::sync::CancellationToken;

mod clients;
mod collections;
mod libs;
mod namespaces;
mod redis_scripts;
mod storages;
mod utils;
mod ws_io;

use self::{
    clients::init_clients,
    libs::auth::{
        realm::init_auth_origins,
        service::init_service_auth_keys,
    },
    ws_io::{
        WS_IO_SERVER,
        init_ws_io_and_get_tower_layer,
    },
};

// Constants/Statics
static APP_SHUTDOWN_TOKEN: LazyLock<CancellationToken> = LazyLock::new(CancellationToken::new);
static APP_TASK_MANAGER: LazyLock<TaskManager> = LazyLock::new(TaskManager::new);

#[global_allocator]
static GLOBAL: GlobalAllocator = GlobalAllocator;

// Functions
fn load_env_by_mode() -> Result<()> {
    #[cfg(debug_assertions)]
    from_filename("./.env.development.local").context("failed to load development environment file")?;

    #[cfg(not(debug_assertions))]
    from_filename("./.env.production.local").context("failed to load production environment file")?;

    Ok(())
}

fn shutdown() {
    APP_SHUTDOWN_TOKEN.cancel();
}

async fn start_server(cancel_token: CancellationToken) -> Result<()> {
    let server_host = env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_owned());
    let server_port = env::var("SERVER_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8080);

    let server_addr = format!("{server_host}:{server_port}");
    let app = Router::new().layer(init_ws_io_and_get_tower_layer());
    let listener = TcpListener::bind((server_host, server_port)).await?;

    tracing::info!(component = "server", addr = %server_addr, "listening");

    serve(listener, app)
        .with_graceful_shutdown(async move { cancel_token.cancelled().await })
        .await?;

    tracing::info!(component = "server", "stopped");
    Ok(())
}

// Main
#[tokio::main]
async fn main() -> Result<()> {
    // 載入環境變數檔
    load_env_by_mode()?;

    // 初始化 logger
    let _logger_guard = init_logger(LoggerInitOptions::from_env()?).context("failed to initialize logger")?;
    tracing::info!(component = "env", "loaded");

    // 載入 auth 設定，缺少時直接中止
    init_auth_origins();
    init_service_auth_keys();

    // 初始化 API、MongoDB、Redis 的連線
    init_clients().await?;

    // 啟動伺服器並註冊關閉流程
    APP_TASK_MANAGER.spawn_with_token(start_server);
    APP_TASK_MANAGER.spawn(async {
        select! {
            () = APP_SHUTDOWN_TOKEN.cancelled() => {},
            _ = wait_for_shutdown_signal() => tracing::info!(component = "shutdown", "signal received"),
        }

        shutdown();
    });

    // 等待關閉訊號
    APP_SHUTDOWN_TOKEN.cancelled().await;
    tracing::info!(component = "shutdown", "starting");

    // 關閉連線並等待背景工作結束
    WS_IO_SERVER.shutdown().await;
    APP_TASK_MANAGER.cancel_and_join_existing().await;

    tracing::info!(component = "shutdown", "complete");
    Ok(())
}
