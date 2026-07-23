use axum::{Json, extract::{Query, State}};
use serde::Deserialize;

use crate::{AppState, error::AppError, models::AuditEvent};

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    case_id: Option<String>,
}

pub async fn list_audit(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<AuditEvent>>, AppError> {
    Ok(Json(
        state
            .repository
            .list_audit_events(query.case_id.as_deref())
            .await?,
    ))
}
