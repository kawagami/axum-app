use crate::api::response::{error, success};
use axum::http::StatusCode;
use color_eyre::eyre::eyre;

/// 健康檢查 - OK 路由處理函數
pub async fn health_ok() -> impl axum::response::IntoResponse {
    success("ok")
}

/// 健康檢查 - 故意失敗的路由處理函數
pub async fn health_fail() -> impl axum::response::IntoResponse {
    let err = eyre!("Intentional error");
    tracing::error!("{:?}", err); // 印完整 backtrace + source
    error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
