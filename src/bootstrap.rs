// src/bootstrap.rs

use crate::{config::AppConfig, state::AppState};
use color_eyre::eyre::{Context, Result};
use redis::Client as RedisClient;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

pub async fn setup_app_state(config: &AppConfig) -> Result<Arc<AppState>> {
    // 設置資料庫連接池
    let db = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await
        .wrap_err("Failed to connect to database")?;

    // 執行資料庫遷移
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .wrap_err("Failed to run database migrations")?;

    // 設置 HTTP 客戶端
    let http_client = Client::builder()
        .timeout(config.request_timeout)
        .build()
        .wrap_err("Failed to build HTTP client")?;

    // 設置 Redis/Valkey 連接
    let redis_client =
        RedisClient::open(config.valkey_url.as_str()).wrap_err("Failed to create Redis client")?;

    let redis = redis_client
        .get_connection_manager()
        .await
        .wrap_err("Failed to connect to Redis/Valkey")?;

    // 測試 Redis 連接
    test_redis_connection(&redis).await?;

    tracing::info!("✅ All services connected successfully");

    Ok(Arc::new(AppState {
        db,
        http_client,
        redis,
    }))
}

/// 測試 Redis 連接是否正常
async fn test_redis_connection(conn: &redis::aio::ConnectionManager) -> Result<()> {
    use redis::AsyncCommands;

    let mut conn = conn.clone();

    let pong: String = conn.ping().await.wrap_err("Failed to ping Redis/Valkey")?;

    tracing::info!("✅ Redis/Valkey connection successful: {}", pong);
    Ok(())
}
