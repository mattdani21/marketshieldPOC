use serde::Serialize;
use sqlx::{SqlitePool, query, query_as};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        Approval, AuditEvent, CaseDetail, EvidenceItem, GovernanceCheck, Hypothesis, MarketCase,
        ScenarioEvaluation, Signal,
    },
};

#[derive(Clone)]
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn seed_demo_data(&self) -> Result<(), AppError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM market_cases")
            .fetch_one(&self.pool)
            .await?;

        if count > 0 {
            return Ok(());
        }

        let now = now_rfc3339()?;
        let case_id = "MS-2026-017";

        query(
            r#"
            INSERT INTO market_cases (
                id, title, status, product, channel, segment, share_change_pp,
                annual_premium_at_risk_m, conversion_baseline_pct,
                conversion_current_pct, confidence_pct, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(case_id)
        .bind("Young professional life-cover conversion decline")
        .bind("open")
        .bind("Individual life risk")
        .bind("Independent advisers")
        .bind("Digitally advised professionals aged 28–45")
        .bind(-1.8_f64)
        .bind(486.0_f64)
        .bind(31.0_f64)
        .bind(24.7_f64)
        .bind(87.0_f64)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let signals = [
            (
                "Competitor A launches rapid underwriting",
                "Public product material indicates a sub-15-minute decision for qualifying professional lives.",
                "high",
                "public",
            ),
            (
                "Broker quote losses concentrated",
                "Three independent broker groups account for 41% of the deterioration.",
                "medium",
                "approved_internal",
            ),
            (
                "Investment platform fee change",
                "Competitor B reduced administration fees in two high-balance tiers.",
                "low",
                "public",
            ),
            (
                "Retention pilot outperforming",
                "Proactive adviser outreach reduced 90-day lapse risk in the test cohort.",
                "positive",
                "approved_internal",
            ),
        ];

        for (title, description, severity, source_classification) in signals {
            query(
                "INSERT INTO signals (id, title, description, severity, source_classification, observed_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(title)
            .bind(description)
            .bind(severity)
            .bind(source_classification)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        }

        let hypotheses = [
            (
                1_i64,
                "Underwriting turnaround",
                "Strong temporal and adviser-level relationship with lost quotes.",
                87.0_f64,
            ),
            (
                2,
                "Adviser workflow friction",
                "Duplicate capture and medical-evidence follow-ups create abandonment.",
                74.0,
            ),
            (
                3,
                "Premium competitiveness",
                "The price gap exists but explains a smaller portion of lost decisions.",
                48.0,
            ),
            (
                4,
                "Benefit design mismatch",
                "No strong evidence of a material benefit disadvantage.",
                22.0,
            ),
        ];

        for (rank, name, explanation, confidence_pct) in hypotheses {
            query(
                "INSERT INTO hypotheses (id, case_id, rank, name, explanation, confidence_pct) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(case_id)
            .bind(rank)
            .bind(name)
            .bind(explanation)
            .bind(confidence_pct)
            .execute(&self.pool)
            .await?;
        }

        let evidence = [
            (
                "Public competitor brochure",
                "Product feature extraction from a public source.",
                "public",
                "admitted",
            ),
            (
                "Quote and issue funnel",
                "Aggregated internal quote, application and issue data.",
                "approved_internal",
                "admitted",
            ),
            (
                "Adviser loss-reason survey",
                "Aggregated responses from 214 advisers.",
                "consented",
                "admitted",
            ),
            (
                "Customer-level health details",
                "Not required to diagnose the market-share problem.",
                "special_personal_information",
                "excluded",
            ),
            (
                "Non-public competitor pricing",
                "Competitively sensitive information is not admitted.",
                "competitively_sensitive",
                "blocked",
            ),
        ];

        for (label, detail, source_classification, admission_status) in evidence {
            query(
                "INSERT INTO evidence_items (id, case_id, label, detail, source_classification, admission_status) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(case_id)
            .bind(label)
            .bind(detail)
            .bind(source_classification)
            .bind(admission_status)
            .execute(&self.pool)
            .await?;
        }

        let checks = [
            (
                "TCF",
                "Customer outcome assessment",
                "pass",
                "Target market, value, communication, servicing and post-sale outcomes are included.",
            ),
            (
                "POPIA-71",
                "Automated decision control",
                "pass",
                "The solution recommends actions; it does not make a solely automated material customer decision.",
            ),
            (
                "COMP-INFO",
                "Competition information control",
                "pass",
                "Only public, licensed or internally generated evidence is admitted.",
            ),
            (
                "OUTSOURCE",
                "Outsourcing materiality assessment",
                "review",
                "Production hosting and model-provider arrangements require formal assessment.",
            ),
        ];

        for (code, name, status, rationale) in checks {
            query(
                "INSERT INTO governance_checks (id, case_id, code, name, status, rationale) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(case_id)
            .bind(code)
            .bind(name)
            .bind(status)
            .bind(rationale)
            .execute(&self.pool)
            .await?;
        }

        let approvals = [
            (1_i64, "Agent analysis", "approved"),
            (2_i64, "Product actuary", "pending"),
            (3_i64, "Compliance and legal", "waiting"),
            (4_i64, "Risk and model validation", "waiting"),
            (5_i64, "Product committee", "waiting"),
        ];

        for (sequence, role_name, status) in approvals {
            query(
                "INSERT INTO approvals (id, case_id, sequence, role_name, status) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(case_id)
            .bind(sequence)
            .bind(role_name)
            .bind(status)
            .execute(&self.pool)
            .await?;
        }

        self.record_audit(
            Some(case_id),
            "demo_seeded",
            "system",
            "seed_demo_data",
            &serde_json::json!({ "case_id": case_id }),
        )
        .await?;

        Ok(())
    }

    pub async fn list_signals(&self) -> Result<Vec<Signal>, AppError> {
        Ok(query_as::<_, Signal>(
            "SELECT id, title, description, severity, source_classification, observed_at FROM signals ORDER BY observed_at DESC",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn list_cases(&self) -> Result<Vec<MarketCase>, AppError> {
        Ok(
            query_as::<_, MarketCase>("SELECT * FROM market_cases ORDER BY updated_at DESC")
                .fetch_all(&self.pool)
                .await?,
        )
    }

    pub async fn get_case(&self, case_id: &str) -> Result<MarketCase, AppError> {
        query_as::<_, MarketCase>("SELECT * FROM market_cases WHERE id = ?")
            .bind(case_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("case {case_id}")))
    }

    pub async fn get_case_detail(&self, case_id: &str) -> Result<CaseDetail, AppError> {
        let case = self.get_case(case_id).await?;
        let hypotheses =
            query_as::<_, Hypothesis>("SELECT * FROM hypotheses WHERE case_id = ? ORDER BY rank")
                .bind(case_id)
                .fetch_all(&self.pool)
                .await?;
        let evidence = query_as::<_, EvidenceItem>(
            "SELECT * FROM evidence_items WHERE case_id = ? ORDER BY rowid",
        )
        .bind(case_id)
        .fetch_all(&self.pool)
        .await?;
        let governance_checks = query_as::<_, GovernanceCheck>(
            "SELECT * FROM governance_checks WHERE case_id = ? ORDER BY rowid",
        )
        .bind(case_id)
        .fetch_all(&self.pool)
        .await?;
        let approvals =
            query_as::<_, Approval>("SELECT * FROM approvals WHERE case_id = ? ORDER BY sequence")
                .bind(case_id)
                .fetch_all(&self.pool)
                .await?;

        Ok(CaseDetail {
            case,
            hypotheses,
            evidence,
            governance_checks,
            approvals,
        })
    }

    pub async fn save_agent_run<T: Serialize>(
        &self,
        case_id: &str,
        status: &str,
        result: &T,
    ) -> Result<String, AppError> {
        let id = Uuid::new_v4().to_string();
        query(
            "INSERT INTO agent_runs (id, case_id, status, result_json, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(case_id)
        .bind(status)
        .bind(serde_json::to_string(result).map_err(|error| AppError::Internal(error.to_string()))?)
        .bind(now_rfc3339()?)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn save_scenario(&self, evaluation: &ScenarioEvaluation) -> Result<(), AppError> {
        query(
            "INSERT INTO scenario_evaluations (id, case_id, kind, result_json, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&evaluation.id)
        .bind(&evaluation.case_id)
        .bind(evaluation.kind.as_str())
        .bind(
            serde_json::to_string(evaluation)
                .map_err(|error| AppError::Internal(error.to_string()))?,
        )
        .bind(&evaluation.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_scenario(&self, scenario_id: &str) -> Result<ScenarioEvaluation, AppError> {
        let json: Option<String> =
            sqlx::query_scalar("SELECT result_json FROM scenario_evaluations WHERE id = ?")
                .bind(scenario_id)
                .fetch_optional(&self.pool)
                .await?;

        let json = json.ok_or_else(|| AppError::NotFound(format!("scenario {scenario_id}")))?;
        serde_json::from_str(&json).map_err(|error| AppError::Internal(error.to_string()))
    }

    pub async fn save_decision(
        &self,
        case_id: &str,
        scenario_id: &str,
        decision: &str,
        notes: Option<&str>,
    ) -> Result<String, AppError> {
        let id = Uuid::new_v4().to_string();
        query(
            "INSERT INTO decisions (id, case_id, scenario_id, decision, notes, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(case_id)
        .bind(scenario_id)
        .bind(decision)
        .bind(notes)
        .bind(now_rfc3339()?)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn next_pending_approval(&self, case_id: &str) -> Result<Option<Approval>, AppError> {
        Ok(query_as::<_, Approval>(
            "SELECT * FROM approvals WHERE case_id = ? AND status = 'pending' ORDER BY sequence LIMIT 1",
        )
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn advance_approval(
        &self,
        case_id: &str,
        decision: &str,
        notes: Option<&str>,
    ) -> Result<Vec<Approval>, AppError> {
        let pending = self.next_pending_approval(case_id).await?;
        let pending = pending
            .ok_or_else(|| AppError::BadRequest("no pending approval remains".to_string()))?;
        let final_status = match decision {
            "approve" => "approved",
            "reject" => "rejected",
            _ => {
                return Err(AppError::BadRequest(
                    "decision must be approve or reject".to_string(),
                ));
            }
        };

        query("UPDATE approvals SET status = ?, decided_at = ?, notes = ? WHERE id = ?")
            .bind(final_status)
            .bind(now_rfc3339()?)
            .bind(notes)
            .bind(&pending.id)
            .execute(&self.pool)
            .await?;

        if final_status == "approved" {
            query(
                "UPDATE approvals SET status = 'pending' WHERE case_id = ? AND sequence = ? AND status = 'waiting'",
            )
            .bind(case_id)
            .bind(pending.sequence + 1)
            .execute(&self.pool)
            .await?;
        } else {
            query("UPDATE market_cases SET status = 'stopped', updated_at = ? WHERE id = ?")
                .bind(now_rfc3339()?)
                .bind(case_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(
            query_as::<_, Approval>("SELECT * FROM approvals WHERE case_id = ? ORDER BY sequence")
                .bind(case_id)
                .fetch_all(&self.pool)
                .await?,
        )
    }

    pub async fn list_audit_events(
        &self,
        case_id: Option<&str>,
    ) -> Result<Vec<AuditEvent>, AppError> {
        let events = if let Some(case_id) = case_id {
            query_as::<_, AuditEvent>(
                "SELECT * FROM audit_events WHERE case_id = ? ORDER BY created_at DESC LIMIT 100",
            )
            .bind(case_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            query_as::<_, AuditEvent>(
                "SELECT * FROM audit_events ORDER BY created_at DESC LIMIT 100",
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(events)
    }

    pub async fn record_audit<T: Serialize>(
        &self,
        case_id: Option<&str>,
        event_type: &str,
        actor_type: &str,
        actor_name: &str,
        payload: &T,
    ) -> Result<(), AppError> {
        query(
            "INSERT INTO audit_events (id, case_id, event_type, actor_type, actor_name, payload_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(case_id)
        .bind(event_type)
        .bind(actor_type)
        .bind(actor_name)
        .bind(serde_json::to_string(payload).map_err(|error| AppError::Internal(error.to_string()))?)
        .bind(now_rfc3339()?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

pub fn now_rfc3339() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| AppError::Internal(format!("failed to format timestamp: {error}")))
}
