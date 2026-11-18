use std::{
    env,
    sync::OnceLock,
};

use anyhow::{
    Context,
    Result,
    anyhow,
};
use mongodb::{
    Client,
    Database,
};

// Constants/Statics
static DB: OnceLock<Database> = OnceLock::new();

// Functions
pub(crate) fn get_db() -> Result<&'static Database> {
    DB.get().ok_or_else(|| anyhow!("mongodb database is not initialized"))
}

pub(crate) async fn init_mongodb() -> Result<()> {
    let uri = env::var("MONGODB_URI").context("MONGODB_URI environment variable is not set")?;
    let client = Client::with_uri_str(uri)
        .await
        .context("failed to connect to mongodb")?;

    let db = client
        .default_database()
        .ok_or_else(|| anyhow!("mongodb uri does not include a default database"))?;

    DB.set(db)
        .map_err(|_| anyhow!("mongodb database is already initialized"))?;

    tracing::info!("mongodb initialized");
    Ok(())
}
