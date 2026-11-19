use std::{
    env,
    sync::{
        Arc,
        LazyLock,
    },
};

use anyhow::{
    Context,
    Result,
};
use kcs_project_pack::{
    api::client::{
        ApiClient,
        ApiResponseError,
    },
    constants::ws_io_event_names,
};
use serde::{
    Deserialize,
    Serialize,
};
use wsio_server::namespace::WsIoServerNamespace;

use crate::{
    clients::apis::MANAGEMENT_BACKEND_API_CLIENT,
    namespaces,
};

// Constants/Statics
static ADMIN_AUTH_ORIGIN: LazyLock<String> = LazyLock::new(|| load_auth_origin("ADMIN_AUTH_ORIGIN"));

// Enums

/// Names match the auth realms defined by the backends.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AuthRealm {
    Admin,
}

impl AuthRealm {
    // Private methods

    /// Client of the backend that owns the realm's sessions.
    fn api_client(self) -> Result<&'static ApiClient> {
        match self {
            Self::Admin => MANAGEMENT_BACKEND_API_CLIENT
                .get()
                .context("management backend API client is not initialized"),
        }
    }

    // Public methods
    pub(super) fn origin(self) -> &'static str {
        match self {
            Self::Admin => &ADMIN_AUTH_ORIGIN,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
        }
    }

    /// Name of the session cookie, without its `__Host-` prefix.
    pub(super) const fn cookie_name(self) -> &'static str {
        match self {
            Self::Admin => "admin-session",
        }
    }

    pub(super) const fn ws_io_force_logout_event_name(self) -> &'static str {
        match self {
            Self::Admin => ws_io_event_names::to_admin_frontend::FORCE_LOGOUT,
        }
    }

    pub(super) fn ws_io_namespace(self) -> &'static Arc<WsIoServerNamespace> {
        match self {
            Self::Admin => &namespaces::ADMIN_FRONTEND,
        }
    }
}

// Structs
#[derive(Serialize)]
struct AuthSessionValidationRequest<'a> {
    ip: &'a str,
    token: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ValidatedAuthSession {
    pub(super) principal_id: String,
    pub(super) session_id: String,
}

// Functions

/// Loads the origin of every realm, so that a missing one fails at startup.
pub(crate) fn init_auth_origins() {
    LazyLock::force(&ADMIN_AUTH_ORIGIN);
}

fn load_auth_origin(env_name: &str) -> String {
    env::var(env_name).unwrap_or_else(|_| panic!("{env_name} is not set"))
}

/// `None` means the backend rejected the session (any 4xx); other failures are
/// errors and say nothing about the session.
pub(super) async fn validate_auth_session(
    realm: AuthRealm,
    token: &str,
    ip: &str,
) -> Result<Option<ValidatedAuthSession>> {
    match realm
        .api_client()?
        .post(
            &format!("/api/internal/ws-io/auth/{}/validate", realm.as_str()),
            Some(&AuthSessionValidationRequest { ip, token }),
        )
        .await
    {
        Ok(session) => Ok(Some(session)),
        Err(error) if ApiResponseError::is_client_error(&error) => Ok(None),
        Err(error) => Err(error),
    }
}
