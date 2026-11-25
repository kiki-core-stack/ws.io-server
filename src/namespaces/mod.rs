mod admin_frontend;
mod management_backend;
mod ops_workers_ts;

pub(crate) use self::{
    admin_frontend::ADMIN_FRONTEND,
    management_backend::MANAGEMENT_BACKEND,
    ops_workers_ts::OPS_WORKERS_TS,
};
