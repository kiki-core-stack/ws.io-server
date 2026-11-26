mod admin_frontend;
mod backend;
mod management_backend;
mod ops_workers_ts;
mod services_backend;

pub(crate) use self::{
    admin_frontend::ADMIN_FRONTEND,
    backend::BACKEND,
    management_backend::MANAGEMENT_BACKEND,
    ops_workers_ts::OPS_WORKERS_TS,
    services_backend::SERVICES_BACKEND,
};
