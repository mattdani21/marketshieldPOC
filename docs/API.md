# API reference

Base path: `/api`

## Health

`GET /health`

Returns service name, version, and health status.

## Dashboard

`GET /dashboard`

Returns synthetic market-share, exposure, conversion, case, and signal measures.

`GET /signals`

Returns the signal inbox.

## Cases

`GET /cases`

Returns market-defence cases.

`GET /cases/{case_id}`

Returns the case, hypotheses, evidence ledger, governance checks, and approval workflow.

`POST /cases/{case_id}/analyse`

Runs and persists the agent workflow.

## Competitive comparison

`GET /product-lines`

Returns the tracked product lines.

`GET /competitors`

Returns the tracked providers. Exactly one carries `is_us: 1` and represents our
own offering; it is held in the same table so every comparison is computed by
one set of rules.

`GET /comparison/{product_line_id}`

The core comparison. For each tracked dimension it returns every provider's
current value with its source, which provider holds the best value, and our gap
to it. It also returns:

- `standings` — every provider's weighted position score, ranked;
- `our_position` — our score, rank, distance behind the leader, and
  `largest_gaps`, the weighted deficits that explain where we are losing;
- `recent_moves` — recorded competitor value changes with their sources.

Scoring: each dimension is normalised across the observed providers, 100 at the
best value and 0 at the worst, then combined using the dimension weights. A
score therefore means "where we sit among these providers on this dimension",
not an absolute rating.

`GET /comparison/{product_line_id}/history`

Returns the recorded position snapshots for the line, oldest first.

`POST /observations`

Records a new competitor value. The previous value is superseded rather than
overwritten, so the change becomes a recoverable competitor move.

```json
{
  "competitor_id": "ridgeline",
  "feature_id": "ra_eac_500k",
  "value": 0.89,
  "source_classification": "public",
  "source_reference": "Public EAC disclosure, July 2026"
}
```

`source_classification` must be `public`, `approved_internal`, `consented` or
`licensed`, and `source_reference` must be non-empty. Competitively sensitive
values are refused.

## Monitoring

`POST /monitor/run`

Recomputes every product line's position, records a snapshot, and compares it to
the previously recorded one. Optional body:

```json
{
  "trigger": "manual",
  "raise_signals": true
}
```

With `raise_signals` false the run reports drift without creating signals or
opening cases. The response reports per-line scores and rank changes, the
findings raised, any signals created, any cases opened, and `breached` — true
when a finding reached the severity that requires human attention.

The same check runs as a binary for scheduled use:

```bash
cargo run --bin monitor -- --report competitive-position.md --trigger scheduled
```

It writes a Markdown report and exits non-zero when a threshold is breached.

## Scenarios

`POST /scenarios/evaluate`

Example:

```json
{
  "case_id": "MS-2026-018",
  "kind": "ra_fee_restructure",
  "overrides": {}
}
```

Supported kinds, by product line:

Retirement annuity:

- `ra_fee_restructure`
- `ra_transfer_turnaround`
- `ra_fund_range_expansion`

Individual life risk:

- `rapid_underwriting`
- `targeted_premium_reduction`
- `benefit_rewards_bundle`

Any line:

- `observe_only`

Evaluating a template against a case from another product line is refused.

Optional override fields:

- `conversion_gain_pp`
- `implementation_uptake_pct`
- `new_business_margin_pct`
- `claims_index`

The response contains metrics, decision scores, stress results, recommendation status, and scenario provenance.

## Decisions and approvals

`POST /decisions`

```json
{
  "case_id": "MS-2026-017",
  "scenario_id": "<scenario UUID>",
  "decision": "submit_for_review",
  "notes": "Sponsor demo submission"
}
```

`POST /cases/{case_id}/approvals/advance`

```json
{
  "decision": "approve",
  "notes": "Approved in controlled demo"
}
```

## Audit

`GET /audit?case_id=MS-2026-017`

Returns the most recent persisted activity events.

## Error format

```json
{
  "code": "bad_request",
  "message": "bad request: explanation"
}
```
