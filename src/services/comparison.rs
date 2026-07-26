//! Turns raw feature observations into an answer to "what do we offer versus
//! the competition, and where are we losing?".
//!
//! The matrix is built by a pure function so that the scoring rules can be
//! tested without a database, and so the same rules serve the API, the
//! monitor, and the regression suite.

use std::collections::HashMap;

use crate::{
    error::AppError,
    models::{
        ComparisonMatrix, Competitor, CompetitorMove, Direction, FeatureComparison,
        FeatureDefinition, FeatureGap, FeatureObservation, OurPosition, ProductLine, ProviderValue,
        Standing,
    },
};

/// How many of the worst weighted deficits are reported as the headline answer.
const LARGEST_GAP_COUNT: usize = 3;

/// Builds the comparison for a product line.
///
/// `as_at` selects a point in time: `None` compares today's values, while a
/// timestamp reconstructs the matrix as it stood then. Because observations are
/// superseded rather than overwritten, past standings are computed from the
/// same rules as the present one instead of being recorded separately.
pub fn build_matrix(
    product_line: ProductLine,
    competitors: &[Competitor],
    features: &[FeatureDefinition],
    observations: &[FeatureObservation],
    as_at: Option<&str>,
    generated_at: String,
) -> Result<ComparisonMatrix, AppError> {
    let us = competitors
        .iter()
        .find(|competitor| competitor.is_us == 1)
        .ok_or_else(|| {
            AppError::Internal("no competitor row is flagged as our own offering".to_string())
        })?;

    let names: HashMap<&str, &Competitor> = competitors
        .iter()
        .map(|competitor| (competitor.id.as_str(), competitor))
        .collect();

    // Values in force at the comparison point. Superseded rows are history and
    // feed `moves` below, except when reconstructing an earlier point in time.
    let mut current: HashMap<(&str, &str), &FeatureObservation> = HashMap::new();
    for observation in observations
        .iter()
        .filter(|observation| in_force_at(observation, as_at))
    {
        current.insert(
            (
                observation.feature_id.as_str(),
                observation.competitor_id.as_str(),
            ),
            observation,
        );
    }

    let mut comparisons = Vec::new();
    // competitor id -> (weighted score total, weight total, features covered)
    let mut totals: HashMap<&str, (f64, f64, usize)> = HashMap::new();

    for feature in features {
        let direction = Direction::parse(&feature.direction)?;
        let observed: Vec<(&Competitor, &FeatureObservation)> = competitors
            .iter()
            .filter_map(|competitor| {
                current
                    .get(&(feature.id.as_str(), competitor.id.as_str()))
                    .map(|observation| (competitor, *observation))
            })
            .collect();

        if observed.is_empty() {
            continue;
        }

        let values: Vec<f64> = observed
            .iter()
            .map(|(_, observation)| observation.value)
            .collect();
        let (best_value, worst_value) = match direction {
            Direction::LowerIsBetter => (min_of(&values), max_of(&values)),
            Direction::HigherIsBetter => (max_of(&values), min_of(&values)),
        };

        let best_provider = observed
            .iter()
            .find(|(_, observation)| (observation.value - best_value).abs() < f64::EPSILON)
            .map(|(competitor, _)| competitor.name.clone())
            .unwrap_or_default();

        let mut providers = Vec::new();
        for (competitor, observation) in &observed {
            let score = normalised_score(observation.value, best_value, worst_value);
            let entry = totals
                .entry(competitor.id.as_str())
                .or_insert((0.0, 0.0, 0));
            entry.0 += score * feature.weight;
            entry.1 += feature.weight;
            entry.2 += 1;

            providers.push(ProviderValue {
                competitor_id: competitor.id.clone(),
                competitor_name: competitor.name.clone(),
                is_us: competitor.is_us == 1,
                value: observation.value,
                score,
                is_best: (observation.value - best_value).abs() < f64::EPSILON,
                source_classification: observation.source_classification.clone(),
                source_reference: observation.source_reference.clone(),
                observed_at: observation.observed_at.clone(),
            });
        }

        let our_observation = current.get(&(feature.id.as_str(), us.id.as_str()));
        let our_value = our_observation.map(|observation| observation.value);
        let our_score = our_value.map(|value| normalised_score(value, best_value, worst_value));

        comparisons.push(FeatureComparison {
            feature_id: feature.id.clone(),
            name: feature.name.clone(),
            unit: feature.unit.clone(),
            direction: feature.direction.clone(),
            weight: feature.weight,
            description: feature.description.clone(),
            our_value,
            our_score,
            best_value,
            best_provider,
            // Always positive when we are behind, whichever way the feature runs.
            gap_to_best: our_value.map(|value| match direction {
                Direction::LowerIsBetter => value - best_value,
                Direction::HigherIsBetter => best_value - value,
            }),
            providers,
        });
    }

    let mut standings: Vec<Standing> = totals
        .iter()
        .map(|(competitor_id, (weighted, weight, covered))| {
            let competitor = names.get(competitor_id);
            Standing {
                competitor_id: (*competitor_id).to_string(),
                competitor_name: competitor
                    .map(|competitor| competitor.name.clone())
                    .unwrap_or_default(),
                is_us: competitor
                    .map(|competitor| competitor.is_us == 1)
                    .unwrap_or(false),
                position_score: if *weight > 0.0 {
                    weighted / weight
                } else {
                    0.0
                },
                rank: 0,
                features_covered: *covered,
            }
        })
        .collect();

    standings.sort_by(|a, b| {
        b.position_score
            .partial_cmp(&a.position_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.competitor_name.cmp(&b.competitor_name))
    });
    for (index, standing) in standings.iter_mut().enumerate() {
        standing.rank = index as i64 + 1;
    }

    let our_standing = standings
        .iter()
        .find(|standing| standing.is_us)
        .ok_or_else(|| {
            AppError::Internal("our own offering has no observed features".to_string())
        })?;
    let leader = standings
        .first()
        .ok_or_else(|| AppError::Internal("no provider standings could be built".to_string()))?;

    let mut largest_gaps: Vec<FeatureGap> = comparisons
        .iter()
        .filter_map(|comparison| {
            let our_value = comparison.our_value?;
            let our_score = comparison.our_score?;
            let gap = comparison.gap_to_best?;
            if gap.abs() < f64::EPSILON {
                return None;
            }
            Some(FeatureGap {
                feature_id: comparison.feature_id.clone(),
                feature_name: comparison.name.clone(),
                unit: comparison.unit.clone(),
                our_value,
                best_value: comparison.best_value,
                best_provider: comparison.best_provider.clone(),
                gap,
                weight: comparison.weight,
                weighted_deficit: comparison.weight * (100.0 - our_score),
            })
        })
        .collect();
    largest_gaps.sort_by(|a, b| {
        b.weighted_deficit
            .partial_cmp(&a.weighted_deficit)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    largest_gaps.truncate(LARGEST_GAP_COUNT);

    let our_position = OurPosition {
        position_score: our_standing.position_score,
        rank: our_standing.rank,
        provider_count: standings.len() as i64,
        leader_competitor_id: leader.competitor_id.clone(),
        leader_name: leader.competitor_name.clone(),
        score_behind_leader: leader.position_score - our_standing.position_score,
        largest_gaps,
    };

    Ok(ComparisonMatrix {
        product_line,
        generated_at,
        features: comparisons,
        standings,
        our_position,
        recent_moves: competitor_moves(competitors, features, observations, as_at)?,
    })
}

/// Timestamps are RFC 3339 in UTC throughout, so lexical ordering is
/// chronological ordering and no parsing is required here.
fn in_force_at(observation: &FeatureObservation, as_at: Option<&str>) -> bool {
    match as_at {
        None => observation.superseded_at.is_none(),
        Some(at) => {
            observation.observed_at.as_str() <= at
                && observation
                    .superseded_at
                    .as_deref()
                    .is_none_or(|superseded| superseded > at)
        }
    }
}

/// Reconstructs value changes from the observation history, newest first. This
/// is what lets the product say "Ridgeline cut their fee on 14 May" with a
/// source attached, rather than only showing today's numbers.
pub fn competitor_moves(
    competitors: &[Competitor],
    features: &[FeatureDefinition],
    observations: &[FeatureObservation],
    as_at: Option<&str>,
) -> Result<Vec<CompetitorMove>, AppError> {
    let feature_index: HashMap<&str, &FeatureDefinition> = features
        .iter()
        .map(|feature| (feature.id.as_str(), feature))
        .collect();
    let competitor_index: HashMap<&str, &Competitor> = competitors
        .iter()
        .map(|competitor| (competitor.id.as_str(), competitor))
        .collect();

    let mut series: HashMap<(&str, &str), Vec<&FeatureObservation>> = HashMap::new();
    for observation in observations {
        series
            .entry((
                observation.feature_id.as_str(),
                observation.competitor_id.as_str(),
            ))
            .or_default()
            .push(observation);
    }

    let mut moves = Vec::new();
    for ((feature_id, competitor_id), mut history) in series {
        let (Some(feature), Some(competitor)) = (
            feature_index.get(feature_id),
            competitor_index.get(competitor_id),
        ) else {
            continue;
        };
        let direction = Direction::parse(&feature.direction)?;
        history.sort_by(|a, b| a.observed_at.cmp(&b.observed_at));

        for pair in history.windows(2) {
            let (previous, current) = (pair[0], pair[1]);
            if (previous.value - current.value).abs() < f64::EPSILON {
                continue;
            }
            if as_at.is_some_and(|at| current.observed_at.as_str() > at) {
                continue;
            }
            moves.push(CompetitorMove {
                competitor_id: competitor.id.clone(),
                competitor_name: competitor.name.clone(),
                feature_id: feature.id.clone(),
                feature_name: feature.name.clone(),
                unit: feature.unit.clone(),
                previous_value: previous.value,
                current_value: current.value,
                changed_at: current.observed_at.clone(),
                favourable_to_them: match direction {
                    Direction::LowerIsBetter => current.value < previous.value,
                    Direction::HigherIsBetter => current.value > previous.value,
                },
                source_reference: current.source_reference.clone(),
            });
        }
    }

    moves.sort_by(|a, b| b.changed_at.cmp(&a.changed_at));
    Ok(moves)
}

/// 100 at the best observed value, 0 at the worst, linear between. Relative to
/// the observed field rather than an absolute scale, so a score only means
/// "where we sit among these providers on this dimension".
fn normalised_score(value: f64, best: f64, worst: f64) -> f64 {
    let span = best - worst;
    if span.abs() < f64::EPSILON {
        return 100.0;
    }
    (100.0 * (value - worst) / span).clamp(0.0, 100.0)
}

fn min_of(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::INFINITY, f64::min)
}

fn max_of(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn competitor(id: &str, name: &str, is_us: i64) -> Competitor {
        Competitor {
            id: id.to_string(),
            name: name.to_string(),
            short_name: name.to_string(),
            is_us,
            notes: String::new(),
        }
    }

    fn feature(id: &str, direction: &str, weight: f64) -> FeatureDefinition {
        FeatureDefinition {
            id: id.to_string(),
            product_line_id: "test_line".to_string(),
            name: id.to_string(),
            unit: "percent".to_string(),
            direction: direction.to_string(),
            weight,
            display_order: 1,
            description: String::new(),
        }
    }

    fn observation(
        competitor_id: &str,
        feature_id: &str,
        value: f64,
        observed_at: &str,
        superseded_at: Option<&str>,
    ) -> FeatureObservation {
        FeatureObservation {
            id: format!("{competitor_id}-{feature_id}-{observed_at}"),
            competitor_id: competitor_id.to_string(),
            feature_id: feature_id.to_string(),
            value,
            source_classification: "public".to_string(),
            source_reference: "test fixture".to_string(),
            observed_at: observed_at.to_string(),
            superseded_at: superseded_at.map(str::to_string),
        }
    }

    fn product_line() -> ProductLine {
        ProductLine {
            id: "test_line".to_string(),
            name: "Test line".to_string(),
            description: String::new(),
        }
    }

    fn matrix(
        competitors: &[Competitor],
        features: &[FeatureDefinition],
        observations: &[FeatureObservation],
    ) -> ComparisonMatrix {
        matrix_as_at(competitors, features, observations, None)
    }

    fn matrix_as_at(
        competitors: &[Competitor],
        features: &[FeatureDefinition],
        observations: &[FeatureObservation],
        as_at: Option<&str>,
    ) -> ComparisonMatrix {
        build_matrix(
            product_line(),
            competitors,
            features,
            observations,
            as_at,
            "2026-07-26T00:00:00Z".to_string(),
        )
        .expect("matrix should build")
    }

    #[test]
    fn lower_is_better_puts_the_cheapest_provider_first() {
        let competitors = [competitor("us", "Us", 1), competitor("rival", "Rival", 0)];
        let features = [feature("cost", "lower_is_better", 1.0)];
        let observations = [
            observation("us", "cost", 1.6, "2026-01-01T00:00:00Z", None),
            observation("rival", "cost", 0.9, "2026-01-01T00:00:00Z", None),
        ];

        let matrix = matrix(&competitors, &features, &observations);

        assert_eq!(matrix.standings[0].competitor_id, "rival");
        assert_eq!(matrix.our_position.rank, 2);
        assert_eq!(matrix.our_position.provider_count, 2);
        assert!((matrix.our_position.position_score - 0.0).abs() < 1e-9);
        assert!((matrix.features[0].gap_to_best.unwrap() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn higher_is_better_inverts_the_ordering() {
        let competitors = [competitor("us", "Us", 1), competitor("rival", "Rival", 0)];
        let features = [feature("fund_range", "higher_is_better", 1.0)];
        let observations = [
            observation("us", "fund_range", 120.0, "2026-01-01T00:00:00Z", None),
            observation("rival", "fund_range", 40.0, "2026-01-01T00:00:00Z", None),
        ];

        let matrix = matrix(&competitors, &features, &observations);

        assert_eq!(matrix.our_position.rank, 1);
        assert!((matrix.features[0].gap_to_best.unwrap() - 0.0).abs() < 1e-9);
        assert!((matrix.our_position.position_score - 100.0).abs() < 1e-9);
    }

    /// A heavily weighted dimension we lose must outrank a light one we lose by
    /// more, because weight is what makes a gap commercially material.
    #[test]
    fn largest_gaps_are_ranked_by_weighted_deficit() {
        let competitors = [competitor("us", "Us", 1), competitor("rival", "Rival", 0)];
        let features = [
            feature("heavy", "lower_is_better", 5.0),
            feature("light", "lower_is_better", 0.5),
        ];
        let observations = [
            observation("us", "heavy", 2.0, "2026-01-01T00:00:00Z", None),
            observation("rival", "heavy", 1.0, "2026-01-01T00:00:00Z", None),
            observation("us", "light", 90.0, "2026-01-01T00:00:00Z", None),
            observation("rival", "light", 10.0, "2026-01-01T00:00:00Z", None),
        ];

        let matrix = matrix(&competitors, &features, &observations);

        assert_eq!(matrix.our_position.largest_gaps[0].feature_id, "heavy");
    }

    #[test]
    fn position_score_is_weighted_across_features() {
        let competitors = [competitor("us", "Us", 1), competitor("rival", "Rival", 0)];
        let features = [
            feature("won", "lower_is_better", 3.0),
            feature("lost", "lower_is_better", 1.0),
        ];
        let observations = [
            observation("us", "won", 1.0, "2026-01-01T00:00:00Z", None),
            observation("rival", "won", 2.0, "2026-01-01T00:00:00Z", None),
            observation("us", "lost", 2.0, "2026-01-01T00:00:00Z", None),
            observation("rival", "lost", 1.0, "2026-01-01T00:00:00Z", None),
        ];

        let matrix = matrix(&competitors, &features, &observations);

        // Win a weight-3 feature (100) and lose a weight-1 feature (0) => 75.
        assert!((matrix.our_position.position_score - 75.0).abs() < 1e-9);
        assert_eq!(matrix.our_position.rank, 1);
    }

    #[test]
    fn superseded_values_are_reported_as_moves_and_excluded_from_scoring() {
        let competitors = [competitor("us", "Us", 1), competitor("rival", "Rival", 0)];
        let features = [feature("cost", "lower_is_better", 1.0)];
        let observations = [
            observation("us", "cost", 1.6, "2026-01-01T00:00:00Z", None),
            observation(
                "rival",
                "cost",
                1.5,
                "2026-01-01T00:00:00Z",
                Some("2026-05-01T00:00:00Z"),
            ),
            observation("rival", "cost", 0.9, "2026-05-01T00:00:00Z", None),
        ];

        let matrix = matrix(&competitors, &features, &observations);

        assert_eq!(matrix.recent_moves.len(), 1);
        let competitor_move = &matrix.recent_moves[0];
        assert!((competitor_move.previous_value - 1.5).abs() < 1e-9);
        assert!((competitor_move.current_value - 0.9).abs() < 1e-9);
        assert!(competitor_move.favourable_to_them);
        // Scoring uses 0.9, not the superseded 1.5.
        assert!((matrix.features[0].best_value - 0.9).abs() < 1e-9);
    }

    /// The same rules, run at an earlier date, must reproduce the position we
    /// held then. This is what makes the seeded trend real rather than asserted.
    #[test]
    fn as_at_reconstructs_the_earlier_position() {
        let competitors = [competitor("us", "Us", 1), competitor("rival", "Rival", 0)];
        let features = [feature("cost", "lower_is_better", 1.0)];
        let observations = [
            observation("us", "cost", 1.6, "2026-01-01T00:00:00Z", None),
            observation(
                "rival",
                "cost",
                1.5,
                "2026-01-01T00:00:00Z",
                Some("2026-05-01T00:00:00Z"),
            ),
            observation("rival", "cost", 0.9, "2026-05-01T00:00:00Z", None),
        ];

        let before = matrix_as_at(
            &competitors,
            &features,
            &observations,
            Some("2026-03-01T00:00:00Z"),
        );
        let after = matrix(&competitors, &features, &observations);

        assert!((before.features[0].gap_to_best.unwrap() - 0.1).abs() < 1e-9);
        assert!((after.features[0].gap_to_best.unwrap() - 0.7).abs() < 1e-9);
        // A move that had not happened yet must not appear in the earlier view.
        assert!(before.recent_moves.is_empty());
        assert_eq!(after.recent_moves.len(), 1);
    }

    #[test]
    fn a_single_provider_scores_full_marks() {
        let competitors = [competitor("us", "Us", 1)];
        let features = [feature("cost", "lower_is_better", 1.0)];
        let observations = [observation("us", "cost", 1.6, "2026-01-01T00:00:00Z", None)];

        let matrix = matrix(&competitors, &features, &observations);

        assert!((matrix.our_position.position_score - 100.0).abs() < 1e-9);
        assert!(matrix.our_position.largest_gaps.is_empty());
    }

    #[test]
    fn features_with_no_observations_are_skipped() {
        let competitors = [competitor("us", "Us", 1), competitor("rival", "Rival", 0)];
        let features = [
            feature("observed", "lower_is_better", 1.0),
            feature("unobserved", "lower_is_better", 1.0),
        ];
        let observations = [
            observation("us", "observed", 1.0, "2026-01-01T00:00:00Z", None),
            observation("rival", "observed", 2.0, "2026-01-01T00:00:00Z", None),
        ];

        let matrix = matrix(&competitors, &features, &observations);

        assert_eq!(matrix.features.len(), 1);
        assert_eq!(matrix.features[0].feature_id, "observed");
    }
}
