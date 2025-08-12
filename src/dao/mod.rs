use std::{error::Error, time::Duration};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use crate::dao::seaorm_mysql::AppState;
pub mod seaorm_mysql;

const DEFAULT_SQL_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_SQL_MIN_CONNECTIONS: u32 = 1;
const DEFAULT_SQL_CONNECT_TIMEOUT: u64 = 5;

pub async fn init_sql_connection(
    sql_addr: &str,
    log_level: log::LevelFilter,
) -> Result<DatabaseConnection, Box<dyn Error>> {
    let mut conn_core_opt = ConnectOptions::new(sql_addr);
    conn_core_opt
        .max_connections(DEFAULT_SQL_MAX_CONNECTIONS)
        .min_connections(DEFAULT_SQL_MIN_CONNECTIONS)
        .connect_timeout(Duration::from_secs(DEFAULT_SQL_CONNECT_TIMEOUT))
        .sqlx_logging(true)
        .sqlx_logging_level(log_level);
    // conn_core_opt
    let conn_core = Database::connect(conn_core_opt).await.expect(
        format!(
            "Connect to db=<{}> failed! Check the db connectable first!",
            sql_addr
        )
        .as_str(),
    );
    Ok(conn_core)
}
