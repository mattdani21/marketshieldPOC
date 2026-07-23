# MarketShield delivery roadmap

## Product objective

Help a product actuary answer:

1. Where is profitable market share being lost?
2. What is driving the loss?
3. Which response is economically viable?
4. Which customer, privacy, competition, model, and operational controls apply?
5. Did the approved response produce the expected outcome?

## Principles

- Start with one product, one channel, and one measurable problem.
- Use public, licensed, or authorised evidence only.
- Keep actuarial calculations in approved deterministic tools.
- Keep material decisions with accountable humans.
- Prefer aggregated or pseudonymised insurer data.
- Record sources, assumptions, models, approvals, and outcomes.

# Phase 0 — Sponsor MVP

**Status:** current repository foundation.

Delivered:

- Rust API and orchestration service;
- SQLite persistence and synthetic case;
- signal, evidence, diagnosis, scenario, governance, approval, and audit workflows;
- API-backed sponsor UI;
- Docker and CI definitions.

Exit condition: the project compiles and tests in CI and the sponsor confirms a narrow pilot use case.

# Phase 1 — Twelve-week controlled pilot

## Weeks 1–2: Frame and govern

- Select one product and distribution channel.
- Define market-share, conversion, value, and customer-outcome measures.
- Baseline the current decision cycle.
- Approve the minimum data set and lawful purpose.
- Confirm competition-information boundaries.
- Complete initial privacy, security, outsourcing, and model-risk assessments.

**Gate 1:** proceed only with a measurable problem, accountable owners, permitted data, and stop conditions.

## Weeks 3–5: Connect and diagnose

- Register approved public competitor sources.
- Download, hash, classify, and version documents.
- Extract product features into a controlled schema.
- Load approved aggregated quote-to-issue or retention information.
- Generate and human-review root-cause hypotheses.

**Gate 2:** proceed only if business owners accept at least one useful diagnosis.

## Weeks 6–9: Simulate and challenge

- Define a versioned actuarial scenario contract.
- Connect an approved pricing, projection, capital, or profitability engine.
- Record model version, basis, run ID, and exceptions.
- Compare operational, pricing, product, retention, and distribution responses.
- Run sensitivity, fairness, claims, lapse, capital, and delivery-risk tests.

**Gate 3:** proceed only if at least one response is economically credible, explainable, and controllable.

## Weeks 10–12: Committee proof

- Produce a committee-ready recommendation.
- Measure preparation-time reduction.
- Demonstrate source, model, approval, and audit traceability.
- Run a restricted shadow-mode workflow.
- Produce the production investment decision.

Pilot success criteria:

- one accepted diagnosis;
- one actuarially validated scenario;
- at least 50% reduction in pack-preparation time;
- no unapproved source or personal-information use;
- control-function acceptance;
- voluntary use on a second case.

# Phase 2 — Months 4–6: Production control plane

- Enterprise identity and role mapping.
- Segregation of duties.
- Data catalogue, lineage, retention, and access enforcement.
- Model and prompt registry.
- Policy-as-code controls.
- Tamper-evident audit export.
- Monitoring, incident response, rollback, resilience, and vendor exit.

# Phase 3 — Months 7–9: Multi-product expansion

- Add life-risk, savings, retirement, and investment use cases.
- Add product-specific schemas and actuarial adapters.
- Add adviser, service-friction, retention, and experiment analytics.
- Improve verified extraction and change detection.

# Phase 4 — Months 10–12: Enterprise operating model

- Establish permanent product and control owners.
- Define service levels and support.
- Integrate committee and portfolio workflows.
- Measure realised value.
- Decide whether to scale, partner, productise, or stop.

# Engineering priority order

1. Build integrity and reproducible CI.
2. Evidence-source registry and document ingestion.
3. Aggregated insurer analytical data.
4. Approved actuarial tool adapter.
5. Enterprise authentication, controls, observability, and audit.

# Pilot non-goals

The pilot will not autonomously change products or prices, underwrite customers, issue advice, target individuals for consequential treatment, launch campaigns, or write to production insurer systems.
