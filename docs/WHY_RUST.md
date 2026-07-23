# Why Rust is used

Rust is used for the service and performance-sensitive orchestration layer because it provides predictable performance, concurrency, memory safety, and deployment control.

## Good Rust use cases

- concurrent document downloads and change detection;
- high-volume parsing and normalisation;
- typed API orchestration;
- deterministic scenario and rules execution;
- streaming event and audit processing;
- controlled insurer-system connectors;
- low-memory container services.

## Where Rust is not automatically the answer

- model inference dominated by network or accelerator latency;
- exploratory actuarial and data-science notebooks;
- approved models already operating safely in another language;
- one-off analysis where iteration speed matters more than runtime speed.

Recommended architecture:

- Rust for the core API, orchestration, parsers, controls, and high-throughput services;
- approved actuarial engines behind versioned adapters;
- Python or notebooks for controlled research where appropriate;
- model providers behind replaceable policy-controlled interfaces.

Rust improves service safety and throughput. It does not remove the need for data-quality controls, model validation, observability, load testing, and sound architecture.
