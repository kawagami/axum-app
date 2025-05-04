use std::time::Duration;

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Router, routing::get};
use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_tracing() {
    color_eyre::install().unwrap(); // 自動啟用 backtrace & 彩色錯誤
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(ErrorLayer::default()) // <- 追蹤 error source
        .init();
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorInfo>,
}

#[derive(Serialize)]
pub struct ApiErrorInfo {
    pub code: u16,
    pub message: String,
}

pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
    let response = ApiResponse {
        success: true,
        data: Some(data),
        error: None,
    };
    (StatusCode::OK, Json(response))
}

pub fn error(status: StatusCode, message: impl Into<String>) -> impl IntoResponse {
    let response = ApiResponse::<()> {
        success: false,
        data: None,
        error: Some(ApiErrorInfo {
            code: status.as_u16(),
            message: message.into(),
        }),
    };
    (status, Json(response))
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();

    // Create a regular axum app.
    let app = Router::new()
        .route("/ok", get(|| async { success("ok") }))
        .route(
            "/fail",
            get(|| async {
                let err = eyre!("Intentional error");
                tracing::error!("{:?}", err); // <- 印完整 backtrace + source
                error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }),
        )
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
