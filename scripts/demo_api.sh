#!/usr/bin/env sh
set -eu
BASE_URL="${BASE_URL:-http://127.0.0.1:8080/api}"
CASE_ID="MS-2026-017"

echo "== Health =="
curl -fsS "$BASE_URL/health"
printf '\n\n== Dashboard ==\n'
curl -fsS "$BASE_URL/dashboard"
printf '\n\n== Analyse case ==\n'
curl -fsS -X POST "$BASE_URL/cases/$CASE_ID/analyse"
printf '\n\n== Evaluate recommended scenario ==\n'
curl -fsS -X POST "$BASE_URL/scenarios/evaluate" \
  -H 'Content-Type: application/json' \
  -d "{\"case_id\":\"$CASE_ID\",\"kind\":\"rapid_underwriting\",\"overrides\":{}}"
printf '\n'
