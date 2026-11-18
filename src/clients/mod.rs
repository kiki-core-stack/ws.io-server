use anyhow::{
    Context,
    Result,
};

pub(crate) mod apis;
pub(crate) mod mongodb;
pub(crate) mod redis;

use self::{
    apis::init_apis,
    mongodb::init_mongodb,
    redis::init_redis,
};

pub(crate) async fn init_clients() -> Result<()> {
    init_apis().context("failed to initialize API clients")?;
    init_mongodb().await.context("failed to initialize mongodb")?;
    init_redis().await.context("failed to initialize redis")?;
    Ok(())
}
