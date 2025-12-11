use std::sync::Arc;

use axum::{Router, routing::get};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::{
    api::handlers::{get_stock_day_all, health_fail, health_ok},
    config::load_config,
    state::AppState,
};

/// 創建應用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    let config = load_config();
    Router::new()
        .route("/ok", get(health_ok))
        .route("/fail", get(health_fail))
        .route("/get_stock_day_all", get(get_stock_day_all))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::with_status_code(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                config.request_timeout,
            ),
        ))
        .with_state(state)
}
