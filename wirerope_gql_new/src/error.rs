use actix_web::{
    dev::ServiceResponse,
    http::{header, StatusCode},
    middleware::ErrorHandlerResponse,
    HttpResponse, ResponseError, Result,
};
use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;

pub type DResult = std::result::Result<HttpResponse, DError>;

/// 自定义错误
#[derive(Debug, Error)]
pub enum LogicErr {
    #[error("[AlreadyExist]{0} already exist!")]
    AlreadyExist(String), //101-
    #[error("[InsertFailed]{0} insert failed!")]
    InsertFailed(String),
    #[error("[UpdateFailed]{0} update failed!")]
    UpdateFailed(String),
    #[error("[NotFound]{0} not found!")]
    NotFound(String),
    #[error("[ConnectFailed]{0} db connect failed!")]
    ConnectFailed(String),
    #[error("[ParamsError]{0} params deserialize error!")]
    ParamsError(String), //201-
    #[error("[NeedUpdate]need update!")]
    NeedUpdate(String),
    #[error("[Rpc]rpc call failed!{0}")]
    RpcCallFailed(String)
}

impl LogicErr {
    pub fn code(&self) -> i32 {
        match self {
            LogicErr::AlreadyExist(_) => 1001,
            LogicErr::InsertFailed(_) => 1002,
            LogicErr::UpdateFailed(_) => 1003,
            LogicErr::NotFound(_) => 1004,
            LogicErr::ConnectFailed(_) => 1005,
            LogicErr::ParamsError(_) => 1006,
            LogicErr::NeedUpdate(_) => 1007,
            LogicErr::RpcCallFailed(_) => 1008
        }
    }
}

///Program error catcher
///
/// for:
///
///     * StdError
///
///     * Display
///
///     * DbErr
///
///     * ResponseError
#[derive(Debug, Error)]
pub enum DError {
    // actix_web inner
    #[error("[db error], {0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("[serialize to json error]:{0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("[redis error]:{0}")]
    RedisError(#[from] redis::RedisError),
    // 业务逻辑报错
    #[error("{0}")]
    Custom(LogicErr),
}

impl DError {
    pub fn err_code(&self) -> i32 {
        match self {
            DError::Db(_) => 101,
            DError::SerializeError(_v) => 102,
            DError::RedisError(_) => 103,
            DError::Custom(v) => v.code(),
        }
    }
}

/// http接口返回模型结构，提供基础的 code，msg，data 等json数据结构
#[derive(Serialize)]
pub struct RespErr {
    pub code: i32,
    pub msg: String,
}

impl ResponseError for DError {
    fn error_response(&self) -> HttpResponse {
        let code: i32 = self.err_code();
        let error_response = RespErr {
            code: code,
            msg: self.to_string(),
        };
        HttpResponse::Ok().json(error_response)
    }
}

pub async fn error_handler() -> DResult {
    Ok(HttpResponse::build(StatusCode::OK).json(serde_json::json!({
        "code": "9999",
        "msg": "unknown error"
    })))
}

///TODO : 系统处理下这里的错误信息
pub fn handle_bad_request<B>(mut res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("Error"),
    );

    // body is unchanged, map to "left" slot
    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}
