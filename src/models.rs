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
    ObserveOnly,
}

impl ScenarioKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RapidUnderwriting => "rapid_underwriting",
            Self::TargetedPremiumReduction => "targeted_premium_reduction",
            Self::BenefitRewardsBundle => "benefit_rewards_bundle",
            Self::ObserveOnly => "observe_only",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::RapidUnderwriting => "Rapid underwriting pilot",
            Self::TargetedPremiumReduction => "Targeted premium reduction",
            Self::BenefitRewardsBundle => "Benefit and rewards bundle",
            Self::ObserveOnly => "No material product change",
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
