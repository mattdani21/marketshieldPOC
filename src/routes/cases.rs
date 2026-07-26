use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    AppState,
    error::AppError,
    models::{AgentRunResult, CaseDetail, MarketCase},
    services::orchestrator,
};

pub async fn list_cases(State(state): State<AppState>) -> Result<Json<Vec<MarketCase>>, AppError> {
    Ok(Json(state.repository.list_cases().await?))
}

pub async fn get_case(
    State(state): State<AppState>,
    Path(case_id): Path<String>,
) -> Result<Json<CaseDetail>, AppError> {
    Ok(Json(state.repository.get_case_detail(&case_id).await?))
}

pub async fn analyse_case(
    State(state): State<AppState>,
    Path(case_id): Path<String>,
) -> Result<Json<AgentRunResult>, AppError> {
    let result = orchestrator::run_case_analysis(&state.repository, &case_id).await?;
    Ok(Json(result))
}
