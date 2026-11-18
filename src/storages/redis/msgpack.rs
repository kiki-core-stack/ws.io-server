use anyhow::{
    Context,
    Result,
};
use redis::AsyncCommands;
use serde::de::DeserializeOwned;

use crate::clients::redis::get_redis;

// Structs
pub(crate) struct RedisMsgpackStorage;

impl RedisMsgpackStorage {
    pub(crate) async fn get_item<T: DeserializeOwned>(key: &str) -> Result<Option<T>> {
        let raw_value: Option<Vec<u8>> = get_redis()?.get(key).await?;
        raw_value
            .map(|raw_value| rmp_serde::from_slice(&raw_value).context("failed to decode redis msgpack value"))
            .transpose()
    }
}
