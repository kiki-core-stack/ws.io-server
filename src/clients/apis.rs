use std::{
    env,
    sync::OnceLock,
};

use anyhow::{
    Context,
    Result,
    anyhow,
    ensure,
};
use http::{
    HeaderMap,
    HeaderValue,
};
use kcs_project_pack::api::client::ApiClient;

// Constants/Statics
pub(crate) static MANAGEMENT_BACKEND_API_CLIENT: OnceLock<ApiClient> = OnceLock::new();

// Functions
fn create_api_client(base_url_env_name: &str, introspection_token_env_name: &str) -> Result<ApiClient> {
    let base_url =
        env::var(base_url_env_name).with_context(|| format!("{base_url_env_name} environment variable is not set"))?;

    let introspection_token = env::var(introspection_token_env_name)
        .with_context(|| format!("{introspection_token_env_name} environment variable is not set"))?;

    ensure!(
        introspection_token.len() >= 32,
        "{introspection_token_env_name} must contain at least 32 bytes"
    );

    let mut default_headers = HeaderMap::new();
    default_headers.insert(
        "x-ws-io-introspection-token",
        HeaderValue::from_str(&introspection_token)
            .with_context(|| format!("{introspection_token_env_name} is not a valid HTTP header value"))?,
    );

    ApiClient::with_default_headers(&base_url, default_headers)
}

pub(crate) fn init_apis() -> Result<()> {
    MANAGEMENT_BACKEND_API_CLIENT
        .set(create_api_client(
            "MANAGEMENT_BACKEND_API_BASE_URL",
            "MANAGEMENT_BACKEND_API_INTROSPECTION_TOKEN",
        )?)
        .map_err(|_| anyhow!("management backend API client is already initialized"))?;

    Ok(())
}
