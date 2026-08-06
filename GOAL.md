# Goal

Build MarketShield as a market-share defence platform ready for any regulated life-insurance market: notice (competitor intel) + respond (governed workflow)

## Roadmap

### M1 — Sponsor MVP foundation: green build, green CI

- [ ] Resolve open issue #1 (Rust formatting + expose compile checks) so `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` pass in CI on every PR
- [ ] Confirm the first authoritative CI run (`ci.yml`) is green — the publication environment had no Rust toolchain, so GitHub Actions is the first real build check
- [ ] Add API smoke and migration tests (open backlog item)
- [ ] Deploy the Docker image (`Dockerfile`, `compose.yaml`) to a controlled development environment (open backlog item)
- [ ] Keep the scheduled `competitive-watch.yml` job healthy and its non-zero-exit-on-breach semantics verified

*Definition of done:* main is always green in CI, issue #1 is closed, and the MVP runs from a Docker image in a controlled environment.

### M2 — Twelve-week controlled pilot (ROADMAP Phase 1)

- [ ] Select one product and channel with a measurable problem; define market-share, conversion, value and customer-outcome measures (Gate 1)
- [ ] Register approved public competitor sources and extract features into the controlled schema (Gate 2: owners accept at least one useful diagnosis)
- [ ] Connect an approved actuarial pricing/projection engine through the versioned scenario contract (Gate 3: at least one economically credible response)
- [ ] Produce a committee-ready recommendation, measure pack-preparation time, and run a restricted shadow-mode workflow
- [ ] Meet pilot success criteria: one accepted diagnosis, one validated scenario, ≥50% pack-prep time reduction, no unapproved source or personal-information use, control-function acceptance, voluntary second case

*Definition of done:* the Phase 1 exit criteria in `docs/ROADMAP.md` are met and a production investment decision is made.

### M3 — Real evidence ingestion (PILOT_BACKLOG M2)

- [ ] Register approved public competitor sources
- [ ] Download and hash product documents; detect duplicates
- [ ] Extract features into the controlled schema with source date, URL and licence recorded
- [ ] Add human verification for material facts

*Definition of done:* competitor moves enter the platform from real, licensed, hashed sources without manual data entry.

### M4 — Insurer analytical data (PILOT_BACKLOG M3)

- [ ] Replace the seeded `product_line_commercials` stand-in with real aggregated quote-to-issue data
- [ ] Define the minimum quote-to-issue dataset; map product, channel, adviser-group and segment dimensions
- [ ] Add data-quality and reconciliation reports; establish purpose, access and retention controls

*Definition of done:* commercial sizing comes from approved aggregated data with quality checks, not seeds.

### M5 — Actuarial tool adapter (PILOT_BACKLOG M4)

- [ ] Define the versioned scenario input contract and the output measures/units
- [ ] Connect an approved pricing or projection engine; record basis, model version and run ID
- [ ] Compare results to approved test cases; add timeout, retry and exception handling

*Definition of done:* every scenario run is attributable to an approved tool version and passes approved test cases.

### M6 — Enterprise control plane (PILOT_BACKLOG M5, ROADMAP Phase 2)

- [ ] Enterprise identity and role mapping; segregation of duties
- [ ] Model and prompt registry; policy-as-code controls
- [ ] Tamper-evident audit export; incident response, rollback, monitoring, resilience and vendor exit

*Definition of done:* the platform satisfies enterprise security, audit and outsourcing requirements per ROADMAP Phase 2.

### M7 — Multi-product expansion + operating model (ROADMAP Phases 3–4)

- [ ] Add life-risk, savings, retirement and investment use cases with product-specific schemas and actuarial adapters
- [ ] Establish permanent product and control owners, service levels and support
- [ ] Measure realised value and decide: scale, partner, productise or stop

*Definition of done:* the Phase 3–4 scope in `docs/ROADMAP.md` is delivered or explicitly descoped with reasons.

## State

See STATE.md for the current state, known gaps and exact test/run commands.
