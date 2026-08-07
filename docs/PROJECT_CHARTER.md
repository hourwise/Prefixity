# Prefixity Project Charter

## Purpose

Prefixity is a provider-neutral context-efficiency profiler and — only if
justified by evidence — an optional context compiler for LLM/agent workloads.

It does **not** aim to maximise cache-hit percentage. It aims to make the
following question *testable*:

> For an observed LLM/agent workload, can a deterministic tool explain where
> context cost is being incurred, identify prefix divergence and unnecessary
> context, model provider-specific cache economics, and simulate alternative
> context policies before modifying live prompts?

A useful result may be "do nothing; your client is already close to optimal".

## Phase 0 scope (this repository)

Phase 0 is an **offline research/analysis harness**. It answers:

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

Phase 0 must work entirely from recorded/synthetic trace files. No network
access is required.

## Out of scope for Phase 0 (explicit non-goals)

- Publishing crates/packages or release artifacts.
- Secrets, paid API calls, telemetry.
- Background daemon, localhost LLM proxy, GUI/dashboard, authentication.
- SQLite (unless an unavoidable requirement is demonstrated).
- Semantic response caching, KV-cache storage, repository indexing/RAG.
- Automated context mutation of live requests.
- Automatic compression (a reserved policy/interface only).
- Live provider calls (deferred to a later Phase 0B live harness).

Phase 0A.1 adds an explicit conceptual separation: the **prefixity score**
(experimental heuristic), **observed prefix reuse** (trace-to-trace
comparison), and **provider-reported cache reuse** (normalized provider
usage) are distinct concepts and are never conflated in output. A single
trace can never prove reuse.

## Source-of-truth principles

1. Prefixity's cache/index state will never be authoritative.
2. Original source/provider state wins over derived Prefixity state.
3. Future Prefixity storage must be disposable and rebuildable.
4. Optimisation must eventually be fail-open: if Prefixity fails, the
   original request should remain usable.
5. Observation precedes transformation.
6. Simulation precedes automatic optimisation.
7. Provider-reported cache usage outranks Prefixity's theoretical estimate
   when determining what actually happened.
8. A lower token count is not automatically a better result if correctness
   degrades.

## Core concept

`prefixity(block)` is an **explainable estimate** of how suitable a context
block is for inclusion in a stable reusable request prefix. It is
experimental, deterministic, **not** a probability, and **not** produced by
machine learning. It may initially use observed change frequency, source
type, position, lifetime, hash stability, dependency changes, expected reuse
and token weight — all explainable.

See `docs/phase-0/PHASE_0_PLAN.md` and `docs/phase-0/SUCCESS_CRITERIA.md`.
