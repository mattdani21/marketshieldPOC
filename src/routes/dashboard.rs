use axum::{Json, extract::State};

use crate::{
    AppState,
    error::AppError,
    models::{DashboardSummary, Signal},
};

pub async fn dashboard(
    State(state): State<AppState>,
) -> Result<Json<DashboardSummary>, AppError> {
    let cases = state.repository.list_cases().await?;
    let primary_case = cases
        .first()
        .ok_or_else(|| AppError::NotFound("dashboard case".to_string()))?;
    let signals = state.repository.list_signals().await?;

    Ok(Json(DashboardSummary {
        estimated_segment_share_pct: 18.4,
        share_change_pp: primary_case.share_change_pp,
        annual_premium_at_risk_m: primary_case.annual_premium_at_risk_m,
        quote_conversion_pct: primary_case.conversion_current_pct,
        conversion_change_pp: primary_case.conversion_current_pct
            - primary_case.conversion_baseline_pct,
        open_cases: cases.iter().filter(|case| case.status == "open").count() as i64,
        signals,
    }))
}

pub async fn signals(State(state): State<AppState>) -> Result<Json<Vec<Signal>>, AppError> {
    Ok(Json(state.repository.list_signals().await?))
}
