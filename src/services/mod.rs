#![allow(dead_code)]
use serde::Deserialize;
#[cfg(feature = "graphql")]
pub mod graphql;
pub mod vo;

#[macro_export]
///
/// 托尔斯泰
macro_rules! resp_suc {
    ($vo:ident, $status:expr) => {
        #[allow(non_snake_case, missing_docs)]
        pub fn $name() -> HttpResponse {
            HttpRe
        }
    };
}

#[allow(unused_macros)]
macro_rules! resp_err {
    ($name:ident, $status:expr) => {
        #[allow(non_snake_case, missing_docs)]
        pub fn $name() -> HttpResponseBuilder {
            HttpResponseBuilder::new($status)
        }
    };
}

#[derive(Deserialize)]
pub struct Paginate {
    page: u64,
    cnt: u64,
}

#[derive(Deserialize)]
pub struct PaginateQuery {
    // discount: Option<bool>,
    // sort: Option<bool>,
    paginate: Paginate,
}
