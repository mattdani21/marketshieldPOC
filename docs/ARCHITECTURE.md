# Architecture

## Current MVP

```text
Browser UI
   |
Axum HTTP API
   |
Application services
   |-- market-defence orchestrator
   |-- deterministic scenario engine
   |-- governance augmentation
   |
Repository layer
   |
SQLite
```

The Rust service also serves the static sponsor interface.

## Domain flow

```text
Signal
  -> Evidence admission
  -> Root-cause hypotheses
  -> Scenario evaluation
  -> Governance checks
  -> Human approval
  -> Audit and outcome measurement
```

## Module responsibilities

- `routes`: HTTP transport and validation boundary.
- `services`: orchestration, scenario, and governance logic.
- `repository`: SQLite queries, persistence, and synthetic seed data.
- `models`: typed transport and domain structures.
- `static`: API-backed demonstration UI.

## Production target

A production design should separate:

- source and document ingestion;
- analytical data products;
- evidence and feature storage;
- agent orchestration;
- actuarial tool adapters;
- policy and approval services;
- tamper-evident audit storage;
- monitoring and outcome measurement.

## Trust boundaries

1. **Public evidence boundary:** only registered and permitted competitor sources.
2. **Insurer data boundary:** approved, minimised, access-controlled aggregates.
3. **Model boundary:** language models may extract and synthesise, not perform authoritative actuarial calculations.
4. **Action boundary:** material changes require approved deterministic systems and human authority.
5. **Audit boundary:** every material source, tool run, control, and decision must be traceable.

## Scale and performance

Rust is intended to support concurrent ingestion, parsing, typed orchestration, and predictable low-memory services. SQLite is suitable only for the POC; production should select an institution-approved transactional and analytical storage architecture.
