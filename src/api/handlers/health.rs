use std::sync::Arc;

use crate::{
    api::response::{error, success},
    error::AppError,
    state::AppState,
};
use axum::{extract::State, http::StatusCode};
use color_eyre::eyre::eyre;

/// 健康檢查 - OK 路由處理函數
pub async fn health_ok(
    State(state): State<Arc<AppState>>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    // db 查詢
    let rows = sqlx::query("SELECT * FROM users")
        .fetch_all(&state.db)
        .await?;

    // http 請求
    let res = state.http_client.get("https://example.com").send().await?;

    tracing::info!("Users: {}, Http status: {}", rows.len(), res.status());

    Ok(success("ok"))
}

/// 健康檢查 - 故意失敗的路由處理函數
pub async fn health_fail() -> impl axum::response::IntoResponse {
    let err = eyre!("Intentional error");
    tracing::error!("{:?}", err); // 印完整 backtrace + source
    error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
