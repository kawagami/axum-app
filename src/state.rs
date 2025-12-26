// src/state.rs

use redis::aio::ConnectionManager;
use reqwest::Client;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub http_client: Client,
    pub redis: ConnectionManager,
}
