use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    AppState,
    error::AppError,
    models::{
        ComparisonMatrix, Competitor, MonitorRunRequest, MonitorRunResult, PositionSnapshot,
        ProductLine, RecordObservationRequest,
    },
    repository::now_rfc3339,
    services::{comparison, monitor},
};

pub async fn list_product_lines(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProductLine>>, AppError> {
    Ok(Json(state.repository.list_product_lines().await?))
}

pub async fn list_competitors(
    State(state): State<AppState>,
) -> Result<Json<Vec<Competitor>>, AppError> {
    Ok(Json(state.repository.list_competitors().await?))
}

/// The comparison a product analyst actually asks for: what we offer against
/// every tracked competitor, on the dimensions the product is chosen on.
pub async fn comparison_matrix(
    State(state): State<AppState>,
    Path(product_line_id): Path<String>,
) -> Result<Json<ComparisonMatrix>, AppError> {
    let line = state.repository.get_product_line(&product_line_id).await?;
    let competitors = state.repository.list_competitors().await?;
    let features = state
        .repository
        .list_feature_definitions(&product_line_id)
        .await?;
    let observations = state
        .repository
        .list_feature_observations(&product_line_id)
        .await?;

    Ok(Json(comparison::build_matrix(
        line,
        &competitors,
        &features,
        &observations,
        None,
        now_rfc3339()?,
    )?))
}

pub async fn position_history(
    State(state): State<AppState>,
    Path(product_line_id): Path<String>,
) -> Result<Json<Vec<PositionSnapshot>>, AppError> {
    state.repository.get_product_line(&product_line_id).await?;
    Ok(Json(
        state
            .repository
            .list_position_snapshots(&product_line_id)
            .await?,
    ))
}

/// Records a new competitor value. The previous value is superseded rather than
/// overwritten, so the change becomes a competitor move the monitor can find.
pub async fn record_observation(
    State(state): State<AppState>,
    Json(request): Json<RecordObservationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !["public", "approved_internal", "consented", "licensed"]
        .contains(&request.source_classification.as_str())
    {
        return Err(AppError::BadRequest(
            "source_classification must be public, approved_internal, consented or licensed"
                .to_string(),
        ));
    }
    if request.source_reference.trim().is_empty() {
        return Err(AppError::BadRequest(
            "source_reference is required so every competitor value carries provenance".to_string(),
        ));
    }

    let observation_id = state
        .repository
        .record_observation(
            &request.competitor_id,
            &request.feature_id,
            request.value,
            &request.source_classification,
            &request.source_reference,
        )
        .await?;

    state
        .repository
        .record_audit(
            None,
            "observation_recorded",
            "human",
            "demo_user",
            &serde_json::json!({
                "observation_id": observation_id,
                "competitor_id": request.competitor_id,
                "feature_id": request.feature_id,
                "value": request.value,
                "source_reference": request.source_reference,
            }),
        )
        .await?;

    Ok(Json(
        serde_json::json!({ "observation_id": observation_id }),
    ))
}

pub async fn run_monitor(
    State(state): State<AppState>,
    request: Option<Json<MonitorRunRequest>>,
) -> Result<Json<MonitorRunResult>, AppError> {
    let request = request.map(|Json(request)| request).unwrap_or_default();
    Ok(Json(monitor::run(&state.repository, request).await?))
}
