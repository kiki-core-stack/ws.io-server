use std::sync::{
    Arc,
    LazyLock,
};

use anyhow::Result;
use wsio_server::{
    connection::WsIoServerConnection,
    namespace::WsIoServerNamespace,
};

use crate::{
    libs::auth::service::{
        InternalService,
        authorize_service_connection,
    },
    ws_io::WS_IO_SERVER,
};

// Constants/Statics
pub(crate) static BACKEND: LazyLock<Arc<WsIoServerNamespace>> = LazyLock::new(|| {
    #[allow(clippy::unwrap_used)]
    WS_IO_SERVER
        .new_namespace_builder("/backend")
        .on_connect(on_connect)
        .with_init_response(|connection, auth_key| {
            authorize_service_connection(connection, auth_key, InternalService::Backend)
        })
        .register()
        .unwrap()
});

// Functions
async fn on_connect(connection: Arc<WsIoServerConnection>) -> Result<()> {
    tracing::debug!(namespace = "backend", connection_id = %connection.id(), "connected");
    Ok(())
}
