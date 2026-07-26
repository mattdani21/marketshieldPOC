use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Signal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub source_classification: String,
    pub observed_at: String,
    pub product_line_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MarketCase {
    pub id: String,
    pub title: String,
    pub status: String,
    pub product: String,
    pub channel: String,
    pub segment: String,
    pub share_change_pp: f64,
    pub annual_premium_at_risk_m: f64,
    pub conversion_baseline_pct: f64,
    pub conversion_current_pct: f64,
    pub confidence_pct: f64,
    pub created_at: String,
    pub updated_at: String,
    pub product_line_id: Option<String>,
    /// Annual quoted premium the case's conversion movement applies to. Drives
    /// scenario sizing, so retirement annuity and life-risk cases scale apart.
    pub annual_quoted_premium_m: f64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Hypothesis {
    pub id: String,
    pub case_id: String,
    pub rank: i64,
    pub name: String,
    pub explanation: String,
    pub confidence_pct: f64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EvidenceItem {
    pub id: String,
    pub case_id: String,
    pub label: String,
    pub detail: String,
    pub source_classification: String,
    pub admission_status: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct GovernanceCheck {
    pub id: String,
    pub case_id: String,
    pub code: String,
    pub name: String,
    pub status: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Approval {
    pub id: String,
    pub case_id: String,
    pub sequence: i64,
    pub role_name: String,
    pub status: String,
    pub decided_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AuditEvent {
    pub id: String,
    pub case_id: Option<String>,
    pub event_type: String,
    pub actor_type: String,
    pub actor_name: String,
    pub payload_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummary {
    pub estimated_segment_share_pct: f64,
    pub share_change_pp: f64,
    pub annual_premium_at_risk_m: f64,
    pub quote_conversion_pct: f64,
    pub conversion_change_pp: f64,
    pub open_cases: i64,
    pub signals: Vec<Signal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaseDetail {
    pub case: MarketCase,
    pub hypotheses: Vec<Hypothesis>,
    pub evidence: Vec<EvidenceItem>,
    pub governance_checks: Vec<GovernanceCheck>,
    pub approvals: Vec<Approval>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStepResult {
    pub step: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentRunResult {
    pub run_id: String,
    pub case_id: String,
    pub status: String,
    pub primary_finding: String,
    pub steps: Vec<AgentStepResult>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    RapidUnderwriting,
    TargetedPremiumReduction,
    BenefitRewardsBundle,
    RaFeeRestructure,
    RaTransferTurnaround,
    RaFundRangeExpansion,
    ObserveOnly,
}

impl ScenarioKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RapidUnderwriting => "rapid_underwriting",
            Self::TargetedPremiumReduction => "targeted_premium_reduction",
            Self::BenefitRewardsBundle => "benefit_rewards_bundle",
            Self::RaFeeRestructure => "ra_fee_restructure",
            Self::RaTransferTurnaround => "ra_transfer_turnaround",
            Self::RaFundRangeExpansion => "ra_fund_range_expansion",
            Self::ObserveOnly => "observe_only",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::RapidUnderwriting => "Rapid underwriting pilot",
            Self::TargetedPremiumReduction => "Targeted premium reduction",
            Self::BenefitRewardsBundle => "Benefit and rewards bundle",
            Self::RaFeeRestructure => "Retirement annuity fee restructure",
            Self::RaTransferTurnaround => "Section 14 transfer turnaround programme",
            Self::RaFundRangeExpansion => "Retirement annuity fund range expansion",
            Self::ObserveOnly => "No material product change",
        }
    }

    /// Product line a response template is designed for. Used to stop a
    /// life-risk response being evaluated against a savings case.
    pub fn product_line_id(self) -> Option<&'static str> {
        match self {
            Self::RapidUnderwriting
            | Self::TargetedPremiumReduction
            | Self::BenefitRewardsBundle => Some("individual_life_risk"),
            Self::RaFeeRestructure | Self::RaTransferTurnaround | Self::RaFundRangeExpansion => {
                Some("retirement_annuity")
            }
            Self::ObserveOnly => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvaluateScenarioRequest {
    pub case_id: String,
    pub kind: ScenarioKind,
    #[serde(default)]
    pub overrides: ScenarioOverrides,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScenarioOverrides {
    pub conversion_gain_pp: Option<f64>,
    pub implementation_uptake_pct: Option<f64>,
    pub new_business_margin_pct: Option<f64>,
    pub claims_index: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioMetric {
    pub key: String,
    pub label: String,
    pub baseline: f64,
    pub scenario: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressResult {
    pub name: String,
    pub status: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioEvaluation {
    pub id: String,
    pub case_id: String,
    pub kind: ScenarioKind,
    pub name: String,
    pub recommendation_status: String,
    pub narrative: String,
    pub metrics: Vec<ScenarioMetric>,
    pub scores: std::collections::BTreeMap<String, u8>,
    pub stresses: Vec<StressResult>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitDecisionRequest {
    pub case_id: String,
    pub scenario_id: String,
    pub decision: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionReceipt {
    pub decision_id: String,
    pub case_id: String,
    pub scenario_id: String,
    pub decision: String,
    pub next_approval_role: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdvanceApprovalRequest {
    pub decision: String,
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// Competitive intelligence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProductLine {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Competitor {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub is_us: i64,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    LowerIsBetter,
    HigherIsBetter,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LowerIsBetter => "lower_is_better",
            Self::HigherIsBetter => "higher_is_better",
        }
    }

    pub fn parse(value: &str) -> Result<Self, crate::error::AppError> {
        match value {
            "lower_is_better" => Ok(Self::LowerIsBetter),
            "higher_is_better" => Ok(Self::HigherIsBetter),
            other => Err(crate::error::AppError::Internal(format!(
                "unknown feature direction: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FeatureDefinition {
    pub id: String,
    pub product_line_id: String,
    pub name: String,
    pub unit: String,
    pub direction: String,
    pub weight: f64,
    pub display_order: i64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FeatureObservation {
    pub id: String,
    pub competitor_id: String,
    pub feature_id: String,
    pub value: f64,
    pub source_classification: String,
    pub source_reference: String,
    pub observed_at: String,
    pub superseded_at: Option<String>,
}

/// One provider's standing on one comparison dimension.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderValue {
    pub competitor_id: String,
    pub competitor_name: String,
    pub is_us: bool,
    pub value: f64,
    pub score: f64,
    pub is_best: bool,
    pub source_classification: String,
    pub source_reference: String,
    pub observed_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureComparison {
    pub feature_id: String,
    pub name: String,
    pub unit: String,
    pub direction: String,
    pub weight: f64,
    pub description: String,
    pub our_value: Option<f64>,
    pub our_score: Option<f64>,
    pub best_value: f64,
    pub best_provider: String,
    /// Signed distance from the best value, in the feature's own unit, always
    /// positive when we are behind.
    pub gap_to_best: Option<f64>,
    pub providers: Vec<ProviderValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Standing {
    pub competitor_id: String,
    pub competitor_name: String,
    pub is_us: bool,
    pub position_score: f64,
    pub rank: i64,
    pub features_covered: usize,
}

/// A competitor value that changed: the provenance-carrying record of a move.
#[derive(Debug, Clone, Serialize)]
pub struct CompetitorMove {
    pub competitor_id: String,
    pub competitor_name: String,
    pub feature_id: String,
    pub feature_name: String,
    pub unit: String,
    pub previous_value: f64,
    pub current_value: f64,
    pub changed_at: String,
    /// True when the move made that provider more competitive.
    pub favourable_to_them: bool,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureGap {
    pub feature_id: String,
    pub feature_name: String,
    pub unit: String,
    pub our_value: f64,
    pub best_value: f64,
    pub best_provider: String,
    pub gap: f64,
    pub weight: f64,
    /// Weight-adjusted score deficit; the ranking key for "what is hurting us most".
    pub weighted_deficit: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OurPosition {
    pub position_score: f64,
    pub rank: i64,
    pub provider_count: i64,
    pub leader_competitor_id: String,
    pub leader_name: String,
    pub score_behind_leader: f64,
    pub largest_gaps: Vec<FeatureGap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonMatrix {
    pub product_line: ProductLine,
    pub generated_at: String,
    pub features: Vec<FeatureComparison>,
    pub standings: Vec<Standing>,
    pub our_position: OurPosition,
    pub recent_moves: Vec<CompetitorMove>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PositionSnapshot {
    pub id: String,
    pub product_line_id: String,
    pub captured_at: String,
    pub position_score: f64,
    pub our_rank: i64,
    pub provider_count: i64,
    pub leader_competitor_id: String,
    pub detail_json: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProductLineCommercials {
    pub product_line_id: String,
    pub product: String,
    pub channel: String,
    pub segment: String,
    pub share_change_pp: f64,
    pub annual_premium_at_risk_m: f64,
    pub annual_quoted_premium_m: f64,
    pub conversion_baseline_pct: f64,
    pub conversion_current_pct: f64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MonitorThreshold {
    pub id: String,
    pub product_line_id: Option<String>,
    pub metric: String,
    pub threshold: f64,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorFinding {
    pub code: String,
    pub severity: String,
    pub product_line_id: String,
    pub product_line_name: String,
    pub headline: String,
    pub detail: String,
    pub observed_value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductLineMonitorResult {
    pub product_line_id: String,
    pub product_line_name: String,
    pub position_score: f64,
    pub previous_position_score: Option<f64>,
    pub score_change: Option<f64>,
    pub rank: i64,
    pub previous_rank: Option<i64>,
    pub provider_count: i64,
    pub leader_name: String,
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorRunResult {
    pub run_id: String,
    pub started_at: String,
    pub trigger: String,
    pub product_lines: Vec<ProductLineMonitorResult>,
    pub findings: Vec<MonitorFinding>,
    pub signals_created: Vec<String>,
    pub cases_opened: Vec<String>,
    /// True when any finding reached the severity that requires human attention.
    pub breached: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MonitorRunRequest {
    #[serde(default)]
    pub trigger: Option<String>,
    /// When false, the run reports drift without creating signals or cases.
    #[serde(default = "default_true")]
    pub raise_signals: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordObservationRequest {
    pub competitor_id: String,
    pub feature_id: String,
    pub value: f64,
    pub source_classification: String,
    pub source_reference: String,
}
