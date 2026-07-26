mod approvals;
mod audit;
mod cases;
mod competitive;
mod dashboard;
mod health;
mod scenarios;

use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/dashboard", get(dashboard::dashboard))
        .route("/signals", get(dashboard::signals))
        .route("/cases", get(cases::list_cases))
        .route("/cases/{case_id}", get(cases::get_case))
        .route("/cases/{case_id}/analyse", post(cases::analyse_case))
        .route("/scenarios/evaluate", post(scenarios::evaluate_scenario))
        .route("/decisions", post(scenarios::submit_decision))
        .route(
            "/cases/{case_id}/approvals/advance",
            post(approvals::advance_approval),
        )
        .route("/audit", get(audit::list_audit))
        .route("/product-lines", get(competitive::list_product_lines))
        .route("/competitors", get(competitive::list_competitors))
        .route(
            "/comparison/{product_line_id}",
            get(competitive::comparison_matrix),
        )
        .route(
            "/comparison/{product_line_id}/history",
            get(competitive::position_history),
        )
        .route("/observations", post(competitive::record_observation))
        .route("/monitor/run", post(competitive::run_monitor))
}
