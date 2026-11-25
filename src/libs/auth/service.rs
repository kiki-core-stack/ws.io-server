use std::{
    env,
    sync::{
        Arc,
        LazyLock,
    },
};

use anyhow::{
    Result,
    bail,
};
use subtle::ConstantTimeEq;
use wsio_server::connection::WsIoServerConnection;

// Constants/Statics
static MANAGEMENT_BACKEND_AUTH_KEY: LazyLock<String> = LazyLock::new(|| load_auth_key("MANAGEMENT_BACKEND_AUTH_KEY"));
static OPS_WORKERS_TS_AUTH_KEY: LazyLock<String> = LazyLock::new(|| load_auth_key("OPS_WORKERS_TS_AUTH_KEY"));

// Enums

/// Backend services connecting with their own auth key.
#[derive(Debug, Clone, Copy)]
pub(crate) enum InternalService {
    ManagementBackend,
    OpsWorkersTs,
}

impl InternalService {
    // Private methods
    fn auth_key(self) -> &'static str {
        match self {
            Self::ManagementBackend => &MANAGEMENT_BACKEND_AUTH_KEY,
            Self::OpsWorkersTs => &OPS_WORKERS_TS_AUTH_KEY,
        }
    }

    // Public methods
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ManagementBackend => "management-backend",
            Self::OpsWorkersTs => "ops-workers-ts",
        }
    }
}

// Functions
pub(crate) async fn authorize_service_connection(
    connection: Arc<WsIoServerConnection>,
    auth_key: Option<String>,
    service: InternalService,
) -> Result<()> {
    if auth_key.is_some_and(|auth_key| bool::from(auth_key.as_bytes().ct_eq(service.auth_key().as_bytes()))) {
        return Ok(());
    }

    connection.disconnect().await;
    bail!("{} authorization failed", service.as_str());
}

/// Loads every auth key, so that a missing one fails at startup.
pub(crate) fn init_service_auth_keys() {
    LazyLock::force(&MANAGEMENT_BACKEND_AUTH_KEY);
    LazyLock::force(&OPS_WORKERS_TS_AUTH_KEY);
}

fn load_auth_key(env_name: &str) -> String {
    env::var(env_name).unwrap_or_else(|_| panic!("{env_name} is not set"))
}
