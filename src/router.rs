use crate::{
    api::handlers::{get_stock_day_all, handler_404, health_fail, health_ok, upload_image},
    config::load_config,
    state::AppState,
};
use axum::{
    Router,
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

/// 創建應用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    let config = load_config();
    Router::new()
        .route("/ok", get(health_ok))
        .route("/fail", get(health_fail))
        .route("/get_stock_day_all", get(get_stock_day_all))
        .route("/upload_image", post(upload_image))
        .fallback(handler_404)
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::with_status_code(StatusCode::SERVICE_UNAVAILABLE, config.request_timeout),
        ))
        .with_state(state)
}
