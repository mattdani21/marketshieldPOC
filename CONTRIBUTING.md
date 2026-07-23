# Contributing

## Development approach

Keep changes small, testable, and traceable to a business or control requirement.

Before opening a pull request:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Branch and commit style

- Branches: `feature/<description>`, `fix/<description>`, or `docs/<description>`.
- Commits: imperative and focused, for example `add evidence source registry`.
- Pull requests must explain the business value, data impact, model impact, control impact, and validation performed.

## Architecture rules

- Keep HTTP handlers thin.
- Put business logic in services.
- Keep database access in the repository layer.
- Use typed request, result, and error contracts.
- Do not place secrets or customer data in source control.
- Do not call an LLM for a calculation that belongs in a deterministic tool.
- Do not add an autonomous material action without an explicit approval gate.
- Record provenance for evidence, model outputs, and decisions.

## Demo and production boundaries

Synthetic demo assumptions must remain visibly marked.

A change that connects real insurer information or an external model must include:

- data classification;
- lawful-purpose and access assessment;
- threat and privacy review;
- model or tool validation plan;
- logging and incident requirements;
- rollback and exit approach.
