use crate::utils::shutdown::shutdown_signal;
use axum::Router;
use color_eyre::eyre::Result;
use tokio::net::TcpListener;

pub async fn run_server(addr: &str, app: Router) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
