use std::{env, path::PathBuf, str::FromStr};

use serde::Deserialize;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub enable_log_file: bool,
    pub enable_stdout: bool,
    pub level: String,
    // pub enable_loki: bool, (废弃了，使用vector来管理日志跟优雅，多前端后端)
}
impl LogConfig {
    ///
    /// 从配置文件初始化配置
    pub fn try_from_file(f: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(f_) = f {
            if f_.is_file() {
                let js_cont: String = std::fs::read_to_string(&f_)?;
                let cfg: LogConfig = serde_json::from_str(js_cont.as_str())
                    .expect(format!("parse log config {} failed!", f_.display()).as_str());
                return Ok(cfg);
            }
        }
        Err("No valid configuration file provided".into())
    }

    ///
    /// 从环境初始化log配置
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            enable_log_file: env::var("ENABLE_LOG_FILE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            enable_stdout: env::var("ENABLE_STDOUT")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }
}

impl LogConfig {
    /// 初始化日志
    /// 这里使用tracing替代log
    /// 后续方便扩展span监控
    pub fn init_logger(cfg: &LogConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut layers: Vec<Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync>> = vec![];
        println!("---\nLog {:#?}\n---", cfg);
        // log-file
        if cfg.enable_log_file {
            // 获取当前目录
            let root_dir = env::current_dir()?;
            // 构建日志目录路径
            let log_dir = root_dir.join("..").join("logs");
            // 创建日志目录
            std::fs::create_dir_all(&log_dir)?;
            let file_appender =
                tracing_appender::rolling::daily(&log_dir, log_dir.join("access.log"));
            {
                let (non_blocking, _guard) =
                    tracing_appender::non_blocking::NonBlockingBuilder::default()
                        .lossy(false)
                        .finish(file_appender);
                let layer: Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync> =
                    tracing_subscriber::fmt::layer()
                        .with_ansi(false)
                        .with_writer(non_blocking)
                        .with_filter(
                            LevelFilter::from_str(&cfg.level)
                                .unwrap_or_else(|e| "info".parse().expect("ENSURE")),
                        )
                        .boxed();
                layers.push(layer);
            }
        }
        // stdout
        if cfg.enable_stdout {
            let layer = tracing_subscriber::fmt::layer()
                .with_filter(
                    LevelFilter::from_str(&cfg.level)
                        .unwrap_or_else(|e| "info".parse().expect("ENSURE")),
                )
                .boxed();

            layers.push(layer);
        }
        tracing_subscriber::registry().with(layers).init();
        Ok(())
    }
}
