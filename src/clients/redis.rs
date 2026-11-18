use std::{
    env,
    sync::OnceLock,
};

use anyhow::{
    Context,
    Result,
    anyhow,
};
use redis::{
    Client,
    aio::ConnectionManager,
};

// Constants/Statics
static REDIS_CONNECTION_MANAGER: OnceLock<ConnectionManager> = OnceLock::new();

// Functions
#[expect(dead_code, reason = "base infrastructure for downstream services")]
pub(crate) fn get_redis() -> Result<ConnectionManager> {
    REDIS_CONNECTION_MANAGER
        .get()
        .ok_or_else(|| anyhow!("redis connection manager is not initialized"))
        .cloned()
}

pub(crate) async fn init_redis() -> Result<()> {
    let url = env::var("REDIS_URL").context("REDIS_URL environment variable is not set")?;
    let redis_client = Client::open(url).context("failed to create redis client")?;
    let connection_manager = redis_client
        .get_connection_manager()
        .await
        .context("failed to connect redis connection manager")?;

    REDIS_CONNECTION_MANAGER
        .set(connection_manager)
        .map_err(|_| anyhow!("redis connection manager is already initialized"))?;

    tracing::info!("redis initialized");
    Ok(())
}
