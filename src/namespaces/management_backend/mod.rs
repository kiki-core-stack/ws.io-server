use std::sync::{
    Arc,
    LazyLock,
};

use anyhow::Result;
use kcs_project_pack::constants::ws_io_event_names;
use wsio_server::{
    connection::WsIoServerConnection,
    namespace::WsIoServerNamespace,
};

use crate::{
    libs::auth::{
        frontend::handle_auth_sessions_revoked,
        service::{
            InternalService,
            authorize_service_connection,
        },
    },
    ws_io::WS_IO_SERVER,
};

// Constants/Statics
pub(crate) static MANAGEMENT_BACKEND: LazyLock<Arc<WsIoServerNamespace>> = LazyLock::new(|| {
    #[allow(clippy::unwrap_used)]
    WS_IO_SERVER
        .new_namespace_builder("/management-backend")
        .on_connect(on_connect)
        .on_ready(on_ready)
        .with_init_response(|connection, auth_key| {
            authorize_service_connection(connection, auth_key, InternalService::ManagementBackend)
        })
        .register()
        .unwrap()
});

// Functions
async fn on_connect(connection: Arc<WsIoServerConnection>) -> Result<()> {
    tracing::debug!(namespace = "management-backend", connection_id = %connection.id(), "connected");

    // Register events
    connection.on(
        ws_io_event_names::to_server::AUTH_SESSIONS_REVOKED,
        handle_auth_sessions_revoked,
    );

    Ok(())
}

async fn on_ready(_: Arc<WsIoServerConnection>) -> Result<()> {
    Ok(())
}
