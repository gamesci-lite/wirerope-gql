mod custom_query;
mod query_root;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Result;
use async_graphql_actix_web::GraphQLRequest;
use async_graphql_actix_web::GraphQLResponse;
use seaography::async_graphql::dynamic::*;
use seaography::async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use seaography::BuilderContext;
use serde_json::json;

use crate::error::DResult;
use crate::services::vo::RespVO;

lazy_static::lazy_static! {
    pub static ref GRAPHQL_BUILD_CTX: BuilderContext = BuilderContext::default();
}

pub async fn graphql_json(schema: web::Data<Schema>, req: GraphQLRequest) -> DResult {
    let resp = schema.execute(req.into_inner()).await;
    Ok(HttpResponse::Ok().json(RespVO::from(&resp.data)))
}

pub async fn graphql_index(schema: web::Data<Schema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphql_playground() -> Result<HttpResponse> {
    let p_source = playground_source(GraphQLPlaygroundConfig::new("/hfrog"));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(p_source))
}
