-- Competitive intelligence model.
--
-- The 0001 schema records what we do about a competitive problem once a human
-- has noticed it. This schema records the thing itself: what each provider
-- offers, on the dimensions the product is actually chosen on, over time.

CREATE TABLE IF NOT EXISTS product_lines (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS competitors (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    short_name TEXT NOT NULL,
    -- Exactly one competitor row represents our own offering. Keeping "us" in
    -- the same table means every comparison is computed the same way.
    is_us INTEGER NOT NULL DEFAULT 0,
    notes TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS feature_definitions (
    id TEXT PRIMARY KEY,
    product_line_id TEXT NOT NULL REFERENCES product_lines(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    unit TEXT NOT NULL,
    -- 'lower_is_better' or 'higher_is_better'; decides which end of the range wins.
    direction TEXT NOT NULL,
    -- Relative importance when rolling features up into a position score.
    weight REAL NOT NULL,
    display_order INTEGER NOT NULL,
    description TEXT NOT NULL
);

-- Append-only observations. A value is current while superseded_at IS NULL;
-- writing a new value supersedes the old one, so competitor movement is a
-- first-class fact rather than something lost on overwrite.
CREATE TABLE IF NOT EXISTS feature_observations (
    id TEXT PRIMARY KEY,
    competitor_id TEXT NOT NULL REFERENCES competitors(id) ON DELETE CASCADE,
    feature_id TEXT NOT NULL REFERENCES feature_definitions(id) ON DELETE CASCADE,
    value REAL NOT NULL,
    source_classification TEXT NOT NULL,
    source_reference TEXT NOT NULL,
    observed_at TEXT NOT NULL,
    superseded_at TEXT
);

-- Commercial context per line, used to size a case the monitor opens by itself.
-- In the MVP these are synthetic stand-ins for the quote-to-issue data
-- connector described in milestone 3 of the pilot backlog.
CREATE TABLE IF NOT EXISTS product_line_commercials (
    product_line_id TEXT PRIMARY KEY REFERENCES product_lines(id) ON DELETE CASCADE,
    product TEXT NOT NULL,
    channel TEXT NOT NULL,
    segment TEXT NOT NULL,
    share_change_pp REAL NOT NULL,
    annual_premium_at_risk_m REAL NOT NULL,
    annual_quoted_premium_m REAL NOT NULL,
    conversion_baseline_pct REAL NOT NULL,
    conversion_current_pct REAL NOT NULL
);

-- Time series of our competitive standing, one row per monitor run per line.
CREATE TABLE IF NOT EXISTS position_snapshots (
    id TEXT PRIMARY KEY,
    product_line_id TEXT NOT NULL REFERENCES product_lines(id) ON DELETE CASCADE,
    captured_at TEXT NOT NULL,
    position_score REAL NOT NULL,
    our_rank INTEGER NOT NULL,
    provider_count INTEGER NOT NULL,
    leader_competitor_id TEXT NOT NULL,
    detail_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS monitor_thresholds (
    id TEXT PRIMARY KEY,
    product_line_id TEXT REFERENCES product_lines(id) ON DELETE CASCADE,
    metric TEXT NOT NULL,
    threshold REAL NOT NULL,
    severity TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS monitor_runs (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    trigger TEXT NOT NULL,
    breached INTEGER NOT NULL,
    result_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_feature_definitions_line
    ON feature_definitions(product_line_id, display_order);
CREATE INDEX IF NOT EXISTS idx_feature_observations_current
    ON feature_observations(feature_id, competitor_id, superseded_at);
CREATE INDEX IF NOT EXISTS idx_feature_observations_observed
    ON feature_observations(observed_at DESC);
CREATE INDEX IF NOT EXISTS idx_position_snapshots_line
    ON position_snapshots(product_line_id, captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_monitor_runs_started
    ON monitor_runs(started_at DESC);
