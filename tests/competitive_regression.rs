//! Competitive regression baseline.
//!
//! These tests pin the competitive position the seeded landscape produces. They
//! fail when the scoring rules change, when a tracked dimension is dropped, or
//! when the landscape is edited — which is the point. A comparison that
//! silently changes shape is worse than no comparison, because decisions get
//! taken against it.
//!
//! When a change here is intended, update the expected values in the same
//! commit and say why in the message.

use marketshield::{
    build_state,
    models::ComparisonMatrix,
    repository::now_rfc3339,
    seed::{INDIVIDUAL_LIFE_RISK, RETIREMENT_ANNUITY, US},
    services::comparison,
};

/// Tolerance for a weighted score expressed on a 0-100 scale.
const TOLERANCE: f64 = 0.01;

async fn matrix(product_line_id: &str) -> ComparisonMatrix {
    let state = build_state("sqlite::memory:")
        .await
        .expect("state should build");
    let repository = &state.repository;

    let line = repository
        .get_product_line(product_line_id)
        .await
        .expect("product line should exist");
    let competitors = repository
        .list_competitors()
        .await
        .expect("competitors should load");
    let features = repository
        .list_feature_definitions(product_line_id)
        .await
        .expect("features should load");
    let observations = repository
        .list_feature_observations(product_line_id)
        .await
        .expect("observations should load");

    comparison::build_matrix(
        line,
        &competitors,
        &features,
        &observations,
        None,
        now_rfc3339().expect("timestamp"),
    )
    .expect("matrix should build")
}

#[tokio::test]
async fn retirement_annuity_standings_are_stable() {
    let matrix = matrix(RETIREMENT_ANNUITY).await;

    let expected = [
        ("ridgeline", 88.010_f64),
        ("table_bay_mutual", 34.654),
        (US, 30.331),
        ("silvertree", 6.897),
    ];

    assert_eq!(matrix.standings.len(), expected.len());
    for (index, (competitor_id, score)) in expected.iter().enumerate() {
        let standing = &matrix.standings[index];
        assert_eq!(
            standing.competitor_id,
            *competitor_id,
            "provider at rank {} changed",
            index + 1
        );
        assert!(
            (standing.position_score - score).abs() < TOLERANCE,
            "{} scored {:.3}, expected {:.3}",
            competitor_id,
            standing.position_score,
            score
        );
    }
}

/// The answer to the sponsor's question. If this changes, the diagnosis the
/// product gives has changed.
#[tokio::test]
async fn the_retirement_annuity_diagnosis_is_cost_then_transfer_speed() {
    let matrix = matrix(RETIREMENT_ANNUITY).await;

    assert_eq!(matrix.our_position.rank, 3);
    assert_eq!(matrix.our_position.provider_count, 4);
    assert_eq!(matrix.our_position.leader_competitor_id, "ridgeline");

    let gaps: Vec<&str> = matrix
        .our_position
        .largest_gaps
        .iter()
        .map(|gap| gap.feature_id.as_str())
        .collect();
    assert_eq!(gaps, ["ra_eac_500k", "ra_eac_2m", "ra_section14_days"]);
}

/// We must keep winning where we actually win; losing these silently would be
/// the same blind spot in the other direction.
#[tokio::test]
async fn we_still_lead_on_fund_range_and_adviser_fee_options() {
    let matrix = matrix(RETIREMENT_ANNUITY).await;

    for feature_id in ["ra_fund_range", "ra_adviser_fee_options"] {
        let feature = matrix
            .features
            .iter()
            .find(|feature| feature.feature_id == feature_id)
            .unwrap_or_else(|| panic!("{feature_id} should be tracked"));

        assert_eq!(
            feature.our_score,
            Some(100.0),
            "{feature_id} should still be a win"
        );
        assert_eq!(feature.gap_to_best, Some(0.0));
    }
}

#[tokio::test]
async fn every_tracked_dimension_is_compared_for_all_four_providers() {
    for product_line_id in [RETIREMENT_ANNUITY, INDIVIDUAL_LIFE_RISK] {
        let matrix = matrix(product_line_id).await;
        assert!(
            !matrix.features.is_empty(),
            "{product_line_id} should track dimensions"
        );

        for feature in &matrix.features {
            assert_eq!(
                feature.providers.len(),
                4,
                "{} on {} is missing a provider value, which would silently \
                 distort the comparison",
                feature.name,
                product_line_id
            );
            assert!(
                feature.our_value.is_some(),
                "{} has no value for our own offering",
                feature.name
            );
        }
    }
}

/// Ridgeline's repricing is the event the demonstration is built around.
#[tokio::test]
async fn the_competitor_repricing_is_recoverable_with_its_sources() {
    let matrix = matrix(RETIREMENT_ANNUITY).await;

    let moves: Vec<&str> = matrix
        .recent_moves
        .iter()
        .map(|item| item.feature_id.as_str())
        .collect();

    for expected in [
        "ra_eac_500k",
        "ra_eac_2m",
        "ra_platform_fee",
        "ra_section14_days",
    ] {
        assert!(
            moves.contains(&expected),
            "missing recorded move {expected}"
        );
    }

    for competitor_move in &matrix.recent_moves {
        assert_eq!(competitor_move.competitor_id, "ridgeline");
        assert!(
            competitor_move.favourable_to_them,
            "{} was recorded as moving against itself",
            competitor_move.feature_name
        );
        assert!(
            !competitor_move.source_reference.trim().is_empty(),
            "a recorded move without a source cannot be used as evidence"
        );
    }
}

#[tokio::test]
async fn individual_life_risk_standings_are_stable() {
    let matrix = matrix(INDIVIDUAL_LIFE_RISK).await;

    assert_eq!(matrix.our_position.rank, 3);
    assert!(
        (matrix.our_position.position_score - 28.634).abs() < TOLERANCE,
        "life risk position was {:.3}",
        matrix.our_position.position_score
    );
    assert_eq!(
        matrix.our_position.largest_gaps[0].feature_id, "life_underwriting_minutes",
        "underwriting turnaround should remain the leading life-risk deficit"
    );
}
