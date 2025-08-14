pub mod dao;
pub mod log;
use dao::DaoSetting;
use static_remote::S3RegionSetting;
use std::env;
use tracing;

#[derive(Debug, Clone)]
pub struct SvrBase {
    pub svr_name: String,
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone)]
pub struct Metrics {
    pub end_point: String,
}

///app运行全局配置上下文
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct RuntimeSetting {
    pub base: SvrBase,
    pub dao: DaoSetting,
    pub metrics: Option<Metrics>,
    pub s3: Option<S3RegionSetting>,
}

impl Default for RuntimeSetting {
    fn default() -> Self {
        println!("env: {:?}", env::var("PORT"));
        let mut conf = RuntimeSetting {
            dao: DaoSetting::new(),
            base: SvrBase {
                svr_name: env::var("SERVICE_NAME").unwrap_or("UNKNOWN SERVICE".to_owned()),
                port: env::var("PORT")
                    .unwrap_or("12121".to_owned())
                    .parse::<u16>()
                    .unwrap(),
                host: env::var("HOST").unwrap_or("0.0.0.0".to_owned()),
            },
            metrics: None,
            s3: None,
        };
        println!("port: {:?}", conf.base);
        #[cfg(feature = "metrics")]
        conf.try_load_metrics();
        conf.try_load_s3();
        println!("---\n{:#?}\n---", conf);
        return conf;
    }
}

impl RuntimeSetting {
    fn try_load_metrics(&mut self) {
        // <endpoint>->Required base param
        if let Ok(endpoint) = env::var("METRIC_ENDPOINT") {
            self.metrics = Some(Metrics {
                end_point: endpoint,
            });
            return;
        }
        tracing::warn!("METRIC_ENDPOINT is not set.RuntimeSetting init metrics config failed!");
        return;
    }

    fn try_load_s3(&mut self) {
        if let Ok(v) = dotenv::from_filename(".s3"){
            let s3 = S3RegionSetting::from_env();
            self.s3 = Some(s3);
            tracing::info!("load s3 config from .s3 file: {:?}", v);
        }
        else {
            tracing::warn!("failed load s3 config!");
        }
        
    }
}
