use axum::{Json, extract::State};
use serde_json::json;

use crate::{
    AppState,
    error::AppError,
    models::{DecisionReceipt, EvaluateScenarioRequest, ScenarioEvaluation, SubmitDecisionRequest},
    repository::now_rfc3339,
    services::{governance, scenario_engine},
};

pub async fn evaluate_scenario(
    State(state): State<AppState>,
    Json(request): Json<EvaluateScenarioRequest>,
) -> Result<Json<ScenarioEvaluation>, AppError> {
    let case = state.repository.get_case(&request.case_id).await?;
    let mut evaluation = scenario_engine::evaluate(&case, request)?;
    evaluation.stresses = governance::governance_stresses(&evaluation);

    state.repository.save_scenario(&evaluation).await?;
    state
        .repository
        .record_audit(
            Some(&evaluation.case_id),
            "scenario_evaluated",
            "system",
            "scenario_engine",
            &evaluation,
        )
        .await?;

    Ok(Json(evaluation))
}

pub async fn submit_decision(
    State(state): State<AppState>,
    Json(request): Json<SubmitDecisionRequest>,
) -> Result<Json<DecisionReceipt>, AppError> {
    let _case = state.repository.get_case(&request.case_id).await?;
    let scenario = state.repository.get_scenario(&request.scenario_id).await?;
    if scenario.case_id != request.case_id {
        return Err(AppError::BadRequest(
            "scenario does not belong to the requested case".to_string(),
        ));
    }
    if !["submit_for_review", "hold", "reject"].contains(&request.decision.as_str()) {
        return Err(AppError::BadRequest(
            "decision must be submit_for_review, hold or reject".to_string(),
        ));
    }

    let decision_id = state
        .repository
        .save_decision(
            &request.case_id,
            &request.scenario_id,
            &request.decision,
            request.notes.as_deref(),
        )
        .await?;
    let next_approval_role = state
        .repository
        .next_pending_approval(&request.case_id)
        .await?
        .map(|approval| approval.role_name);
    let created_at = now_rfc3339()?;

    state
        .repository
        .record_audit(
            Some(&request.case_id),
            "decision_submitted",
            "human",
            "demo_user",
            &json!({
                "decision_id": decision_id,
                "scenario_id": request.scenario_id,
                "decision": request.decision,
            }),
        )
        .await?;

    Ok(Json(DecisionReceipt {
        decision_id,
        case_id: request.case_id,
        scenario_id: request.scenario_id,
        decision: request.decision,
        next_approval_role,
        created_at,
    }))
}
