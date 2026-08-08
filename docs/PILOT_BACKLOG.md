# Pilot backlog

## Milestone 1 — Runnable sponsor MVP

- [x] Rust service skeleton.
- [x] SQLite schema and seeded case.
- [x] API-backed sponsor UI.
- [x] Agent workflow endpoint.
- [x] Scenario engine.
- [x] Governance checks.
- [x] Human approval workflow.
- [x] Audit events.
- [x] Compile, lint, and test in CI.
- [x] Generate and commit `Cargo.lock`.
- [ ] Deploy to a controlled development environment.

## Milestone 1b — Competitive comparison and monitoring

- [x] Competitor, product-line and feature model.
- [x] Append-only observations preserving superseded values.
- [x] Weighted comparison engine with per-dimension gaps and standings.
- [x] Point-in-time comparison for historical position.
- [x] Position time series per product line.
- [x] Threshold-based drift detection.
- [x] Automatic signal creation and case opening on material drift.
- [x] Scheduled competitive-position check that fails on a breach.
- [x] Retirement annuity product line and response templates.
- [x] End-to-end API tests and a pinned competitive regression baseline.
- [ ] Per-line and per-dimension threshold tuning with product owners.
- [ ] Alerting to a channel product owners actually watch.

## Milestone 2 — Real evidence ingestion

- [x] Controlled feature schema with source classification and reference.
- [x] Detect and preserve changed competitor values as recoverable moves.
- [ ] Register approved public competitor sources.
- [ ] Download and hash product documents.
- [ ] Extract features from documents into the controlled schema.
- [ ] Store source date, URL and licence alongside the classification.
- [ ] Detect duplicate documents.
- [ ] Add human verification for material facts.

## Milestone 3 — Insurer analytical data

- [ ] Replace the seeded `product_line_commercials` stand-in with real data.
- [ ] Define the minimum quote-to-issue dataset.
- [ ] Use aggregated or pseudonymised fields by default.
- [ ] Map product, channel, adviser group, and segment dimensions.
- [ ] Add data-quality and reconciliation reports.
- [ ] Establish purpose, access, and retention controls.

## Milestone 4 — Actuarial tool adapter

- [ ] Define a versioned scenario input contract.
- [ ] Define output measures and units.
- [ ] Connect an approved pricing or projection engine.
- [ ] Record basis, model version, and run identifier.
- [ ] Compare results to approved test cases.
- [ ] Add timeout, retry, and exception handling.

## Milestone 5 — Enterprise control plane

- [ ] Enterprise identity and role mapping.
- [ ] Segregation of duties.
- [ ] Model and prompt registry.
- [ ] Policy-as-code controls.
- [ ] Tamper-evident audit export.
- [ ] Incident response and rollback.
- [ ] Outsourcing, security, and architecture approval.

## Sponsor pilot success measures

- The monitor detects a real competitive move before it is raised by a human.
- One diagnosis accepted by product and distribution owners.
- One scenario validated using an approved actuarial tool.
- At least 50% reduction in committee-pack preparation time.
- No unapproved source or personal-information use.
- Compliance, risk, data, security, and actuarial acceptance.
- Users voluntarily open a second market-defence case.
