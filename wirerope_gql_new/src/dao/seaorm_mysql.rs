#[allow(dead_code)]
use redis::aio::MultiplexedConnection;
use sea_orm::{Database, DatabaseConnection};
use crate::{config::RuntimeSetting, error::DError};

#[derive(Clone)]
pub struct AppState {
    pub rtx_setting: RuntimeSetting,
    pub conn: DatabaseConnection,
    pub conn_r: Option<DatabaseConnection>,
    pub redis_pool: Option<MultiplexedConnection>,
    
}
