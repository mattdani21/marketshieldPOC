use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use marketshield::{app::build_router, build_state, config::Config, ensure_baseline_monitor_run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "marketshield=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let state = build_state(&config.database_url).await?;
    ensure_baseline_monitor_run(&state).await?;

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;

    info!(bind = %config.bind, "MarketShield API started");
    axum::serve(listener, app).await?;

    Ok(())
}
