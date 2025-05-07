use std::sync::Arc;

use axum::{Router, routing::get};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::{
    api::handlers::{health_fail, health_ok},
    config::load_config,
    state::AppState,
};

/// 創建應用路由
pub fn create_router(state: Arc<AppState>) -> Router {
    let config = load_config();
    Router::new()
        .route("/ok", get(health_ok))
        .route("/fail", get(health_fail))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(config.request_timeout),
        ))
        .with_state(state)
}
