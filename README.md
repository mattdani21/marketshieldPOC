# MarketShield AI — Rust MVP

MarketShield is a sponsor-ready starting point for an agentic market-share defence platform for a South African life and investments company.

The application converts a competitive signal into a governed workflow:

`observe → enrich → diagnose → simulate → govern → approve → measure`

This repository is an **MVP foundation**, not a production underwriting or pricing system.

## What is implemented

- Rust HTTP service using Axum and Tokio.
- SQLite persistence using SQLx.
- Seeded synthetic competitor signals and one market-defence case.
- Persisted agent-analysis runs.
- Evidence admission and exclusion ledger.
- Deterministic scenario engine for four product responses.
- Commercial, actuarial and customer-outcome scorecard.
- Governance checks and human approval workflow.
- Audit-event API.
- Standalone browser UI served by the Rust process.
- Docker build and GitHub Actions CI definition.

## What is deliberately mocked

- Competitor web collection and document extraction.
- Insurer data connectors.
- LLM calls.
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

- Rust 1.97 or newer.
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
- `GET /api/cases`
- `GET /api/cases/MS-2026-017`
- `POST /api/cases/MS-2026-017/analyse`
- `POST /api/scenarios/evaluate`
- `POST /api/decisions`
- `POST /api/cases/MS-2026-017/approvals/advance`
- `GET /api/audit?case_id=MS-2026-017`

See [`docs/API.md`](docs/API.md) for payloads.

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
  app.rs                 Router, static UI and middleware
  config.rs              Environment configuration
  db.rs                  SQLite connection and schema setup
  error.rs               API error mapping
  models.rs              Domain and transport types
  repository.rs          Persistence and seeded demo data
  routes/                 HTTP handlers
  services/
    orchestrator.rs       Agent workflow
    scenario_engine.rs    Deterministic response model
    governance.rs         Control augmentation
static/
  index.html              API-backed sponsor interface
migrations/
  0001_init.sql           Initial schema
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

## Repository status

This repository begins as a proof of concept. The current environment used to prepare the initial commit did not include a Rust toolchain, so GitHub Actions is the first authoritative compile, lint, and test check. See [`docs/PROJECT_STATUS.md`](docs/PROJECT_STATUS.md).
