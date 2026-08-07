# Phase 0 success / failure criteria

## Success criteria

Phase 0 is successful if the harness:

- deterministically identifies prefix divergence;
- distinguishes **theoretical reuse** from **provider-reported reuse**;
- represents provider economics as **configurable data** (not hard-coded);
- demonstrates cases where optimisation **helps**;
- demonstrates cases where optimisation **does not help**;
- demonstrates cases where an apparent optimisation makes cost **worse**;
- proves by tests that **required blocks cannot be silently removed**.

Phase 0 does **not** need to prove Prefixity is commercially useful. It
exists to make that question testable.

### Mapping to fixtures/tests

| Criterion | Evidence |
| --- | --- |
| Deterministic divergence | `compare` on fixtures 01/02/03; determinism tests. |
| Theoretical vs reported reuse | `analyse` reconciliation; fixture 04 (theoretical 9,500 vs reported 15,500 kept distinct). |
| Economics as data | `CostProfile` files; fixture 05 (worthwhile under one profile, not under another). |
| Optimisation helps | `simulate` on fixture 06 (defer-volatile saves 20,000 tokens). |
| Optimisation does not help | fixture 07 (`already-optimal`: all policies produce zero change; no false recommendation). |
| Optimisation makes cost worse | fixture 05 with `synthetic-cache-write-expensive` (caching is a net loss). |
| Required blocks never removed | fixture 08 + policy tests (required block retained by every policy). |

## Phase 0 stop / pivot conditions

Later development should stop or pivot if live testing shows:

- context organisation produces negligible benefit;
- native clients/providers already expose all useful analysis;
- simulations do not predict real provider behaviour;
- optimisation repeatedly reduces correctness;
- provider-specific differences are too small to justify abstraction;
- overhead exceeds meaningful savings.

## Quality requirements

- Idiomatic stable Rust; simple dependencies; no `unsafe`.
- No speculative abstractions; no TODO placeholders for required behaviour.
- Public structures/functions documented.
- Structured error types; no panics on malformed user trace input.
- Bounded/safe input handling.
- README states Prefixity is experimental research software and Phase 0 does
  not modify live LLM requests.
