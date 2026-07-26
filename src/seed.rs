//! Synthetic competitive landscape for the demonstration.
//!
//! Every value here is invented. The point of the seed is not the numbers but
//! the shape: a set of providers measured on the dimensions a retirement
//! annuity is actually chosen on, with a history that contains a real
//! competitor move for the monitor to find.

use sqlx::query;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::ProductLine,
    repository::{Repository, days_ago_rfc3339},
    services::comparison,
};

/// Days before today that each seeded historical snapshot is dated. All three
/// precede the competitor repricing at 75 days, so the recorded history ends
/// before the decline and the first live run is the one that finds it.
const SNAPSHOT_DAYS_AGO: [i64; 3] = [150, 120, 90];

pub const RETIREMENT_ANNUITY: &str = "retirement_annuity";
pub const INDIVIDUAL_LIFE_RISK: &str = "individual_life_risk";

pub const US: &str = "cape_meridian";

struct Observation {
    competitor: &'static str,
    feature: &'static str,
    value: f64,
    /// Days before today the value was first observed.
    observed_days_ago: i64,
    /// Days before today the value stopped applying, if it has been replaced.
    superseded_days_ago: Option<i64>,
    source_reference: &'static str,
}

pub async fn seed_competitive_data(repository: &Repository) -> Result<(), AppError> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM product_lines")
        .fetch_one(repository.pool())
        .await?;
    if existing > 0 {
        return Ok(());
    }

    seed_product_lines(repository).await?;
    seed_competitors(repository).await?;
    seed_feature_definitions(repository).await?;
    seed_observations(repository).await?;
    seed_thresholds(repository).await?;
    seed_commercials(repository).await?;
    backfill_existing_life_case(repository).await?;
    seed_position_history(repository).await?;

    repository
        .record_audit(
            None,
            "competitive_landscape_seeded",
            "system",
            "seed_competitive_data",
            &serde_json::json!({
                "product_lines": [RETIREMENT_ANNUITY, INDIVIDUAL_LIFE_RISK],
                "note": "All competitor values are synthetic.",
            }),
        )
        .await?;

    Ok(())
}

async fn seed_product_lines(repository: &Repository) -> Result<(), AppError> {
    let lines = [
        (
            RETIREMENT_ANNUITY,
            "Retirement annuity",
            "Individual retirement annuity sold through advisers and direct digital channels.",
        ),
        (
            INDIVIDUAL_LIFE_RISK,
            "Individual life risk",
            "Underwritten individual life cover sold through advisers.",
        ),
    ];

    for (id, name, description) in lines {
        query("INSERT INTO product_lines (id, name, description) VALUES (?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(description)
            .execute(repository.pool())
            .await?;
    }

    Ok(())
}

async fn seed_competitors(repository: &Repository) -> Result<(), AppError> {
    let competitors = [
        (
            US,
            "Cape Meridian Life & Investments",
            "Cape Meridian",
            1,
            "Our own offering. Held in the same table as competitors so every comparison uses one set of rules.",
        ),
        (
            "ridgeline",
            "Ridgeline Wealth",
            "Ridgeline",
            0,
            "Direct-led challenger. The provider named in the sponsor's share-loss question.",
        ),
        (
            "table_bay_mutual",
            "Table Bay Mutual",
            "Table Bay",
            0,
            "Established mutual with a comparable adviser footprint.",
        ),
        (
            "silvertree",
            "Silvertree Financial",
            "Silvertree",
            0,
            "Adviser-only incumbent competing on service rather than cost.",
        ),
    ];

    for (id, name, short_name, is_us, notes) in competitors {
        query(
            "INSERT INTO competitors (id, name, short_name, is_us, notes) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(short_name)
        .bind(is_us)
        .bind(notes)
        .execute(repository.pool())
        .await?;
    }

    Ok(())
}

/// (id, product line, name, unit, direction, weight, display order, description)
type FeatureSeed = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    f64,
    i64,
    &'static str,
);

async fn seed_feature_definitions(repository: &Repository) -> Result<(), AppError> {
    let features: [FeatureSeed; 12] = [
        (
            "ra_eac_500k",
            RETIREMENT_ANNUITY,
            "Effective annual cost at R500k",
            "percent",
            "lower_is_better",
            3.0,
            1,
            "Total effective annual cost on a R500 000 balance, the headline figure in adviser comparisons.",
        ),
        (
            "ra_eac_2m",
            RETIREMENT_ANNUITY,
            "Effective annual cost at R2m",
            "percent",
            "lower_is_better",
            2.5,
            2,
            "Effective annual cost on a R2 000 000 balance, where fee tiering matters most.",
        ),
        (
            "ra_platform_fee",
            RETIREMENT_ANNUITY,
            "Platform administration fee",
            "percent",
            "lower_is_better",
            2.0,
            3,
            "Annual platform administration charge before asset management fees.",
        ),
        (
            "ra_section14_days",
            RETIREMENT_ANNUITY,
            "Section 14 transfer turnaround",
            "days",
            "lower_is_better",
            2.0,
            4,
            "Working days from transfer instruction to assets invested.",
        ),
        (
            "ra_onboarding_minutes",
            RETIREMENT_ANNUITY,
            "Digital onboarding time",
            "minutes",
            "lower_is_better",
            1.5,
            5,
            "Median time to complete a new retirement annuity application online.",
        ),
        (
            "ra_fund_range",
            RETIREMENT_ANNUITY,
            "Regulation 28 fund choices",
            "count",
            "higher_is_better",
            1.5,
            6,
            "Number of Regulation 28 compliant portfolios available on the product.",
        ),
        (
            "ra_min_monthly",
            RETIREMENT_ANNUITY,
            "Minimum monthly contribution",
            "zar",
            "lower_is_better",
            1.0,
            7,
            "Lowest recurring contribution accepted, which sets the accessible market.",
        ),
        (
            "ra_adviser_fee_options",
            RETIREMENT_ANNUITY,
            "Adviser fee structure options",
            "count",
            "higher_is_better",
            1.0,
            8,
            "Number of supported adviser remuneration structures.",
        ),
        (
            "life_underwriting_minutes",
            INDIVIDUAL_LIFE_RISK,
            "Underwriting decision turnaround",
            "minutes",
            "lower_is_better",
            3.0,
            1,
            "Median time from submitted application to an underwriting decision.",
        ),
        (
            "life_premium_index",
            INDIVIDUAL_LIFE_RISK,
            "Premium index, professional lives",
            "index",
            "lower_is_better",
            2.0,
            2,
            "Premium for a standard professional life, indexed to the market average at 100.",
        ),
        (
            "life_tele_underwriting",
            INDIVIDUAL_LIFE_RISK,
            "Tele-underwriting coverage",
            "percent",
            "higher_is_better",
            1.5,
            3,
            "Share of applications completed without a face-to-face medical appointment.",
        ),
        (
            "life_cover_without_medicals",
            INDIVIDUAL_LIFE_RISK,
            "Maximum cover without medicals",
            "zar_millions",
            "higher_is_better",
            1.5,
            4,
            "Largest sum assured issued on non-medical evidence.",
        ),
    ];

    for (id, line, name, unit, direction, weight, order, description) in features {
        query(
            "INSERT INTO feature_definitions (id, product_line_id, name, unit, direction, weight, display_order, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(line)
        .bind(name)
        .bind(unit)
        .bind(direction)
        .bind(weight)
        .bind(order)
        .bind(description)
        .execute(repository.pool())
        .await?;
    }

    Ok(())
}

fn current(
    competitor: &'static str,
    feature: &'static str,
    value: f64,
    source_reference: &'static str,
) -> Observation {
    Observation {
        competitor,
        feature,
        value,
        observed_days_ago: 200,
        superseded_days_ago: None,
        source_reference,
    }
}

fn observations() -> Vec<Observation> {
    let mut values = vec![
        // --- Retirement annuity: our own offering -----------------------
        current(
            US,
            "ra_eac_500k",
            1.62,
            "Internal product fee schedule 2026.1",
        ),
        current(
            US,
            "ra_eac_2m",
            1.24,
            "Internal product fee schedule 2026.1",
        ),
        current(
            US,
            "ra_platform_fee",
            0.55,
            "Internal product fee schedule 2026.1",
        ),
        current(
            US,
            "ra_section14_days",
            47.0,
            "Internal transfer operations report, Q2 2026",
        ),
        current(
            US,
            "ra_onboarding_minutes",
            38.0,
            "Internal digital funnel analytics, Q2 2026",
        ),
        current(US, "ra_fund_range", 96.0, "Internal fund list 2026.1"),
        current(US, "ra_min_monthly", 750.0, "Internal product rules 2026.1"),
        current(
            US,
            "ra_adviser_fee_options",
            4.0,
            "Internal adviser remuneration guide 2026",
        ),
        // --- Retirement annuity: Table Bay ------------------------------
        current(
            "table_bay_mutual",
            "ra_eac_500k",
            1.48,
            "Public EAC disclosure, March 2026",
        ),
        current(
            "table_bay_mutual",
            "ra_eac_2m",
            1.15,
            "Public EAC disclosure, March 2026",
        ),
        current(
            "table_bay_mutual",
            "ra_platform_fee",
            0.50,
            "Public fee schedule, March 2026",
        ),
        current(
            "table_bay_mutual",
            "ra_section14_days",
            38.0,
            "Published service standard, 2026",
        ),
        current(
            "table_bay_mutual",
            "ra_onboarding_minutes",
            25.0,
            "Observed public application flow",
        ),
        current(
            "table_bay_mutual",
            "ra_fund_range",
            65.0,
            "Public fund list, March 2026",
        ),
        current(
            "table_bay_mutual",
            "ra_min_monthly",
            1000.0,
            "Public product brochure, 2026",
        ),
        current(
            "table_bay_mutual",
            "ra_adviser_fee_options",
            3.0,
            "Public adviser guide, 2026",
        ),
        // --- Retirement annuity: Silvertree -----------------------------
        current(
            "silvertree",
            "ra_eac_500k",
            1.71,
            "Public EAC disclosure, February 2026",
        ),
        current(
            "silvertree",
            "ra_eac_2m",
            1.33,
            "Public EAC disclosure, February 2026",
        ),
        current(
            "silvertree",
            "ra_platform_fee",
            0.60,
            "Public fee schedule, February 2026",
        ),
        current(
            "silvertree",
            "ra_section14_days",
            52.0,
            "Published service standard, 2026",
        ),
        current(
            "silvertree",
            "ra_onboarding_minutes",
            44.0,
            "Observed public application flow",
        ),
        current(
            "silvertree",
            "ra_fund_range",
            38.0,
            "Public fund list, February 2026",
        ),
        current(
            "silvertree",
            "ra_min_monthly",
            250.0,
            "Public product brochure, 2026",
        ),
        current(
            "silvertree",
            "ra_adviser_fee_options",
            2.0,
            "Public adviser guide, 2026",
        ),
        // --- Retirement annuity: Ridgeline, unchanged dimensions --------
        current(
            "ridgeline",
            "ra_onboarding_minutes",
            9.0,
            "Observed public application flow",
        ),
        current(
            "ridgeline",
            "ra_fund_range",
            61.0,
            "Public fund list, June 2026",
        ),
        current(
            "ridgeline",
            "ra_min_monthly",
            500.0,
            "Public product brochure, 2026",
        ),
        current(
            "ridgeline",
            "ra_adviser_fee_options",
            3.0,
            "Public adviser guide, 2026",
        ),
        // --- Individual life risk ---------------------------------------
        current(
            US,
            "life_underwriting_minutes",
            2880.0,
            "Internal underwriting service report, Q2 2026",
        ),
        current(
            US,
            "life_premium_index",
            104.0,
            "Internal competitive premium study, 2026",
        ),
        current(
            US,
            "life_tele_underwriting",
            55.0,
            "Internal underwriting analytics, Q2 2026",
        ),
        current(
            US,
            "life_cover_without_medicals",
            5.0,
            "Internal underwriting rules 2026.1",
        ),
        current(
            "ridgeline",
            "life_underwriting_minutes",
            14.0,
            "Public product announcement, May 2026",
        ),
        current(
            "ridgeline",
            "life_premium_index",
            98.0,
            "Public premium comparison, 2026",
        ),
        current(
            "ridgeline",
            "life_tele_underwriting",
            92.0,
            "Public product material, 2026",
        ),
        current(
            "ridgeline",
            "life_cover_without_medicals",
            12.0,
            "Public product material, 2026",
        ),
        current(
            "table_bay_mutual",
            "life_underwriting_minutes",
            1440.0,
            "Published service standard, 2026",
        ),
        current(
            "table_bay_mutual",
            "life_premium_index",
            101.0,
            "Public premium comparison, 2026",
        ),
        current(
            "table_bay_mutual",
            "life_tele_underwriting",
            70.0,
            "Public product material, 2026",
        ),
        current(
            "table_bay_mutual",
            "life_cover_without_medicals",
            8.0,
            "Public product material, 2026",
        ),
        current(
            "silvertree",
            "life_underwriting_minutes",
            4320.0,
            "Published service standard, 2026",
        ),
        current(
            "silvertree",
            "life_premium_index",
            107.0,
            "Public premium comparison, 2026",
        ),
        current(
            "silvertree",
            "life_tele_underwriting",
            40.0,
            "Public product material, 2026",
        ),
        current(
            "silvertree",
            "life_cover_without_medicals",
            4.0,
            "Public product material, 2026",
        ),
    ];

    // Ridgeline's repricing: the event that the sponsor's question is about.
    // Each dimension carries the superseded value so the move is recoverable.
    let moves: [(&'static str, f64, f64, i64, &'static str); 4] = [
        (
            "ra_eac_500k",
            1.42,
            0.95,
            75,
            "Public EAC disclosure, May 2026",
        ),
        (
            "ra_eac_2m",
            1.08,
            0.72,
            75,
            "Public EAC disclosure, May 2026",
        ),
        (
            "ra_platform_fee",
            0.45,
            0.30,
            75,
            "Public fee schedule, May 2026",
        ),
        (
            "ra_section14_days",
            31.0,
            12.0,
            110,
            "Published service standard, April 2026",
        ),
    ];

    for (feature, previous, current_value, changed_days_ago, source_reference) in moves {
        values.push(Observation {
            competitor: "ridgeline",
            feature,
            value: previous,
            observed_days_ago: 200,
            superseded_days_ago: Some(changed_days_ago),
            source_reference: "Public disclosure, prior version",
        });
        values.push(Observation {
            competitor: "ridgeline",
            feature,
            value: current_value,
            observed_days_ago: changed_days_ago,
            superseded_days_ago: None,
            source_reference,
        });
    }

    values
}

async fn seed_observations(repository: &Repository) -> Result<(), AppError> {
    for observation in observations() {
        let superseded_at = observation
            .superseded_days_ago
            .map(days_ago_rfc3339)
            .transpose()?;

        query(
            "INSERT INTO feature_observations (id, competitor_id, feature_id, value, source_classification, source_reference, observed_at, superseded_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(observation.competitor)
        .bind(observation.feature)
        .bind(observation.value)
        // Our own values are internal; everything else is public by construction,
        // which is what the competition-information control depends on.
        .bind(if observation.competitor == US {
            "approved_internal"
        } else {
            "public"
        })
        .bind(observation.source_reference)
        .bind(days_ago_rfc3339(observation.observed_days_ago)?)
        .bind(superseded_at)
        .execute(repository.pool())
        .await?;
    }

    Ok(())
}

async fn seed_thresholds(repository: &Repository) -> Result<(), AppError> {
    let thresholds = [
        (
            "position_score_drop",
            3.0,
            "high",
            "Weighted competitive position fell by more than 3 points since the previous run.",
        ),
        (
            "rank_loss",
            1.0,
            "high",
            "We were overtaken by at least one provider since the previous run.",
        ),
        (
            "competitor_move",
            1.0,
            "medium",
            "A competitor improved its own position on a tracked dimension.",
        ),
    ];

    for (metric, threshold, severity, description) in thresholds {
        query(
            "INSERT INTO monitor_thresholds (id, product_line_id, metric, threshold, severity, description) VALUES (?, NULL, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(metric)
        .bind(threshold)
        .bind(severity)
        .bind(description)
        .execute(repository.pool())
        .await?;
    }

    Ok(())
}

/// Synthetic stand-ins for the quote-to-issue dataset. A monitor-opened case
/// needs commercial sizing, and until the insurer data connector exists these
/// are the only figures available.
async fn seed_commercials(repository: &Repository) -> Result<(), AppError> {
    let commercials = [
        (
            RETIREMENT_ANNUITY,
            "Retirement annuity",
            "Independent advisers and direct digital",
            "Pre-retirement savers aged 35–55",
            -2.4_f64,
            312.0_f64,
            1_460.0_f64,
            24.1_f64,
            19.2_f64,
        ),
        (
            INDIVIDUAL_LIFE_RISK,
            "Individual life risk",
            "Independent advisers",
            "Digitally advised professionals aged 28–45",
            -1.8,
            486.0,
            2_187.0,
            31.0,
            24.7,
        ),
    ];

    for (
        line,
        product,
        channel,
        segment,
        share_change_pp,
        at_risk,
        quoted,
        baseline_conversion,
        current_conversion,
    ) in commercials
    {
        query(
            "INSERT INTO product_line_commercials (product_line_id, product, channel, segment, share_change_pp, annual_premium_at_risk_m, annual_quoted_premium_m, conversion_baseline_pct, conversion_current_pct) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(line)
        .bind(product)
        .bind(channel)
        .bind(segment)
        .bind(share_change_pp)
        .bind(at_risk)
        .bind(quoted)
        .bind(baseline_conversion)
        .bind(current_conversion)
        .execute(repository.pool())
        .await?;
    }

    Ok(())
}

/// The 0001 seed predates product lines, so the life-risk case is attached to
/// its line here rather than being rewritten.
async fn backfill_existing_life_case(repository: &Repository) -> Result<(), AppError> {
    query(
        "UPDATE market_cases SET product_line_id = ?, annual_quoted_premium_m = ? WHERE product_line_id IS NULL",
    )
    .bind(INDIVIDUAL_LIFE_RISK)
    .bind(2_187.0_f64)
    .execute(repository.pool())
    .await?;

    Ok(())
}

/// Computes historical snapshots from the observation history using the same
/// scoring rules as a live run, so the trend line is reconstructed rather than
/// invented.
async fn seed_position_history(repository: &Repository) -> Result<(), AppError> {
    let competitors = repository.list_competitors().await?;

    for line in repository.list_product_lines().await? {
        let features = repository.list_feature_definitions(&line.id).await?;
        let observations = repository.list_feature_observations(&line.id).await?;

        for days in SNAPSHOT_DAYS_AGO {
            let captured_at = days_ago_rfc3339(days)?;
            let matrix = comparison::build_matrix(
                ProductLine {
                    id: line.id.clone(),
                    name: line.name.clone(),
                    description: line.description.clone(),
                },
                &competitors,
                &features,
                &observations,
                Some(&captured_at),
                captured_at.clone(),
            )?;

            repository
                .save_position_snapshot(&matrix, &captured_at)
                .await?;
        }
    }

    Ok(())
}
