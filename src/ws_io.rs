use std::sync::LazyLock;

use wsio_server::{
    WsIoServer,
    request_adapters::tower::layer::WsIoServerLayer,
};

use crate::namespaces;

// Constants/Statics
pub(crate) static WS_IO_SERVER: LazyLock<WsIoServer> = LazyLock::new(|| {
    #[allow(clippy::unwrap_used)]
    WsIoServer::builder().build().unwrap()
});

// Functions
pub(crate) fn init_ws_io_and_get_tower_layer() -> WsIoServerLayer {
    namespaces::ADMIN_FRONTEND.path();
    namespaces::MANAGEMENT_BACKEND.path();
    tracing::info!(
        component = "ws.io",
        namespace_count = WS_IO_SERVER.namespace_count(),
        "initialized"
    );

    WS_IO_SERVER.layer()
}
