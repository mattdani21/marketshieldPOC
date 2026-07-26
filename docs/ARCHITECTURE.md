# Architecture

## Current MVP

```text
Browser UI          Scheduled job (monitor binary)
   |                        |
Axum HTTP API               |
   |                        |
Application services <------+
   |-- comparison engine (pure, point-in-time capable)
   |-- competitive monitor (drift detection, case opening)
   |-- market-defence orchestrator
   |-- deterministic scenario engine
   |-- governance augmentation
   |
Repository layer
   |
SQLite
```

The Rust service also serves the static sponsor interface. The monitor binary
shares the library, so a scheduled run and an API-triggered run take the same
path through the same code.

## Domain flow

The product has two entry points into the same governed workflow. The upper path
is what stops a competitive move being noticed late.

```text
Competitor observation (superseded, never overwritten)
  -> Comparison and position scoring
  -> Position snapshot
  -> Drift detection against thresholds
  -> Signal, and on material drift an opened case
        |
        v
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
- `services/comparison`: provider scoring and gap analysis. A pure function over
  observations, so the API, the monitor and the regression suite all score
  identically and it can be tested without a database.
- `services/monitor`: recomputes position, compares to the last recorded
  snapshot, raises signals, and opens a case on material drift.
- `services`: orchestration, scenario, and governance logic.
- `repository`: SQLite queries and persistence.
- `seed`: synthetic competitive landscape and its recorded history.
- `models`: typed transport and domain structures.
- `static`: API-backed demonstration UI.

## Why observations are append-only

A competitor's current fee is worth less than the fact that they changed it, when
they changed it, and what it was before. Superseding rather than overwriting
means competitor moves are recoverable with their sources, historical positions
can be recomputed by the same rules that produce today's, and the monitor can
tell a change it has already reported from one it has not.

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
4. **Action boundary:** material changes require approved deterministic systems and human authority. The monitor's autonomy stops at recording a position, raising a signal, and opening a case with every approval gate unmet.
5. **Audit boundary:** every material source, tool run, control, and decision must be traceable. Every competitor value carries a source reference, and every monitor run and monitor-opened case is written to the audit log with the position that triggered it.

## Scale and performance

Rust is intended to support concurrent ingestion, parsing, typed orchestration, and predictable low-memory services. SQLite is suitable only for the POC; production should select an institution-approved transactional and analytical storage architecture.
