use crate::{config::AppConfig, state::AppState};
use color_eyre::eyre::{Context, Result};
use redis::Client as RedisClient;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration}; // 引入 Duration

pub async fn setup_app_state(config: &AppConfig) -> Result<Arc<AppState>> {
    // 1. 設置資料庫連接池 (加入重試邏輯)
    let mut retry_count = 0;
    let max_retries = 5;

    let db = loop {
        match PgPoolOptions::new()
            .max_connections(config.db_max_connections)
            // 設定單次嘗試的超時，避免卡死
            .acquire_timeout(Duration::from_secs(3))
            .connect(&config.database_url)
            .await
        {
            Ok(pool) => break pool,
            Err(e) => {
                retry_count += 1;
                if retry_count > max_retries {
                    return Err(e).wrap_err("資料庫連線多次重試失敗，放棄啟動");
                }
                tracing::warn!(
                    "📡 資料庫連線失敗 ({}/{}), 2秒後重試: {}",
                    retry_count,
                    max_retries,
                    e
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    };

    // 2. 設置 HTTP 客戶端
    let http_client = Client::builder()
        .timeout(config.request_timeout)
        .build()
        .wrap_err("Failed to build HTTP client")?;

    // 3. 設置 Redis/Valkey 連接 (Redis 通常啟動很快，但保險起見也可加入簡易重試)
    let redis_client =
        RedisClient::open(config.valkey_url.as_str()).wrap_err("Failed to create Redis client")?;

    let redis = loop {
        match redis_client.get_connection_manager().await {
            Ok(manager) => {
                // 測試連線
                if test_redis_connection(&manager).await.is_ok() {
                    break manager;
                }
                tracing::warn!("📡 Redis PING 失敗，等待重試...");
            }
            Err(e) => {
                tracing::warn!("📡 Redis 管理器建立失敗: {}, 等待重試...", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    };

    tracing::info!("✅ 所有服務已就緒 (All services connected successfully)");

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
