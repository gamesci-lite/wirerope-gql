use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::LogicErr;

pub const DESC_SUC: &str = "suc";
pub const CODE_SUC: i32 = 0;

/// http接口返回模型结构，提供基础的 code，msg，data 等json数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespRet {
    pub code: i32,
    pub msg: String,
}

impl RespRet {
    pub fn with_msg(msg: &str) -> Self {
        Self {
            code: CODE_SUC,
            msg: msg.to_string(),
        }
    }

    pub fn with_error(err: &LogicErr) -> Self {
        Self {
            code: err.code(),
            msg: err.to_string(),
        }
    }

    pub fn new(code: i32, msg: &str) -> Self {
        Self {
            code,
            msg: msg.to_owned(),
        }
    }
}

impl Default for RespRet {
    fn default() -> Self {
        Self {
            code: CODE_SUC,
            msg: DESC_SUC.to_string(),
        }
    }
}

/// http接口返回模型结构，提供基础的 code，msg，data 等json数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespVO<T> {
    pub code: i32,
    pub msg: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> RespVO<T> {
    pub fn from(arg: &T) -> Self
    where
        T: Serialize + DeserializeOwned + Clone,
    {
        Self {
            code: CODE_SUC,
            msg: "".to_string(),
            data: Some(arg.clone()),
        }
    }
}
