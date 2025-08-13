#![allow(unused)]
mod config;
mod dao;
pub mod error;
mod metrics;
mod middleware;
mod services;
pub mod util;
use crate::{dao::init_sql_connection, error::DError};
use actix_web::{web, App, HttpServer};
use config::{log::LogConfig, RuntimeSetting};
use dao::seaorm_mysql::AppState;
use dotenv::dotenv;
use log::LevelFilter;
use redis::{aio::MultiplexedConnection, AsyncConnectionConfig, Client};
use sea_orm::{ConnectOptions, Database};
use std::{str::FromStr, time::Duration};
use tracing_actix_web::TracingLogger;

#[cfg(all(feature = "metrics"))]
use metrics::{RequestMetrics, RequestTracing};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ------------
    // logger
    let f = config::log::LogConfig::from_env().map_err(|e| {
        tracing::error!("Failed to set up logger: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to load logger config")
    })?;
    LogConfig::init_logger(&f).map_err(|e| {
        tracing::error!("Failed to init logger: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to init logger")
    })?;
    let log_level = LevelFilter::from_str(&f.level.clone()).unwrap_or(LevelFilter::Info);
    // ------------

    // ------------
    //*.env */
    tracing::info!("<AppState> connecting...");
    let o = dotenv().ok().expect(
        "[Attention]Not found .env config in root path!Can't execute server by default values!",
    );
    tracing::info!("Load env file file: {:?}", o);
    let rt_setting: RuntimeSetting = RuntimeSetting::default();
    // ------------

    // ------------
    // metrics
    #[cfg(all(feature = "metrics", not(debug_assertions)))]
    if let Some(metrics_config) = &rt_setting.metrics {
        metrics::self_host::setup_metrics_tracing(&rt_setting.base.svr_name, metrics_config)
            .map_err(|e| {
                tracing::error!("Failed to set up metrics_tracing: {:?}", e);
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to set up metrics_tracing",
                )
            })?;
    } else {
        tracing::warn!("Not found metrics config, pass init!");
    }
    // ------------

    // ------------

    //ctx
    // Create response context
    let state: AppState = {
        // main dao
        let db_main_connection = {
            let con = init_sql_connection(&rt_setting.dao.db_main_addr, log_level)
                .await
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Connect to sql failed! Err:{:?}", e),
                    )
                })?;
            con
        };
        // replica dao
        let db_replica_connection = {
            if rt_setting.dao.db_replica_addr.len() > 0 {
            let con = init_sql_connection(&rt_setting.dao.db_replica_addr, log_level)
                .await
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Connect to sql failed! Err:{:?}", e),
                    )
                })?;
                Some(con)
            } else {
                None
            }
        };
        AppState {
            rtx_setting: rt_setting.to_owned(),
            conn: db_main_connection,
            conn_r: db_replica_connection,
            redis_pool: None,
        }
    };
    // services
    let svr = HttpServer::new(move || {
        let conn_graph = state.conn.clone();
        let state_host = &state.rtx_setting.base.host.to_owned();
        let app = App::new()
            .wrap(TracingLogger::default())
            .wrap(RequestMetrics::default())
            .wrap(RequestTracing::new())
            .app_data(web::QueryConfig::default().error_handler(|req, _err| {
                tracing::warn!("[error] on <GLOBAL> deserialize Query");
                DError::Custom(error::LogicErr::ParamsError(req.to_string())).into()
            }))
            .app_data(web::JsonConfig::default().error_handler(|body_err, _req| {
                tracing::error!("[error] on <GLOBAL> deserialize json-body");
                DError::Custom(error::LogicErr::ParamsError(body_err.to_string())).into()
            })) // All GraphQL
            .configure(move |c| {
                use actix_web::web::Data;
                use crate::services::graphql::{
                    graphql_index, graphql_json, graphql_playground, GRAPHQL_BUILD_CTX,
                };
                use entity_graphql;
                // DEPTH_LIMIT
                // COMPLEXITY_LIMIT
                tracing::info!("graphql schema init success");
                tracing::info!("Visit GraphQL Playground at {:?}", state_host);
                let mut builder = seaography::Builder::new(&GRAPHQL_BUILD_CTX, conn_graph.clone());
                builder = entity_graphql::register_entity_modules(builder);
                builder = entity_graphql::register_active_enums(builder);
                let schema = builder
                    .schema_builder()
                    .data(conn_graph)
                    .finish()
                    .expect("graphql schema init failed");
                c.app_data(Data::new(schema.clone()));
                c.service(
                    web::resource("/health")
                        .guard(actix_web::guard::Get())
                        .to(|| async { actix_web::HttpResponse::Ok().body("ok") }),
                );
                c.service(
                    web::resource("/gql")
                        .guard(actix_web::guard::Post())
                        .to(graphql_index),
                );
                c.service(
                    web::resource("/gql")
                        .guard(actix_web::guard::Get())
                        .to(graphql_playground),
                );
                c.service(
                    web::resource("/gql/json")
                        .guard(actix_web::guard::Post())
                        .to(graphql_json),
                );
            });
        app
    });

    svr
        .bind((rt_setting.base.host.as_str(), rt_setting.base.port))?
        .run()
        .await
    
}
