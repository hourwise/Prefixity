# Phase 0 plan

## Goal

Build an **offline research/analysis harness** that can answer, deterministically
and from trace files only:

- What context blocks were sent?
- Which blocks changed between consecutive requests?
- Where did the observed reusable prefix first diverge (trace-to-trace)?
- How much context theoretically remained reusable?
- What cache usage did the provider report, if supplied in the trace?
- What portion was cache-read, cache-written and fresh?
- Which blocks account for most fresh input?
- What would hypothetical alternative policies have done?
- Would those alternatives reduce estimated cost?
- Could an apparent optimisation actually increase estimated cost?

Phase 0 exists to confirm or invalidate the core thesis *before* significant
engineering effort is spent.

## Deliverables

1. A small Cargo workspace:
   - `prefixity-core` — deterministic analysis logic;
   - `prefixity-cli` — thin CLI over that logic.
2. A versioned trace format (`docs/phase-0/TRACE_FORMAT.md`).
3. Analysis functions: validation, ordered comparison, first-prefix-divergence,
   reusable-prefix estimation (observed, trace-to-trace), changed/unchanged accounting, fresh-context
   accounting, provider-usage reconciliation, cost calculation, and
   human-readable explanations of lost reuse.
4. An offline policy simulator with five policies (`baseline`,
   `stable-prefix`, `defer-volatile`, `prune-stale-tool-output`, `combined`)
   and a reserved `compression` name. Policies never mutate their input.
5. A CLI (`prefixity`) with `validate`, `analyse`, `compare`, `simulate`
   and `--json` / `--provider-profile` options.
6. Eight synthetic fixture scenarios under `fixtures/traces/`.
7. SYNTHETIC provider profiles under `provider-profiles/`.
8. Tests: unit + integration, including determinism and non-mutation proofs.
9. Documentation: charter, threat model, prior art, plan, trace format,
   experiments, success criteria.

## The "prefixity" score

`prefixity(block)` is an explainable estimate of how suitable a block is for
a stable reusable prefix. Phase 0 uses a **conservative initial heuristic**
(source-type baseline + optional/stale penalties + lifetime adjustment) with
documented constants. It is provisional, not a probability, not ML. Every
score carries reasons and numeric signals.

## What is deliberately NOT built

- Live provider calls (deferred to a Phase 0B live harness).
- Automatic compression (reserved interface only).
- Daemon/proxy/GUI/auth/telemetry/storage/SQLite/RAG/semantic caching.
- Hard-coded "current" provider pricing.

## Build/verify steps

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

All must pass before Phase 0 is considered complete.
