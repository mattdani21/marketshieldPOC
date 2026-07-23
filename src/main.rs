mod app;
mod config;
mod db;
mod error;
mod models;
mod repository;
mod routes;
mod services;

use std::sync::Arc;

use config::Config;
use db::connect;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{app::build_router, repository::Repository};

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<Repository>,
}

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
    let pool = connect(&config.database_url).await?;
    let repository = Arc::new(Repository::new(pool));
    repository.seed_demo_data().await?;

    let state = AppState { repository };
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;

    info!(bind = %config.bind, "MarketShield API started");
    axum::serve(listener, app).await?;

    Ok(())
}
