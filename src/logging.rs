use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 設置追蹤系統和彩色錯誤報告
pub fn setup_tracing() -> Result<()> {
    // 安裝 color_eyre 以獲得彩色錯誤和自動回溯
    color_eyre::install()?;

    // 設置追蹤訂閱者
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
        .with(ErrorLayer::default()) // 追蹤錯誤源
        .init();

    Ok(())
}
