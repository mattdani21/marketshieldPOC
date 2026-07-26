# MarketShield AI — Rust MVP

MarketShield is a sponsor-ready starting point for an agentic market-share defence platform for a South African life and investments company.

It exists to answer one question — *"a competitor is taking share in our retirement annuity business and we do not know why"* — and then to stop that question being asked cold a second time.

It does that in two halves:

- **Notice.** Track what every provider offers on the dimensions the product is actually chosen on, score our position against them, and watch that position on a schedule. When it moves against us, raise a signal and open a case without waiting to be asked.
- **Respond.** Convert that case into a governed workflow: `observe → enrich → diagnose → simulate → govern → approve → measure`.

This repository is an **MVP foundation**, not a production underwriting or pricing system.

## What is implemented

Competitive intelligence:

- Competitor, product-line and feature model, with our own offering held alongside competitors so one set of rules produces every comparison.
- Append-only feature observations: a new value supersedes rather than overwrites the old one, so competitor moves stay recoverable with their sources.
- Weighted comparison engine producing per-dimension gaps, provider standings, our rank, and the weighted deficits that explain where we are losing.
- Point-in-time comparison, so past standings are recomputed by the same rules rather than recorded separately.
- Position time series per product line.
- Scheduled monitor that recomputes position, detects drift against thresholds, raises signals, and opens a case when the drift is material.
- `monitor` binary that writes a Markdown competitive-position report and exits non-zero on a breach, for scheduled use in CI.

Governed response:

- Rust HTTP service using Axum and Tokio, with SQLite persistence using SQLx.
- Retirement annuity and individual life risk product lines, each with its own response templates. A template cannot be evaluated against another line's case.
- Deterministic scenario engine sized from the case rather than a global constant.
- Commercial, actuarial and customer-outcome scorecard.
- Evidence admission and exclusion ledger, governance checks and human approval workflow.
- Persisted agent-analysis runs and an audit-event API.
- Browser UI served by the Rust process, showing the comparison matrix, position trend, deficits and competitor moves.
- Docker build, code CI, and a scheduled competitive-watch workflow.

## The seeded demonstration

A fresh database seeds four retirement annuity providers across eight tracked dimensions, with a recorded history in which the challenger repriced. On first start the monitor runs, finds that our position fell, and opens a retirement annuity case by itself — carrying its diagnosis, evidence and governance checks.

Cape Meridian ranks 3 of 4, roughly 58 points behind the leader. The largest weighted deficits are effective annual cost at R500k and R2m, then Section 14 transfer turnaround. We still lead on fund range and adviser fee options. Every number is synthetic.

## What is deliberately mocked

- Competitor web collection and document extraction. Competitor values are entered through `POST /api/observations` or seeded; nothing crawls a source.
- Insurer data connectors. The commercial sizing a monitor-opened case carries comes from a seeded per-line table standing in for the quote-to-issue dataset.
- LLM calls. The agent workflow returns a fixed narrative; the diagnosis a monitor-opened case carries is computed from the comparison, not generated.
- Validated actuarial pricing, capital and claims models.
- Identity and access management.
- Enterprise audit immutability.
- Regulatory rule interpretation and sign-off.

The deterministic engine exists to prove workflow and sponsorship value. It must be replaced or wrapped by institution-approved actuarial tooling before any real decision use.

## Run with Docker

```bash
docker compose up --build
```

Open `http://localhost:8080`.

## Run with a local Rust toolchain

Requirements:

- Rust 1.85 or newer (the 2024 edition minimum).
- A C toolchain required by SQLite dependencies on some platforms.

```bash
mkdir -p data
cargo run
```

The default service address is `http://127.0.0.1:8080`.

Optional environment variables:

```bash
export MARKETSHIELD_BIND=0.0.0.0:8080
export MARKETSHIELD_DATABASE_URL=sqlite://data/marketshield.db
export RUST_LOG=marketshield=info,tower_http=info
```

## Test and quality commands

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## API walkthrough

```bash
./scripts/demo_api.sh
```

Core endpoints:

- `GET /api/health`
- `GET /api/dashboard`
- `GET /api/comparison/retirement_annuity`
- `GET /api/comparison/retirement_annuity/history`
- `GET /api/competitors`
- `POST /api/observations`
- `POST /api/monitor/run`
- `GET /api/cases`
- `GET /api/cases/MS-2026-017`
- `POST /api/cases/MS-2026-017/analyse`
- `POST /api/scenarios/evaluate`
- `POST /api/decisions`
- `POST /api/cases/MS-2026-017/approvals/advance`
- `GET /api/audit?case_id=MS-2026-017`

See [`docs/API.md`](docs/API.md) for payloads.

## Competitive position check

The same monitor the API exposes also runs as a binary, for scheduled use:

```bash
cargo run --bin monitor -- --report competitive-position.md --trigger scheduled
```

It prints and writes a Markdown report — position by product line, findings,
the full feature comparison, and recorded competitor moves — and **exits
non-zero when our position has moved against us by more than the configured
threshold**. Code CI answers "does it still build"; this answers "are we still
competitive".

Flags: `--report <path>`, `--trigger <name>`, `--no-signals` (report without
creating signals or cases), `--allow-breach` (report without failing).

`.github/workflows/competitive-watch.yml` runs it daily. Note that this
repository has no persistent database, so each CI run reseeds and re-detects the
same seeded decline: it demonstrates the mechanism rather than tracking real
drift. Pointed at a persistent database, each run compares against the position
recorded by the previous run and the exit code becomes meaningful.

## Sponsor and delivery documentation

- [`docs/BUSINESS_CASE.md`](docs/BUSINESS_CASE.md) — sponsorship rationale, value hypotheses, pilot ask, and stop conditions.
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — twelve-week pilot and twelve-month delivery roadmap.
- [`docs/GOVERNANCE.md`](docs/GOVERNANCE.md) — customer, privacy, competition, actuarial, outsourcing, and audit boundaries.
- [`docs/WHY_RUST.md`](docs/WHY_RUST.md) — where Rust creates value and where it does not.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — system components and trust boundaries.
- [`docs/SPONSOR_DEMO_TALK_TRACK.md`](docs/SPONSOR_DEMO_TALK_TRACK.md) — guided sponsorship demonstration.
- [`docs/PILOT_BACKLOG.md`](docs/PILOT_BACKLOG.md) — prioritised engineering backlog.

## Project structure

```text
src/
  lib.rs                 State construction and startup baseline run
  main.rs                Service binary
  bin/monitor.rs         Scheduled competitive-position check
  app.rs                 Router, static UI and middleware
  config.rs              Environment configuration
  db.rs                  SQLite connection and schema setup
  error.rs               API error mapping
  models.rs              Domain and transport types
  repository.rs          Persistence and seeded demo data
  seed.rs                Synthetic competitive landscape
  routes/                 HTTP handlers
  services/
    comparison.rs         Competitor comparison and scoring
    monitor.rs            Drift detection and case opening
    orchestrator.rs       Agent workflow
    scenario_engine.rs    Deterministic response model
    governance.rs         Control augmentation
tests/
  api_workflow.rs         End-to-end API tests
  competitive_regression.rs  Pinned competitive baseline
static/
  index.html              API-backed sponsor interface
migrations/
  0001_init.sql           Initial schema
  0002_competitive_intelligence.sql  Competitor and monitoring schema
docs/
  BUSINESS_CASE.md
  ROADMAP.md
  GOVERNANCE.md
  WHY_RUST.md
  ARCHITECTURE.md
  API.md
  PROJECT_STATUS.md
  SPONSOR_DEMO_TALK_TRACK.md
  PILOT_BACKLOG.md
```

## Important boundary

The MVP may recommend and document actions. It must not autonomously:

- change a premium or product rule;
- decline or modify customer cover;
- select customers for a consequential offer;
- issue regulated advice;
- launch a campaign;
- write to a production policy, underwriting or investment platform.

Those actions require approved deterministic systems and accountable human authority.

The monitor does act without being asked, and the boundary of that act is deliberate. It may record a position snapshot, raise a signal, and open a case for human diagnosis. Opening a case commits nothing: the case arrives with every approval gate unmet and a model-validation check already marked for review. It cannot approve, decide, price, or contact anyone. Every monitor run and every case it opens is written to the audit log with the position that triggered it.

Position scoring is a deterministic demonstration model. It ranks providers relative to each other on the values it has been given; it is not a validated measure of competitiveness and must not be used to justify a pricing or product decision on its own.

## Repository status

This proof of concept now compiles, lints and tests locally, and both are enforced in CI. Two pre-existing CI blockers were fixed along the way: `Cargo.toml` pinned `rust-version = "1.97"`, ahead of released stable, so `cargo build` refused to run at all; and `cargo fmt --check` failed on the original tree. `Cargo.lock` is now committed for reproducible builds. See [`docs/PROJECT_STATUS.md`](docs/PROJECT_STATUS.md).
