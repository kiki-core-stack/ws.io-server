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
    libs::auth::{
        frontend::{
            require_authenticated_frontend_connection,
            start_authenticated_frontend_session,
        },
        realm::AuthRealm,
    },
    ws_io::WS_IO_SERVER,
};

// Constants/Statics
pub(crate) static ADMIN_FRONTEND: LazyLock<Arc<WsIoServerNamespace>> = LazyLock::new(|| {
    #[allow(clippy::unwrap_used)]
    WS_IO_SERVER
        .new_namespace_builder("/admin-frontend")
        .on_connect(on_connect)
        .with_middleware(middleware)
        .register()
        .unwrap()
});

// Functions
async fn middleware(connection: Arc<WsIoServerConnection>) -> Result<()> {
    require_authenticated_frontend_connection(&connection, AuthRealm::Admin).await
}

async fn on_connect(connection: Arc<WsIoServerConnection>) -> Result<()> {
    start_authenticated_frontend_session(&connection, AuthRealm::Admin)
}
