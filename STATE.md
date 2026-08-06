# State

## Current state

Rust 2024 MVP (axum + tokio + sqlx/SQLite, MSRV 1.88) with a complete notice + respond loop: competitor/product-line/feature model, append-only observations, weighted comparison engine, point-in-time standings, position time series, threshold drift detection that opens a case, deterministic scenario engine, evidence admission/exclusion ledger, governance checks, human approval workflow, audit events, persisted agent-analysis runs, and a browser UI served by the Rust process. Docker (`Dockerfile`, `compose.yaml`), CI (`ci.yml` + scheduled `competitive-watch.yml`), committed `Cargo.lock`, end-to-end API tests, a seeded 4-provider retirement-annuity demonstration, and a full doc set (`docs/ROADMAP.md`, `PILOT_BACKLOG.md`, `PROJECT_STATUS.md`, `GOVERNANCE.md`, `ARCHITECTURE.md`, `API.md`, `BUSINESS_CASE.md`, `OPERATING_MODEL.md`, `WHY_RUST.md`, `SPONSOR_DEMO_TALK_TRACK.md`) are committed. Default branch is `main`. `docs/PROJECT_STATUS.md` (dated 23 July 2026) places the stage at "sponsor MVP and repository foundation".

## Broken / incomplete

- Open issue #1: "fix Rust formatting and expose compile checks" — formatting/clippy gates are not yet surfaced in CI.
- `docs/PROJECT_STATUS.md`: the publication environment had no `rustc`/`cargo`/Docker, so GitHub Actions is the first authoritative build check — the first CI run still needs inspection.
- `docs/PILOT_BACKLOG.md` open items: API smoke + migration tests not added; not deployed to a controlled development environment; per-line/per-dimension threshold tuning and alerting to a watched channel pending.
- Known boundaries (documented): competitor evidence and insurer performance data are synthetic; scenario values are not approved actuarial outputs; SQLite audit storage is not tamper-evident; no enterprise identity, secrets or observability.
- Deliberately mocked (README): web collection, insurer data connectors, LLM calls, validated actuarial models, IAM, regulatory rule interpretation.

## Last known blockers

Issue #1 open (Rust formatting / compile-check exposure). No other open issues. First CI run unverified.

## Test command

`cargo test` (quality gates: `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`)

## Run command

`docker compose up --build` (http://localhost:8080), or locally: `mkdir -p data && cargo run` (default `http://127.0.0.1:8080`; optional `MARKETSHIELD_BIND`, `MARKETSHIELD_DATABASE_URL`, `RUST_LOG`)
