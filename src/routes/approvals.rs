use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::json;

use crate::{
    AppState,
    error::AppError,
    models::{AdvanceApprovalRequest, Approval},
};

pub async fn advance_approval(
    State(state): State<AppState>,
    Path(case_id): Path<String>,
    Json(request): Json<AdvanceApprovalRequest>,
) -> Result<Json<Vec<Approval>>, AppError> {
    let approvals = state
        .repository
        .advance_approval(&case_id, &request.decision, request.notes.as_deref())
        .await?;

    state
        .repository
        .record_audit(
            Some(&case_id),
            "approval_decided",
            "human",
            "demo_approver",
            &json!({
                "decision": request.decision,
                "notes": request.notes,
            }),
        )
        .await?;

    Ok(Json(approvals))
}
