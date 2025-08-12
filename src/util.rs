extern crate chrono;

use chrono::prelude::*;

///毫秒时间戳
pub fn cur_timestamp() -> i64 {
    let now: DateTime<Local> = Local::now();
    let millis = now.timestamp_micros();
    return millis;
}

///获取当前时间
pub fn cur_datetime() -> DateTime<Local> {
    Local::now()
}

///semverr字符串版本转换成int32
pub fn convert_version_to_int32(version: &str) -> Result<i32, &'static str> {
    // 将版本字符串分成三个部分
    let parts: Vec<i32> = version
        .split('.')
        .map(|s| s.parse::<i32>())
        .collect::<Result<Vec<i32>, _>>()
        .map_err(|_| "Invalid version string format")?;

    // 确保我们得到了三个部分
    if parts.len() != 3 {
        return Err("Received invalid version string");
    }

    // 确保每个部分都不大于 1023
    if parts.iter().any(|&part| part >= 1024) {
        return Err("Version string invalid, some parts are bigger than 1023");
    }

    // 将每个部分分别左移 0, 10 或 20 位并组合成一个 32 位整数
    let numeric_version = (parts[0] << 20) | (parts[1] << 10) | parts[2];

    Ok(numeric_version)
}

///serde_json换换成Vec<String>
pub fn value_to_vec_string(
    value: &serde_json::Value,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // 检查是否是数组类型
    if let serde_json::Value::Array(array) = value {
        // 将 Value 数组中的每个元素转换为 String
        let result: Result<Vec<String>, _> = array
            .iter()
            .map(|v| {
                // 尝试将每个元素转换为 String
                if let serde_json::Value::String(s) = v {
                    Ok(s.clone())
                } else {
                    Err("Element is not a string")
                }
            })
            .collect();

        result.map_err(|e| e.into())
    } else {
        Err("Value is not an array".into())
    }
}
