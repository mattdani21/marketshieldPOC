//! End-to-end workflow tests against the real router and a real (in-memory)
//! database. These cover the path a sponsor demonstration actually walks:
//! monitor → case → analysis → scenario → decision → approval → audit.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use marketshield::{AppState, app::build_router, build_state, ensure_baseline_monitor_run};
use serde_json::{Value, json};
use tower::ServiceExt;

/// Each test gets its own private in-memory database, seeded and monitored once,
/// so tests are independent and can run in parallel.
async fn test_app() -> (Router, AppState) {
    let state = build_state("sqlite::memory:")
        .await
        .expect("state should build");
    ensure_baseline_monitor_run(&state)
        .await
        .expect("baseline monitor run should succeed");

    (build_router(state.clone()), state)
}

async fn get(app: &Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should complete");

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);

    (status, value)
}

async fn post(app: &Router, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should complete");

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);

    (status, value)
}

/// Finds the retirement annuity case the monitor opened for itself.
async fn retirement_annuity_case_id(app: &Router) -> String {
    let (status, cases) = get(app, "/api/cases").await;
    assert_eq!(status, StatusCode::OK);

    cases
        .as_array()
        .expect("cases should be a list")
        .iter()
        .find(|case| case["product_line_id"] == "retirement_annuity")
        .and_then(|case| case["id"].as_str())
        .expect("the monitor should have opened a retirement annuity case")
        .to_string()
}

#[tokio::test]
async fn health_reports_ok() {
    let (app, _state) = test_app().await;
    let (status, body) = get(&app, "/api/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

/// The core promise: the system opens the retirement annuity case itself,
/// before anyone asks it to.
#[tokio::test]
async fn the_monitor_opens_a_retirement_annuity_case_without_being_asked() {
    let (app, _state) = test_app().await;
    let case_id = retirement_annuity_case_id(&app).await;

    let (status, detail) = get(&app, &format!("/api/cases/{case_id}")).await;
    assert_eq!(status, StatusCode::OK);

    let hypotheses = detail["hypotheses"]
        .as_array()
        .expect("hypotheses should be a list");
    assert!(
        !hypotheses.is_empty(),
        "an opened case must carry its diagnosis"
    );
    assert_eq!(
        hypotheses[0]["name"], "Effective annual cost at R500k",
        "cost should be the leading diagnosed cause"
    );

    // The case must record why it was allowed to exist, not just that it does.
    let audit = get(&app, &format!("/api/audit?case_id={case_id}")).await.1;
    let events: Vec<&str> = audit
        .as_array()
        .expect("audit should be a list")
        .iter()
        .filter_map(|event| event["event_type"].as_str())
        .collect();
    assert!(events.contains(&"case_opened_by_monitor"));
}

#[tokio::test]
async fn the_comparison_answers_what_we_offer_against_competitors() {
    let (app, _state) = test_app().await;
    let (status, matrix) = get(&app, "/api/comparison/retirement_annuity").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(matrix["our_position"]["leader_name"], "Ridgeline Wealth");
    assert_eq!(matrix["our_position"]["rank"], 3);
    assert_eq!(matrix["our_position"]["provider_count"], 4);

    // Every competitor value must carry a source, or the competition-information
    // control is not evidenced.
    for feature in matrix["features"].as_array().expect("features") {
        for provider in feature["providers"].as_array().expect("providers") {
            let source = provider["source_reference"].as_str().unwrap_or_default();
            assert!(
                !source.trim().is_empty(),
                "provider value without a source reference in {}",
                feature["name"]
            );
        }
    }
}

#[tokio::test]
async fn an_unknown_product_line_is_not_found() {
    let (app, _state) = test_app().await;
    let (status, _) = get(&app, "/api/comparison/does_not_exist").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// The full governed path, end to end.
#[tokio::test]
async fn a_case_runs_from_analysis_through_to_approval() {
    let (app, _state) = test_app().await;
    let case_id = retirement_annuity_case_id(&app).await;

    let (status, analysis) = post(&app, &format!("/api/cases/{case_id}/analyse"), json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(analysis["status"], "completed");

    let (status, scenario) = post(
        &app,
        "/api/scenarios/evaluate",
        json!({ "case_id": case_id, "kind": "ra_fee_restructure", "overrides": {} }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let scenario_id = scenario["id"].as_str().expect("scenario id").to_string();

    let (status, receipt) = post(
        &app,
        "/api/decisions",
        json!({
            "case_id": case_id,
            "scenario_id": scenario_id,
            "decision": "submit_for_review",
            "notes": "integration test"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(receipt["next_approval_role"], "Product actuary");

    let (status, approvals) = post(
        &app,
        &format!("/api/cases/{case_id}/approvals/advance"),
        json!({ "decision": "approve", "notes": "integration test" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let approvals = approvals.as_array().expect("approvals should be a list");
    assert_eq!(approvals[1]["status"], "approved");
    assert_eq!(
        approvals[2]["status"], "pending",
        "approving one gate should open the next"
    );

    let (_, audit) = get(&app, &format!("/api/audit?case_id={case_id}")).await;
    let events: Vec<&str> = audit
        .as_array()
        .expect("audit should be a list")
        .iter()
        .filter_map(|event| event["event_type"].as_str())
        .collect();
    for expected in [
        "agent_analysis_completed",
        "scenario_evaluated",
        "decision_submitted",
    ] {
        assert!(events.contains(&expected), "missing audit event {expected}");
    }
}

#[tokio::test]
async fn a_scenario_cannot_be_evaluated_against_another_product_line() {
    let (app, _state) = test_app().await;
    let case_id = retirement_annuity_case_id(&app).await;

    let (status, _) = post(
        &app,
        "/api/scenarios/evaluate",
        json!({ "case_id": case_id, "kind": "rapid_underwriting", "overrides": {} }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_decision_cannot_reference_another_cases_scenario() {
    let (app, _state) = test_app().await;
    let case_id = retirement_annuity_case_id(&app).await;

    let (_, scenario) = post(
        &app,
        "/api/scenarios/evaluate",
        json!({ "case_id": case_id, "kind": "ra_fee_restructure", "overrides": {} }),
    )
    .await;

    let (status, _) = post(
        &app,
        "/api/decisions",
        json!({
            "case_id": "MS-2026-017",
            "scenario_id": scenario["id"],
            "decision": "submit_for_review"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Competitor values without provenance must be refused: the competition-
/// information control depends on every value having a citable source.
#[tokio::test]
async fn an_observation_without_usable_provenance_is_refused() {
    let (app, _state) = test_app().await;

    let (status, _) = post(
        &app,
        "/api/observations",
        json!({
            "competitor_id": "ridgeline",
            "feature_id": "ra_eac_500k",
            "value": 0.89,
            "source_classification": "competitively_sensitive",
            "source_reference": "overheard at a conference"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = post(
        &app,
        "/api/observations",
        json!({
            "competitor_id": "ridgeline",
            "feature_id": "ra_eac_500k",
            "value": 0.89,
            "source_classification": "public",
            "source_reference": "   "
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

async fn position_score(app: &Router) -> f64 {
    get(app, "/api/comparison/retirement_annuity").await.1["our_position"]["position_score"]
        .as_f64()
        .expect("position score")
}

async fn finding_codes(run: &Value) -> Vec<String> {
    run["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .filter_map(|finding| finding["code"].as_str())
        .map(str::to_string)
        .collect()
}

async fn record(app: &Router, competitor: &str, feature: &str, value: f64) {
    let (status, _) = post(
        app,
        "/api/observations",
        json!({
            "competitor_id": competitor,
            "feature_id": feature,
            "value": value,
            "source_classification": "public",
            "source_reference": "Public disclosure, July 2026"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

/// A competitor undercutting us lowers our position and is reported as a move,
/// but a small shift must not escalate into an actionable breach.
#[tokio::test]
async fn a_small_competitor_move_is_detected_without_breaching() {
    let (app, _state) = test_app().await;
    let before = position_score(&app).await;

    record(&app, "table_bay_mutual", "ra_eac_500k", 0.80).await;

    let after = position_score(&app).await;
    assert!(
        after < before,
        "a competitor undercutting us should lower our position ({before} -> {after})"
    );

    let (status, run) = post(
        &app,
        "/api/monitor/run",
        json!({ "trigger": "test", "raise_signals": false }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let codes = finding_codes(&run).await;
    assert!(
        codes.iter().any(|code| code == "competitor_move"),
        "the move itself should always be reported"
    );
    assert_eq!(
        run["breached"], false,
        "a sub-threshold shift must not be escalated"
    );
}

/// The loop that keeps the product from being blindsided twice: a material
/// competitive move is detected, breaches the threshold, and opens a case.
#[tokio::test]
async fn a_material_competitor_move_breaches_the_threshold_and_opens_a_case() {
    let (app, _state) = test_app().await;

    // Close the retirement annuity case the baseline run opened, so the monitor
    // is free to open a new one for this fresh deterioration.
    let case_id = retirement_annuity_case_id(&app).await;
    let (status, _) = post(
        &app,
        &format!("/api/cases/{case_id}/approvals/advance"),
        json!({ "decision": "reject", "notes": "closed for test" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let before = position_score(&app).await;

    // The trailing provider becomes competitive on both cost dimensions, which
    // moves us to the bottom of the range on the two heaviest features.
    record(&app, "silvertree", "ra_eac_500k", 1.00).await;
    record(&app, "silvertree", "ra_eac_2m", 1.00).await;

    let after = position_score(&app).await;
    assert!(
        before - after > 3.0,
        "the move should cost more than the 3 point threshold ({before} -> {after})"
    );

    let (status, run) = post(&app, "/api/monitor/run", json!({ "trigger": "test" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(run["breached"], true);

    let codes = finding_codes(&run).await;
    assert!(codes.iter().any(|code| code == "position_score_drop"));
    assert!(codes.iter().any(|code| code == "competitor_move"));

    assert_eq!(
        run["cases_opened"].as_array().expect("cases").len(),
        1,
        "a breach with no open case should open one"
    );
}

/// Running the monitor twice with nothing changing must not manufacture drift
/// or open duplicate cases.
#[tokio::test]
async fn a_repeated_monitor_run_finds_nothing_and_opens_nothing() {
    let (app, _state) = test_app().await;
    let case_count = |value: &Value| value.as_array().expect("cases").len();

    let before = case_count(&get(&app, "/api/cases").await.1);

    let (status, run) = post(&app, "/api/monitor/run", json!({ "trigger": "test" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(run["breached"], false);
    assert_eq!(run["findings"].as_array().expect("findings").len(), 0);
    assert_eq!(run["cases_opened"].as_array().expect("cases").len(), 0);

    let after = case_count(&get(&app, "/api/cases").await.1);
    assert_eq!(before, after, "a quiet run must not open cases");
}
