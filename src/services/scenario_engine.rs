use std::collections::BTreeMap;

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        EvaluateScenarioRequest, MarketCase, ScenarioEvaluation, ScenarioKind, ScenarioMetric,
        StressResult,
    },
    repository::now_rfc3339,
};

/// Fallback quoted-premium base for cases seeded before the column existed.
const DEFAULT_ANNUAL_QUOTED_PREMIUM_M: f64 = 2_187.0;

struct Template {
    conversion_gain_pp: f64,
    uptake_pct: f64,
    margin_pct: f64,
    baseline_margin_pct: f64,
    claims_index: f64,
    capital_ratio: f64,
    growth_score: u8,
    profitability_score: u8,
    customer_score: u8,
    delivery_score: u8,
    confidence_score: u8,
    narrative: &'static str,
}

pub fn evaluate(
    case: &MarketCase,
    request: EvaluateScenarioRequest,
) -> Result<ScenarioEvaluation, AppError> {
    if request.case_id != case.id {
        return Err(AppError::BadRequest(
            "request case_id does not match the loaded case".to_string(),
        ));
    }

    if let (Some(template_line), Some(case_line)) = (
        request.kind.product_line_id(),
        case.product_line_id.as_deref(),
    ) && template_line != case_line
    {
        return Err(AppError::BadRequest(format!(
            "response template {} applies to {template_line}, but case {} is a {case_line} case",
            request.kind.as_str(),
            case.id
        )));
    }

    let template = template(request.kind);
    let conversion_gain_pp = request
        .overrides
        .conversion_gain_pp
        .unwrap_or(template.conversion_gain_pp);
    let uptake_pct = request
        .overrides
        .implementation_uptake_pct
        .unwrap_or(template.uptake_pct);
    let margin_pct = request
        .overrides
        .new_business_margin_pct
        .unwrap_or(template.margin_pct);
    let claims_index = request
        .overrides
        .claims_index
        .unwrap_or(template.claims_index);

    validate_range("conversion_gain_pp", conversion_gain_pp, -10.0, 20.0)?;
    validate_range("implementation_uptake_pct", uptake_pct, 0.0, 100.0)?;
    validate_range("new_business_margin_pct", margin_pct, -20.0, 50.0)?;
    validate_range("claims_index", claims_index, 50.0, 200.0)?;

    let quoted_premium_base_m = if case.annual_quoted_premium_m > 0.0 {
        case.annual_quoted_premium_m
    } else {
        DEFAULT_ANNUAL_QUOTED_PREMIUM_M
    };

    let scenario_conversion = case.conversion_current_pct + conversion_gain_pp;
    let annualised_nbp_uplift_m =
        quoted_premium_base_m * (conversion_gain_pp / 100.0) * (uptake_pct / 100.0);
    let vnb_uplift_m = annualised_nbp_uplift_m * (margin_pct / 100.0);
    let capital_strain_m = annualised_nbp_uplift_m * template.capital_ratio;

    let metrics = vec![
        metric(
            "conversion",
            "Quote conversion",
            case.conversion_current_pct,
            scenario_conversion,
            "percent",
        ),
        metric(
            "nbp",
            "Annualised new-business premium uplift",
            0.0,
            annualised_nbp_uplift_m,
            "zar_millions",
        ),
        metric(
            "vnb",
            "Value of new business uplift",
            0.0,
            vnb_uplift_m,
            "zar_millions",
        ),
        metric(
            "margin",
            "New-business margin",
            template.baseline_margin_pct,
            margin_pct,
            "percent",
        ),
        metric(
            "capital",
            "Initial capital strain",
            0.0,
            capital_strain_m,
            "zar_millions",
        ),
        metric(
            "claims",
            "Expected claims index",
            100.0,
            claims_index,
            "index",
        ),
    ];

    let stresses = stress_results(request.kind, margin_pct, claims_index);
    let failed = stresses
        .iter()
        .filter(|stress| stress.status == "fail")
        .count();
    let recommendation_status = match failed {
        0 => "viable",
        1 => "conditional",
        _ => "not_preferred",
    }
    .to_string();

    let scores = BTreeMap::from([
        ("growth".to_string(), template.growth_score),
        ("profitability".to_string(), template.profitability_score),
        ("customer_outcomes".to_string(), template.customer_score),
        ("delivery".to_string(), template.delivery_score),
        ("confidence".to_string(), template.confidence_score),
    ]);

    Ok(ScenarioEvaluation {
        id: Uuid::new_v4().to_string(),
        case_id: case.id.clone(),
        kind: request.kind,
        name: request.kind.display_name().to_string(),
        recommendation_status,
        narrative: template.narrative.to_string(),
        metrics,
        scores,
        stresses,
        created_at: now_rfc3339()?,
    })
}

fn template(kind: ScenarioKind) -> Template {
    match kind {
        ScenarioKind::RapidUnderwriting => Template {
            conversion_gain_pp: 3.8,
            uptake_pct: 65.0,
            margin_pct: 13.2,
            baseline_margin_pct: 14.8,
            claims_index: 101.5,
            capital_ratio: 0.20,
            growth_score: 86,
            profitability_score: 78,
            customer_score: 88,
            delivery_score: 72,
            confidence_score: 84,
            narrative: "Best balanced response. It targets the strongest diagnosed cause while limiting broad price and claims risk.",
        },
        ScenarioKind::TargetedPremiumReduction => Template {
            conversion_gain_pp: 2.9,
            uptake_pct: 74.0,
            margin_pct: 7.3,
            baseline_margin_pct: 14.8,
            claims_index: 100.0,
            capital_ratio: 0.26,
            growth_score: 76,
            profitability_score: 45,
            customer_score: 74,
            delivery_score: 91,
            confidence_score: 61,
            narrative: "Fast to implement but economically weaker. It may treat the symptom while leaving workflow friction intact.",
        },
        ScenarioKind::BenefitRewardsBundle => Template {
            conversion_gain_pp: 2.1,
            uptake_pct: 83.0,
            margin_pct: 11.4,
            baseline_margin_pct: 14.8,
            claims_index: 103.2,
            capital_ratio: 0.36,
            growth_score: 65,
            profitability_score: 69,
            customer_score: 77,
            delivery_score: 43,
            confidence_score: 55,
            narrative: "Potential proposition value, but delivery is slower and the evidence does not show benefits as the primary loss driver.",
        },
        ScenarioKind::RaFeeRestructure => Template {
            conversion_gain_pp: 4.4,
            uptake_pct: 71.0,
            margin_pct: 5.9,
            baseline_margin_pct: 8.6,
            claims_index: 100.0,
            capital_ratio: 0.09,
            growth_score: 88,
            profitability_score: 52,
            customer_score: 91,
            delivery_score: 66,
            confidence_score: 79,
            narrative: "Directly closes the diagnosed effective-annual-cost gap. It is the strongest growth response, but it permanently resets margin on the existing book as well as new business.",
        },
        ScenarioKind::RaTransferTurnaround => Template {
            conversion_gain_pp: 3.1,
            uptake_pct: 78.0,
            margin_pct: 8.4,
            baseline_margin_pct: 8.6,
            claims_index: 100.0,
            capital_ratio: 0.06,
            growth_score: 74,
            profitability_score: 81,
            customer_score: 83,
            delivery_score: 58,
            confidence_score: 72,
            narrative: "Attacks the Section 14 transfer turnaround gap without repricing. It protects margin, but it is an operations programme rather than a product change and lands more slowly.",
        },
        ScenarioKind::RaFundRangeExpansion => Template {
            conversion_gain_pp: 1.4,
            uptake_pct: 61.0,
            margin_pct: 8.5,
            baseline_margin_pct: 8.6,
            claims_index: 100.0,
            capital_ratio: 0.05,
            growth_score: 46,
            profitability_score: 77,
            customer_score: 62,
            delivery_score: 71,
            confidence_score: 44,
            narrative: "Cheap to deliver, but the evidence ranks fund choice well below cost and transfer speed as a loss driver. Expect a limited share response.",
        },
        ScenarioKind::ObserveOnly => Template {
            conversion_gain_pp: 0.5,
            uptake_pct: 82.0,
            margin_pct: 14.9,
            baseline_margin_pct: 14.8,
            claims_index: 100.0,
            capital_ratio: 0.47,
            growth_score: 28,
            profitability_score: 83,
            customer_score: 52,
            delivery_score: 96,
            confidence_score: 48,
            narrative: "Low execution risk, but likely to allow further profitable share loss if the diagnosis is correct.",
        },
    }
}

fn stress_results(kind: ScenarioKind, margin_pct: f64, claims_index: f64) -> Vec<StressResult> {
    let mut results = Vec::new();

    match kind {
        ScenarioKind::RapidUnderwriting => {
            results.push(stress(
                "Claims deterioration",
                if margin_pct >= 10.0 { "pass" } else { "fail" },
                "Margin remains above the demo pilot threshold after an adverse selection allowance.",
            ));
            results.push(stress(
                "Lower adoption",
                "pass",
                "The value case remains positive at 50% of expected uptake.",
            ));
            results.push(stress(
                "Fairness review",
                "review",
                "Eligibility criteria require compliance challenge and customer-outcome monitoring.",
            ));
        }
        ScenarioKind::TargetedPremiumReduction => {
            results.push(stress(
                "Margin resilience",
                if margin_pct >= 10.0 { "pass" } else { "fail" },
                "The broad discount materially reduces the new-business margin.",
            ));
            results.push(stress(
                "Competitor response",
                "fail",
                "The option creates a credible risk of price escalation without removing workflow friction.",
            ));
            results.push(stress(
                "Fairness review",
                "pass",
                "The segment criteria can be disclosed and consistently applied.",
            ));
        }
        ScenarioKind::BenefitRewardsBundle => {
            results.push(stress(
                "Claims and utilisation",
                if claims_index <= 102.0 {
                    "pass"
                } else {
                    "review"
                },
                "Additional behavioural and utilisation evidence is required.",
            ));
            results.push(stress(
                "Delivery delay",
                "fail",
                "The expected lead time may miss the current competitive response window.",
            ));
            results.push(stress(
                "Customer clarity",
                "review",
                "Benefit conditions require consumer testing and plain-language review.",
            ));
        }
        ScenarioKind::RaFeeRestructure => {
            results.push(stress(
                "Margin resilience",
                if margin_pct >= 6.0 { "pass" } else { "fail" },
                "The restructure holds above the savings-business margin floor used in this demonstration, with little headroom.",
            ));
            results.push(stress(
                "Existing book repricing",
                "review",
                "Applying the new fee basis to in-force policies is required for fair treatment, and materially widens the cost of the response.",
            ));
            results.push(stress(
                "Competitor response",
                "review",
                "The leading competitor retains room to price below the proposed basis.",
            ));
        }
        ScenarioKind::RaTransferTurnaround => {
            results.push(stress(
                "Margin resilience",
                if margin_pct >= 6.0 { "pass" } else { "fail" },
                "The response is operational, so the new-business margin is largely preserved.",
            ));
            results.push(stress(
                "Delivery capacity",
                "review",
                "The turnaround target depends on transfer administration capacity and third-party fund-house response times.",
            ));
            results.push(stress(
                "Residual cost gap",
                "fail",
                "Faster transfers do not close the diagnosed effective-annual-cost gap, which remains the largest single deficit.",
            ));
        }
        ScenarioKind::RaFundRangeExpansion => {
            results.push(stress(
                "Evidence strength",
                "fail",
                "Fund choice ranks below cost and transfer speed in the diagnosed loss drivers, so the expected share response is weak.",
            ));
            results.push(stress(
                "Reg 28 and due diligence",
                "review",
                "Each added portfolio requires Regulation 28 compliance and manager due diligence.",
            ));
            results.push(stress(
                "Residual cost gap",
                "fail",
                "The response leaves both the effective-annual-cost and transfer-turnaround gaps fully open.",
            ));
        }
        ScenarioKind::ObserveOnly => {
            results.push(stress(
                "Further share decline",
                "fail",
                "The material economic exposure remains substantially unaddressed.",
            ));
            results.push(stress(
                "False signal",
                "pass",
                "The option avoids unnecessary product or pricing changes if the diagnosis proves wrong.",
            ));
            results.push(stress(
                "Adviser confidence",
                "fail",
                "The leading workflow and turnaround issue remains unresolved.",
            ));
        }
    }

    results
}

fn metric(key: &str, label: &str, baseline: f64, scenario: f64, unit: &str) -> ScenarioMetric {
    ScenarioMetric {
        key: key.to_string(),
        label: label.to_string(),
        baseline,
        scenario,
        unit: unit.to_string(),
    }
}

fn stress(name: &str, status: &str, rationale: &str) -> StressResult {
    StressResult {
        name: name.to_string(),
        status: status.to_string(),
        rationale: rationale.to_string(),
    }
}

fn validate_range(name: &str, value: f64, min: f64, max: f64) -> Result<(), AppError> {
    if !(min..=max).contains(&value) {
        return Err(AppError::BadRequest(format!(
            "{name} must be between {min} and {max}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_case() -> MarketCase {
        MarketCase {
            id: "MS-TEST".to_string(),
            title: "Test".to_string(),
            status: "open".to_string(),
            product: "Life".to_string(),
            channel: "Adviser".to_string(),
            segment: "Test segment".to_string(),
            share_change_pp: -1.0,
            annual_premium_at_risk_m: 100.0,
            conversion_baseline_pct: 31.0,
            conversion_current_pct: 24.7,
            confidence_pct: 80.0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            product_line_id: Some("individual_life_risk".to_string()),
            annual_quoted_premium_m: 2_187.0,
        }
    }

    fn ra_case() -> MarketCase {
        MarketCase {
            id: "MS-RA-TEST".to_string(),
            product_line_id: Some("retirement_annuity".to_string()),
            annual_quoted_premium_m: 1_460.0,
            conversion_current_pct: 19.2,
            ..demo_case()
        }
    }

    #[test]
    fn rapid_underwriting_is_viable_by_default() {
        let evaluation = evaluate(
            &demo_case(),
            EvaluateScenarioRequest {
                case_id: "MS-TEST".to_string(),
                kind: ScenarioKind::RapidUnderwriting,
                overrides: Default::default(),
            },
        )
        .expect("scenario should evaluate");

        assert_eq!(evaluation.recommendation_status, "viable");
        assert!(evaluation.metrics.iter().any(|metric| metric.key == "vnb"));
    }

    fn evaluate_kind(case: &MarketCase, kind: ScenarioKind) -> ScenarioEvaluation {
        evaluate(
            case,
            EvaluateScenarioRequest {
                case_id: case.id.clone(),
                kind,
                overrides: Default::default(),
            },
        )
        .expect("scenario should evaluate")
    }

    fn metric_value(evaluation: &ScenarioEvaluation, key: &str) -> f64 {
        evaluation
            .metrics
            .iter()
            .find(|metric| metric.key == key)
            .unwrap_or_else(|| panic!("metric {key} should be present"))
            .scenario
    }

    /// Golden numbers. These values reach a product committee, so a refactor
    /// that moves them should fail loudly rather than pass quietly.
    #[test]
    fn ra_fee_restructure_economics_are_stable() {
        let evaluation = evaluate_kind(&ra_case(), ScenarioKind::RaFeeRestructure);

        assert_eq!(evaluation.recommendation_status, "conditional");
        assert!((metric_value(&evaluation, "conversion") - 23.6).abs() < 1e-6);
        assert!((metric_value(&evaluation, "nbp") - 45.6104).abs() < 1e-4);
        assert!((metric_value(&evaluation, "vnb") - 2.6910136).abs() < 1e-4);
        assert!((metric_value(&evaluation, "capital") - 4.104936).abs() < 1e-4);
    }

    #[test]
    fn ra_fund_range_expansion_is_not_preferred() {
        let evaluation = evaluate_kind(&ra_case(), ScenarioKind::RaFundRangeExpansion);
        assert_eq!(evaluation.recommendation_status, "not_preferred");
    }

    #[test]
    fn transfer_turnaround_protects_margin_better_than_fee_restructure() {
        let ra_case = ra_case();
        let fee = evaluate_kind(&ra_case, ScenarioKind::RaFeeRestructure);
        let transfer = evaluate_kind(&ra_case, ScenarioKind::RaTransferTurnaround);

        assert!(metric_value(&transfer, "margin") > metric_value(&fee, "margin"));
        assert!(metric_value(&fee, "nbp") > metric_value(&transfer, "nbp"));
    }

    /// Scenario sizing must follow the case, not a global life-risk constant.
    #[test]
    fn uplift_scales_with_the_case_premium_base() {
        let mut small = ra_case();
        small.annual_quoted_premium_m = 1_000.0;
        let mut large = ra_case();
        large.annual_quoted_premium_m = 2_000.0;

        let small_uplift = metric_value(
            &evaluate_kind(&small, ScenarioKind::RaFeeRestructure),
            "nbp",
        );
        let large_uplift = metric_value(
            &evaluate_kind(&large, ScenarioKind::RaFeeRestructure),
            "nbp",
        );

        assert!((large_uplift - small_uplift * 2.0).abs() < 1e-6);
    }

    #[test]
    fn life_risk_template_is_rejected_for_a_retirement_annuity_case() {
        let result = evaluate(
            &ra_case(),
            EvaluateScenarioRequest {
                case_id: "MS-RA-TEST".to_string(),
                kind: ScenarioKind::RapidUnderwriting,
                overrides: Default::default(),
            },
        );

        assert!(
            result.is_err(),
            "cross-product-line template must be refused"
        );
    }

    #[test]
    fn observe_only_applies_to_any_product_line() {
        for case in [demo_case(), ra_case()] {
            let evaluation = evaluate_kind(&case, ScenarioKind::ObserveOnly);
            assert_eq!(evaluation.recommendation_status, "not_preferred");
        }
    }

    #[test]
    fn extreme_override_is_rejected() {
        let result = evaluate(
            &demo_case(),
            EvaluateScenarioRequest {
                case_id: "MS-TEST".to_string(),
                kind: ScenarioKind::RapidUnderwriting,
                overrides: crate::models::ScenarioOverrides {
                    conversion_gain_pp: Some(99.0),
                    ..Default::default()
                },
            },
        );

        assert!(result.is_err());
    }
}
