use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::{
    Context,
    Result,
    bail,
};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use tokio::time::sleep;
use wsio_server::{
    connection::WsIoServerConnection,
    core::traits::task::spawner::TaskSpawner,
};

use super::realm::{
    AuthRealm,
    validate_auth_session,
};
use crate::utils::http::{
    extract_client_ip_from_request_headers,
    extract_host_cookie_from_request_headers,
};

// Constants/Statics
const SESSION_REVALIDATION_INTERVAL: Duration = Duration::from_mins(5);

// Structs

/// Auth realm name, principal id and the ids of the revoked sessions.
#[derive(Debug, Deserialize)]
pub(crate) struct AuthSessionsRevokedData(AuthRealm, String, Vec<String>);

pub(crate) struct AuthenticatedFrontendSession {
    client_ip: String,
    pub(crate) principal_id: ObjectId,
    pub(crate) session_id: String,
    token: String,
}

// Functions

/// The handshake is a GET request, which the backend's origin check of write
/// requests does not cover, so the `Origin` is checked here. `None` means the
/// connection carries no session cookie; whether guests are allowed is up to
/// the namespace.
pub(crate) async fn authenticate_frontend_connection(
    connection: &Arc<WsIoServerConnection>,
    realm: AuthRealm,
) -> Result<Option<AuthenticatedFrontendSession>> {
    let origin = connection
        .headers()
        .get("origin")
        .and_then(|origin| origin.to_str().ok());

    if origin != Some(realm.origin()) {
        connection.disconnect().await;
        bail!("origin is not allowed");
    }

    let Some(token) = extract_host_cookie_from_request_headers(connection.headers(), realm.cookie_name()) else {
        return Ok(None);
    };

    let Some(client_ip) = extract_client_ip_from_request_headers(connection.headers()) else {
        connection.disconnect().await;
        bail!("client IP is missing");
    };

    let session = match validate_auth_session(realm, &token, client_ip).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            connection.disconnect().await;
            bail!("auth session is invalid");
        },
        Err(error) => {
            // wsio-server does not log middleware errors, so the reason is recorded here.
            tracing::warn!(
                component = "auth",
                realm = realm.as_str(),
                connection_id = %connection.id(),
                error = format!("{error:#}"),
                "failed to validate auth session"
            );

            return Err(error);
        },
    };

    Ok(Some(AuthenticatedFrontendSession {
        client_ip: client_ip.to_owned(),
        principal_id: ObjectId::parse_str(session.principal_id)
            .context("invalid principal id returned by authentication service")?,
        session_id: session.session_id,
        token,
    }))
}

pub(crate) fn extract_authenticated_frontend_session_from_connection(
    connection: &Arc<WsIoServerConnection>,
) -> Option<Arc<AuthenticatedFrontendSession>> {
    connection.extensions().get::<AuthenticatedFrontendSession>()
}

/// A device whose session was just replaced (e.g. by logging in again) is
/// notified as well, as its previous session is among the revoked ones.
pub(crate) async fn handle_auth_sessions_revoked(
    _: Arc<WsIoServerConnection>,
    data: Arc<AuthSessionsRevokedData>,
) -> Result<()> {
    let AuthSessionsRevokedData(realm, principal_id, session_ids) = data.as_ref();
    if session_ids.is_empty() {
        return Ok(());
    }

    let room_names = session_ids.iter().map(|session_id| format!("authSession:{session_id}"));
    let broadcast_operator = realm.ws_io_namespace().to(room_names);
    if let Err(error) = broadcast_operator
        .clone()
        .emit::<()>(realm.ws_io_force_logout_event_name(), None)
        .await
    {
        tracing::warn!(
            component = "auth",
            event = "auth_sessions_revoked",
            realm = realm.as_str(),
            principal_id = %principal_id,
            error = %error,
            "failed to emit force logout event"
        );
    }

    if let Err(error) = broadcast_operator.disconnect().await {
        tracing::warn!(
            component = "auth",
            event = "auth_sessions_revoked",
            realm = realm.as_str(),
            principal_id = %principal_id,
            error = %error,
            "failed to disconnect revoked sessions"
        );
    }

    Ok(())
}

/// For namespaces that only accept logged-in connections.
pub(crate) async fn require_authenticated_frontend_connection(
    connection: &Arc<WsIoServerConnection>,
    realm: AuthRealm,
) -> Result<()> {
    let Some(session) = authenticate_frontend_connection(connection, realm).await? else {
        connection.disconnect().await;
        bail!("unauthorized");
    };

    connection.extensions().insert(session);
    Ok(())
}

/// Joins the rooms of the session stored by the middleware and keeps
/// revalidating it while the connection stays open.
pub(crate) fn start_authenticated_frontend_session(
    connection: &Arc<WsIoServerConnection>,
    realm: AuthRealm,
) -> Result<()> {
    let session = extract_authenticated_frontend_session_from_connection(connection)
        .context("auth session extension is missing")?;

    tracing::debug!(
        component = "auth",
        realm = realm.as_str(),
        connection_id = %connection.id(),
        principal_id = %session.principal_id,
        session_id = %session.session_id,
        "connected"
    );

    start_frontend_session_revalidation(connection, realm, &session);
    connection.join([
        format!("{}:{}", realm.as_str(), session.principal_id),
        format!("authSession:{}", session.session_id),
    ]);

    Ok(())
}

/// Revocations pushed by the backends only make the disconnect immediate; this
/// is what guarantees a connection never outlives its session.
fn start_frontend_session_revalidation(
    connection: &Arc<WsIoServerConnection>,
    realm: AuthRealm,
    session: &Arc<AuthenticatedFrontendSession>,
) {
    let connection_clone = Arc::clone(connection);
    let session = Arc::clone(session);
    connection.spawn_task(async move {
        loop {
            sleep(SESSION_REVALIDATION_INTERVAL).await;
            match validate_auth_session(realm, &session.token, &session.client_ip).await {
                Ok(Some(validated_session)) if validated_session.session_id == session.session_id => {},
                Ok(_) => {
                    let _ = connection_clone
                        .emit::<()>(realm.ws_io_force_logout_event_name(), None)
                        .await;

                    connection_clone.disconnect().await;
                    return Ok(());
                },
                Err(error) => {
                    tracing::warn!(
                        component = "auth",
                        realm = realm.as_str(),
                        connection_id = %connection_clone.id(),
                        error = format!("{error:#}"),
                        "failed to revalidate auth session"
                    );
                },
            }
        }
    });
}
