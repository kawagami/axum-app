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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "0.0.0.0".to_string(),
            request_timeout: Duration::from_secs(10),
        }
    }
}

/// 載入應用程式配置
pub fn load_config() -> AppConfig {
    // 這裡可以從環境變數或配置檔案載入配置
    // 現在先使用預設值
    AppConfig::default()
}
