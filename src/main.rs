mod api;
mod config;
mod error;
mod logging;
mod router;
mod utils;

use color_eyre::eyre::Result;
use config::load_config;
use logging::setup_tracing;
use router::create_router;
use tokio::net::TcpListener;
use utils::shutdown::shutdown_signal;

#[tokio::main]
async fn main() -> Result<()> {
    // 設置日誌系統
    setup_tracing()?;

    // 創建應用路由
    let app = create_router();

    let config = load_config();

    let addr = format!("{}:{}", config.host, config.port);

    // 創建 TCP 監聽器
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    // 運行服務器並支持優雅關閉
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
