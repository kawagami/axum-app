use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
    pub request_timeout: Duration,
    pub database_url: String,
    pub db_max_connections: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: std::env::var("APP_PORT")
                .expect("Not Found APP_PORT")
                .parse::<u16>()
                .expect("APP_PORT value must be a valid u16 number"),
            host: std::env::var("APP_HOST").expect("Not Found APP_HOST"),
            request_timeout: Duration::from_secs(
                std::env::var("REQUEST_TIMEOUT")
                    .expect("Not Found REQUEST_TIMEOUT")
                    .parse::<u64>()
                    .expect("REQUEST_TIMEOUT value must be a valid u64 number"),
            ),
            database_url: std::env::var("DATABASE_URL").expect("Not Found DATABASE_URL"),
            db_max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .expect("Not Found DB_MAX_CONNECTIONS")
                .parse::<u32>()
                .expect("DB_MAX_CONNECTIONS value must be a valid u32 number"),
        }
    }
}

pub fn load_config() -> AppConfig {
    dotenvy::dotenv().ok();

    AppConfig::default()
}
