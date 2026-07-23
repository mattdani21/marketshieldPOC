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

## Scenarios

`POST /scenarios/evaluate`

Example:

```json
{
  "case_id": "MS-2026-017",
  "kind": "rapid_underwriting",
  "overrides": {}
}
```

Supported kinds:

- `rapid_underwriting`
- `targeted_premium_reduction`
- `benefit_rewards_bundle`
- `observe_only`

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
