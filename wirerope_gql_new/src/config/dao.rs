use std::env;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct DaoSetting {
    // mysql
    pub db_main_addr: String,
    pub db_replica_addr: String,
    // redis 
    pub redis_host: Option<String>,
    pub redis_port: Option<String>,
    pub redis_usr: Option<String>, 
    pub redis_pwd: Option<String>
}

impl DaoSetting {
    pub fn new() -> Self {
        DaoSetting {
            db_main_addr: env::var("DB_MAIN_ADDR").unwrap_or("mysql://unknown".to_owned()),
            db_replica_addr: env::var("DB_REPLICA_ADDR").unwrap_or("".to_owned()),
            redis_port: env::var("REDIS_PORT").ok(),
            redis_host: env::var("REDIS_HOST").ok(),
            redis_usr: env::var("REDIS_USR").ok(),
            redis_pwd: env::var("REDIS_PWD").ok()
        }
    }
}