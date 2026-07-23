PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS signals (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL,
    source_classification TEXT NOT NULL,
    observed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS market_cases (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    product TEXT NOT NULL,
    channel TEXT NOT NULL,
    segment TEXT NOT NULL,
    share_change_pp REAL NOT NULL,
    annual_premium_at_risk_m REAL NOT NULL,
    conversion_baseline_pct REAL NOT NULL,
    conversion_current_pct REAL NOT NULL,
    confidence_pct REAL NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS hypotheses (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES market_cases(id) ON DELETE CASCADE,
    rank INTEGER NOT NULL,
    name TEXT NOT NULL,
    explanation TEXT NOT NULL,
    confidence_pct REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS evidence_items (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES market_cases(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    detail TEXT NOT NULL,
    source_classification TEXT NOT NULL,
    admission_status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS governance_checks (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES market_cases(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    rationale TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS approvals (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES market_cases(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL,
    role_name TEXT NOT NULL,
    status TEXT NOT NULL,
    decided_at TEXT,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS scenario_evaluations (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES market_cases(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    result_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS decisions (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES market_cases(id) ON DELETE CASCADE,
    scenario_id TEXT NOT NULL REFERENCES scenario_evaluations(id),
    decision TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_runs (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES market_cases(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    result_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    case_id TEXT REFERENCES market_cases(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,
    actor_type TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_hypotheses_case ON hypotheses(case_id, rank);
CREATE INDEX IF NOT EXISTS idx_evidence_case ON evidence_items(case_id);
CREATE INDEX IF NOT EXISTS idx_governance_case ON governance_checks(case_id);
CREATE INDEX IF NOT EXISTS idx_approvals_case ON approvals(case_id, sequence);
CREATE INDEX IF NOT EXISTS idx_scenarios_case ON scenario_evaluations(case_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_case ON audit_events(case_id, created_at DESC);
