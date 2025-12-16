use crate::{config::AppConfig, state::AppState};
use color_eyre::eyre::Result;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

pub async fn setup_app_state(config: &AppConfig) -> Result<Arc<AppState>> {
    let db = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let http_client = Client::builder().timeout(config.request_timeout).build()?;

    Ok(Arc::new(AppState { db, http_client }))
}
