use crate::{config::AppConfig, state::AppState};
use color_eyre::eyre::Result;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};

pub async fn setup_app_state(config: &AppConfig) -> Result<Arc<AppState>> {
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let http_client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    Ok(Arc::new(AppState { db, http_client }))
}
