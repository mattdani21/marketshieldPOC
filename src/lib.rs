pub mod app;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod repository;
pub mod routes;
pub mod seed;
pub mod services;

use std::sync::Arc;

use crate::{
    error::AppError, models::MonitorRunRequest, repository::Repository, services::monitor,
};

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<Repository>,
}

/// Connects, migrates and seeds. Shared by the service, the monitor binary and
/// the test suite so all three exercise the same construction path.
pub async fn build_state(database_url: &str) -> Result<AppState, AppError> {
    let pool = db::connect(database_url).await?;
    let repository = Arc::new(Repository::new(pool));

    repository.seed_demo_data().await?;
    seed::seed_competitive_data(&repository).await?;

    Ok(AppState { repository })
}

/// Runs the monitor once on a database that has never been monitored, so a
/// fresh environment starts with its competitive position already assessed
/// rather than waiting for the first scheduled run.
pub async fn ensure_baseline_monitor_run(state: &AppState) -> Result<(), AppError> {
    if state.repository.monitor_run_count().await? > 0 {
        return Ok(());
    }

    monitor::run(
        &state.repository,
        MonitorRunRequest {
            trigger: Some("startup_baseline".to_string()),
            raise_signals: true,
        },
    )
    .await?;

    Ok(())
}
