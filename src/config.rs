use std::time::Duration;

/// 應用程式配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 服務器埠號
    pub port: u16,
    /// 服務器主機地址
    pub host: String,
    /// 請求超時時間
    pub request_timeout: Duration,
    /// 請求超時時間
    pub database_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "0.0.0.0".to_string(),
            request_timeout: Duration::from_secs(10),
            // database_url: "postgresql://kawa:kawa@host.docker.internal:5432/kawa".to_string(),
            database_url: std::env::var("DATABASE_URL").expect("Not Found DATABASE_URL"),
        }
    }
}

/// 載入應用程式配置
pub fn load_config() -> AppConfig {
    // 載入 .env 檔案
    dotenvy::dotenv().ok();

    AppConfig::default()
}
