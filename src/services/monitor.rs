//! The standing watch.
//!
//! Everything else in this service reacts to a competitive problem a human has
//! already noticed. This module is the part that notices: it recomputes our
//! position on every tracked product line, compares it to the last recorded
//! position, and raises a signal — and, where the drift is material, opens a
//! case — without anyone asking it to.

use serde_json::json;

use crate::{
    error::AppError,
    models::{
        ComparisonMatrix, MonitorFinding, MonitorRunRequest, MonitorRunResult, MonitorThreshold,
        ProductLine, ProductLineMonitorResult,
    },
    repository::{Repository, now_rfc3339},
    services::comparison,
};

/// Severity at which a finding is treated as requiring human attention: it
/// fails a scheduled check and is allowed to open a case.
const ACTIONABLE_SEVERITY: &str = "high";

pub async fn run(
    repository: &Repository,
    request: MonitorRunRequest,
) -> Result<MonitorRunResult, AppError> {
    let started_at = now_rfc3339()?;
    let trigger = request.trigger.unwrap_or_else(|| "manual".to_string());
    let competitors = repository.list_competitors().await?;

    let mut product_lines = Vec::new();
    let mut findings = Vec::new();
    let mut signals_created = Vec::new();
    let mut cases_opened = Vec::new();

    for line in repository.list_product_lines().await? {
        let features = repository.list_feature_definitions(&line.id).await?;
        let observations = repository.list_feature_observations(&line.id).await?;
        if features.is_empty() || observations.is_empty() {
            continue;
        }

        let thresholds = repository.list_monitor_thresholds(&line.id).await?;
        let previous = repository.latest_position_snapshot(&line.id).await?;

        let matrix = comparison::build_matrix(
            line.clone(),
            &competitors,
            &features,
            &observations,
            None,
            started_at.clone(),
        )?;

        let snapshot_id = repository
            .save_position_snapshot(&matrix, &started_at)
            .await?;

        let line_findings = detect_drift(&line, &matrix, previous.as_ref(), &thresholds);

        product_lines.push(ProductLineMonitorResult {
            product_line_id: line.id.clone(),
            product_line_name: line.name.clone(),
            position_score: matrix.our_position.position_score,
            previous_position_score: previous.as_ref().map(|snapshot| snapshot.position_score),
            score_change: previous
                .as_ref()
                .map(|snapshot| matrix.our_position.position_score - snapshot.position_score),
            rank: matrix.our_position.rank,
            previous_rank: previous.as_ref().map(|snapshot| snapshot.our_rank),
            provider_count: matrix.our_position.provider_count,
            leader_name: matrix.our_position.leader_name.clone(),
            snapshot_id,
        });

        let line_is_actionable = line_findings
            .iter()
            .any(|finding| finding.severity == ACTIONABLE_SEVERITY);

        if request.raise_signals {
            for finding in &line_findings {
                let signal_id = repository
                    .create_signal(
                        &finding.headline,
                        &finding.detail,
                        &finding.severity,
                        "approved_internal",
                        Some(&line.id),
                    )
                    .await?;
                signals_created.push(signal_id);
            }

            if line_is_actionable
                && !repository
                    .open_case_exists_for_product_line(&line.id)
                    .await?
            {
                let case_id = open_case(repository, &line, &matrix).await?;
                cases_opened.push(case_id);
            }
        }

        findings.extend(line_findings);
    }

    let breached = findings
        .iter()
        .any(|finding| finding.severity == ACTIONABLE_SEVERITY);

    let mut result = MonitorRunResult {
        run_id: String::new(),
        started_at: started_at.clone(),
        trigger: trigger.clone(),
        product_lines,
        findings,
        signals_created,
        cases_opened,
        breached,
    };

    result.run_id = repository
        .save_monitor_run(&started_at, &trigger, breached, &result)
        .await?;

    repository
        .record_audit(
            None,
            "monitor_run_completed",
            "agent",
            "competitive_monitor",
            &json!({
                "run_id": result.run_id,
                "trigger": trigger,
                "breached": breached,
                "findings": result.findings.len(),
                "cases_opened": result.cases_opened,
            }),
        )
        .await?;

    Ok(result)
}

/// Compares the freshly computed position to the last recorded one. On a first
/// run there is nothing to compare against, so the run only establishes a
/// baseline rather than reporting a phantom decline.
fn detect_drift(
    line: &ProductLine,
    matrix: &ComparisonMatrix,
    previous: Option<&crate::models::PositionSnapshot>,
    thresholds: &[MonitorThreshold],
) -> Vec<MonitorFinding> {
    let mut findings = Vec::new();
    let Some(previous) = previous else {
        return findings;
    };

    let threshold_for = |metric: &str| thresholds.iter().find(|item| item.metric == metric);

    let score_change = matrix.our_position.position_score - previous.position_score;
    if let Some(threshold) = threshold_for("position_score_drop")
        && score_change <= -threshold.threshold
    {
        findings.push(MonitorFinding {
            code: "position_score_drop".to_string(),
            severity: threshold.severity.clone(),
            product_line_id: line.id.clone(),
            product_line_name: line.name.clone(),
            headline: format!(
                "{} competitive position fell {:.1} points",
                line.name,
                score_change.abs()
            ),
            detail: format!(
                "Weighted position moved from {:.1} to {:.1} against {} providers. {} now leads. Largest deficits: {}.",
                previous.position_score,
                matrix.our_position.position_score,
                matrix.our_position.provider_count,
                matrix.our_position.leader_name,
                describe_gaps(matrix),
            ),
            observed_value: score_change.abs(),
            threshold: threshold.threshold,
        });
    }

    let rank_change = matrix.our_position.rank - previous.our_rank;
    if let Some(threshold) = threshold_for("rank_loss")
        && rank_change as f64 >= threshold.threshold
    {
        findings.push(MonitorFinding {
            code: "rank_loss".to_string(),
            severity: threshold.severity.clone(),
            product_line_id: line.id.clone(),
            product_line_name: line.name.clone(),
            headline: format!(
                "{} slipped from rank {} to rank {}",
                line.name, previous.our_rank, matrix.our_position.rank
            ),
            detail: format!(
                "We were overtaken on the weighted comparison of {} tracked dimensions. {} leads.",
                matrix.features.len(),
                matrix.our_position.leader_name
            ),
            observed_value: rank_change as f64,
            threshold: threshold.threshold,
        });
    }

    // Competitor moves that landed since the last look, with their sources.
    if let Some(threshold) = threshold_for("competitor_move") {
        for competitor_move in matrix
            .recent_moves
            .iter()
            .filter(|item| item.favourable_to_them && item.changed_at > previous.captured_at)
        {
            findings.push(MonitorFinding {
                code: "competitor_move".to_string(),
                severity: threshold.severity.clone(),
                product_line_id: line.id.clone(),
                product_line_name: line.name.clone(),
                headline: format!(
                    "{} improved {}",
                    competitor_move.competitor_name, competitor_move.feature_name
                ),
                detail: format!(
                    "{} moved from {} to {} {} on {}. Source: {}.",
                    competitor_move.competitor_name,
                    competitor_move.previous_value,
                    competitor_move.current_value,
                    competitor_move.unit,
                    // Date only: the time of day of a published disclosure is noise.
                    competitor_move
                        .changed_at
                        .get(..10)
                        .unwrap_or(&competitor_move.changed_at),
                    competitor_move.source_reference,
                ),
                observed_value: (competitor_move.current_value - competitor_move.previous_value)
                    .abs(),
                threshold: threshold.threshold,
            });
        }
    }

    findings
}

fn describe_gaps(matrix: &ComparisonMatrix) -> String {
    if matrix.our_position.largest_gaps.is_empty() {
        return "none".to_string();
    }

    matrix
        .our_position
        .largest_gaps
        .iter()
        .map(|gap| {
            format!(
                "{} ({} {} versus {} {} at {})",
                gap.feature_name,
                gap.our_value,
                gap.unit,
                gap.best_value,
                gap.unit,
                gap.best_provider
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// Opens a market-defence case from the diagnosed gaps. The hypotheses are the
/// weighted deficits in order, so the case arrives already carrying the
/// evidence for why it exists.
async fn open_case(
    repository: &Repository,
    line: &ProductLine,
    matrix: &ComparisonMatrix,
) -> Result<String, AppError> {
    let commercials = repository.get_product_line_commercials(&line.id).await?;
    let case_id = repository.next_case_id().await?;

    repository
        .insert_case(
            &case_id,
            &format!(
                "{} competitive position decline against {}",
                line.name, matrix.our_position.leader_name
            ),
            &commercials.product,
            &commercials.channel,
            &commercials.segment,
            &line.id,
            commercials.share_change_pp,
            commercials.annual_premium_at_risk_m,
            commercials.annual_quoted_premium_m,
            commercials.conversion_baseline_pct,
            commercials.conversion_current_pct,
            // Confidence follows the distance to the leader: a wide, consistent
            // gap is a more reliable diagnosis than a narrow one.
            (60.0 + matrix.our_position.score_behind_leader * 0.5).min(95.0),
        )
        .await?;

    for (index, gap) in matrix.our_position.largest_gaps.iter().enumerate() {
        repository
            .insert_hypothesis(
                &case_id,
                index as i64 + 1,
                &gap.feature_name,
                &format!(
                    "We are at {} {} against {} {} at {}. This dimension carries weight {:.1} in the comparison.",
                    gap.our_value, gap.unit, gap.best_value, gap.unit, gap.best_provider, gap.weight
                ),
                // Highest weighted deficit becomes the leading hypothesis.
                (95.0 - index as f64 * 13.0).max(40.0),
            )
            .await?;
    }

    repository
        .insert_evidence(
            &case_id,
            "Competitor feature comparison",
            &format!(
                "{} tracked dimensions across {} providers, drawn from published disclosures and approved internal product data.",
                matrix.features.len(),
                matrix.our_position.provider_count
            ),
            "public",
            "admitted",
        )
        .await?;
    repository
        .insert_evidence(
            &case_id,
            "Competitive position time series",
            "Recorded position snapshots showing when the decline began.",
            "approved_internal",
            "admitted",
        )
        .await?;
    repository
        .insert_evidence(
            &case_id,
            "Non-public competitor pricing",
            "Competitively sensitive information is not admitted.",
            "competitively_sensitive",
            "blocked",
        )
        .await?;

    let checks = [
        (
            "COMP-INFO",
            "Competition information control",
            "pass",
            "Every competitor value carries a published source reference; no non-public pricing was admitted.",
        ),
        (
            "POPIA-71",
            "Automated decision control",
            "pass",
            "The monitor opened a case for human diagnosis. It made no customer decision.",
        ),
        (
            "TCF",
            "Customer outcome assessment",
            "review",
            "Customer-outcome impact must be assessed against whichever response is selected.",
        ),
        (
            "MODEL-VAL",
            "Model validation",
            "review",
            "Position scoring is a deterministic demonstration model and is not validated for decision use.",
        ),
    ];
    for (code, name, status, rationale) in checks {
        repository
            .insert_governance_check(&case_id, code, name, status, rationale)
            .await?;
    }

    let approvals = [
        (1_i64, "Agent analysis", "approved"),
        (2, "Product actuary", "pending"),
        (3, "Compliance and legal", "waiting"),
        (4, "Risk and model validation", "waiting"),
        (5, "Product committee", "waiting"),
    ];
    for (sequence, role_name, status) in approvals {
        repository
            .insert_approval(&case_id, sequence, role_name, status)
            .await?;
    }

    repository
        .record_audit(
            Some(&case_id),
            "case_opened_by_monitor",
            "agent",
            "competitive_monitor",
            &json!({
                "case_id": case_id,
                "product_line_id": line.id,
                "position_score": matrix.our_position.position_score,
                "rank": matrix.our_position.rank,
                "leader": matrix.our_position.leader_name,
            }),
        )
        .await?;

    Ok(case_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Competitor, FeatureDefinition, FeatureObservation, PositionSnapshot};

    fn line() -> ProductLine {
        ProductLine {
            id: "test_line".to_string(),
            name: "Test line".to_string(),
            description: String::new(),
        }
    }

    fn thresholds() -> Vec<MonitorThreshold> {
        [
            ("position_score_drop", 3.0, "high"),
            ("rank_loss", 1.0, "high"),
            ("competitor_move", 1.0, "medium"),
        ]
        .into_iter()
        .map(|(metric, threshold, severity)| MonitorThreshold {
            id: metric.to_string(),
            product_line_id: None,
            metric: metric.to_string(),
            threshold,
            severity: severity.to_string(),
            description: String::new(),
        })
        .collect()
    }

    fn snapshot(score: f64, rank: i64, captured_at: &str) -> PositionSnapshot {
        PositionSnapshot {
            id: "previous".to_string(),
            product_line_id: "test_line".to_string(),
            captured_at: captured_at.to_string(),
            position_score: score,
            our_rank: rank,
            provider_count: 2,
            leader_competitor_id: "rival".to_string(),
            detail_json: "[]".to_string(),
        }
    }

    /// Builds a two-provider matrix where our cost is `our_cost` and the rival
    /// has moved from 1.5 to `rival_cost`.
    fn matrix_with(our_cost: f64, rival_cost: f64) -> ComparisonMatrix {
        let competitors = [
            Competitor {
                id: "us".to_string(),
                name: "Us".to_string(),
                short_name: "Us".to_string(),
                is_us: 1,
                notes: String::new(),
            },
            Competitor {
                id: "rival".to_string(),
                name: "Rival".to_string(),
                short_name: "Rival".to_string(),
                is_us: 0,
                notes: String::new(),
            },
        ];
        let features = [FeatureDefinition {
            id: "cost".to_string(),
            product_line_id: "test_line".to_string(),
            name: "Cost".to_string(),
            unit: "percent".to_string(),
            direction: "lower_is_better".to_string(),
            weight: 1.0,
            display_order: 1,
            description: String::new(),
        }];
        let observation =
            |competitor: &str, value: f64, at: &str, superseded: Option<&str>| FeatureObservation {
                id: format!("{competitor}-{at}"),
                competitor_id: competitor.to_string(),
                feature_id: "cost".to_string(),
                value,
                source_classification: "public".to_string(),
                source_reference: "fixture".to_string(),
                observed_at: at.to_string(),
                superseded_at: superseded.map(str::to_string),
            };
        let observations = [
            observation("us", our_cost, "2026-01-01T00:00:00Z", None),
            observation(
                "rival",
                1.5,
                "2026-01-01T00:00:00Z",
                Some("2026-05-01T00:00:00Z"),
            ),
            observation("rival", rival_cost, "2026-05-01T00:00:00Z", None),
        ];

        comparison::build_matrix(
            line(),
            &competitors,
            &features,
            &observations,
            None,
            "2026-07-26T00:00:00Z".to_string(),
        )
        .expect("matrix should build")
    }

    #[test]
    fn a_first_run_establishes_a_baseline_without_reporting_drift() {
        let findings = detect_drift(&line(), &matrix_with(1.6, 0.9), None, &thresholds());
        assert!(findings.is_empty());
    }

    #[test]
    fn a_score_drop_beyond_the_threshold_is_actionable() {
        let findings = detect_drift(
            &line(),
            &matrix_with(1.6, 0.9),
            Some(&snapshot(40.0, 2, "2026-02-01T00:00:00Z")),
            &thresholds(),
        );

        let drop = findings
            .iter()
            .find(|finding| finding.code == "position_score_drop")
            .expect("the score drop should be reported");
        assert_eq!(drop.severity, ACTIONABLE_SEVERITY);
        assert!(drop.detail.contains("Cost"));
    }

    #[test]
    fn a_score_drop_within_the_threshold_is_ignored() {
        // Our score is 0 here, so a previous score of 2.0 is a 2 point drop.
        let findings = detect_drift(
            &line(),
            &matrix_with(1.6, 0.9),
            Some(&snapshot(2.0, 2, "2026-02-01T00:00:00Z")),
            &thresholds(),
        );

        assert!(
            !findings
                .iter()
                .any(|finding| finding.code == "position_score_drop")
        );
    }

    #[test]
    fn an_improvement_is_never_reported_as_drift() {
        let findings = detect_drift(
            &line(),
            &matrix_with(0.8, 0.9),
            Some(&snapshot(0.0, 2, "2026-02-01T00:00:00Z")),
            &thresholds(),
        );

        assert!(
            !findings
                .iter()
                .any(|finding| finding.code == "position_score_drop")
        );
    }

    #[test]
    fn a_competitor_move_after_the_last_run_is_reported_with_its_source() {
        let findings = detect_drift(
            &line(),
            &matrix_with(1.6, 0.9),
            Some(&snapshot(40.0, 2, "2026-02-01T00:00:00Z")),
            &thresholds(),
        );

        let moved = findings
            .iter()
            .find(|finding| finding.code == "competitor_move")
            .expect("the competitor move should be reported");
        assert!(moved.detail.contains("fixture"));
        assert_eq!(moved.severity, "medium");
    }

    /// A move already known at the last run must not be raised a second time.
    #[test]
    fn a_competitor_move_before_the_last_run_is_not_repeated() {
        let findings = detect_drift(
            &line(),
            &matrix_with(1.6, 0.9),
            Some(&snapshot(40.0, 2, "2026-06-01T00:00:00Z")),
            &thresholds(),
        );

        assert!(
            !findings
                .iter()
                .any(|finding| finding.code == "competitor_move")
        );
    }
}
