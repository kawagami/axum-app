mod api;
mod bootstrap;
mod config;
mod error;
mod logging;
mod router;
mod server;
mod state;
mod utils;

use bootstrap::setup_app_state;
use color_eyre::eyre::Result;
use config::load_config;
use logging::setup_tracing;
use router::create_router;
use server::run_server;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;
    let config = load_config();

    let app_state = setup_app_state(&config).await?;
    let app = create_router(app_state);

    let addr = format!("{}:{}", config.host, config.port);
    run_server(&addr, app).await?;

    Ok(())
}
