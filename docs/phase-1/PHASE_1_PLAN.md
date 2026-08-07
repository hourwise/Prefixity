# Phase 1 Plan — Real Workload Observation and Quality-Gated Context Decisions

## Status

**Design gate only. No Phase 1 runtime implementation is authorized by this document.**

Phase 0 is complete. Phase 1 begins only after this design set is reviewed and accepted.

## Goal

Determine whether Prefixity can make useful, provider-neutral context-management decisions on **realistic agent workloads** without sacrificing task quality.

The central Phase 1 question is:

> Given the actual context supplied to an agent, which intervention — if any — is justified: keep, defer, prune, relocate, compress, or do nothing?

Phase 1 is not a compression project and is not a cache-hit maximization project. It is a **context decision layer** research phase.

## What Phase 0 established

Phase 0 established enough to justify this next question:

- structural prefix reuse can be observed deterministically;
- provider-reported cache reuse must remain distinct from structural reuse potential;
- different tokenizer/accounting systems must not be compared by absolute token subtraction;
- real provider cache state may lag structural reuse potential;
- stable-prefix, early-divergence, late-divergence and persistence cases can be captured and reconciled;
- provider-specific usage can be normalized behind explicit versioned API-surface schemas;
- unsafe prompt mutation must remain gated behind evidence and quality checks.

Phase 1 therefore does **not** repeat the Phase 0B cache-validation matrix.

## Phase 1 research hypothesis

> A deterministic, provider-neutral observer can identify context-management opportunities in real agent trajectories and recommend an intervention that reduces avoidable context cost while preserving load-bearing context and task quality.

A valid result is:

> **DO NOTHING — the observed context is already appropriate or the quality risk of intervention is too high.**

## Phase 1A — Real-workload ingestion and observation

Question:

> Can Prefixity ingest representative agent trajectories and produce useful, reproducible explanations without modifying them?

Build only what is required to:

- import public benchmark trajectories or a normalized derivative representation;
- convert them into Prefixity request/turn/block structures;
- preserve source provenance and benchmark/task identity;
- identify context growth, divergence, repetition, volatility and structural churn;
- mark externally supplied gold/required context where available;
- generate offline reports and machine-readable observations;
- compare Prefixity recommendations against external labels or known outcomes.

**No live prompt mutation in Phase 1A.**

## Phase 1B — Offline intervention planning

Question:

> Can Prefixity recommend an intervention without silently treating all non-gold context as safely removable?

Allowed recommendation classes:

- `KEEP`
- `DEFER`
- `PRUNE`
- `RELOCATE_CANDIDATE`
- `COMPRESS_CANDIDATE`
- `DO_NOTHING`

Each recommendation must carry reasons/evidence, evidence strength, dependencies, expected structural effect, expected quality risk, provider-state dependence, and whether it is hypothetical only.

Phase 1B remains **offline**.

## Phase 1C — Quality-gated controlled replay

Question:

> When selected interventions are actually replayed, do they improve end-to-end efficiency without materially reducing task success?

Compare at minimum baseline/full context, Prefixity-selected intervention, no-op baseline, and where practical one relevant external specialist/baseline.

Measure task success, required/gold-context retention, total and fresh input, provider cache reads where available, output, tool calls, rounds, repeated reads, recovery behaviour, latency and economic cost.

## Core design principle

**Optimize intervention quality, not token deletion.**

> Maximize avoidable-context reduction subject to a strong preservation constraint on load-bearing context and task success.

A lower token count is not automatically an improvement.

## Context decision model

```text
real agent trajectory
        |
        v
Prefixity Context Manifest
        |
        +----------------------+----------------------+
        |                      |                      |
        v                      v                      v
structural state         quality/dependency      economics/provider state
        |                      |                      |
        +----------------------+----------------------+
                               |
                               v
                      intervention plan
                               |
          KEEP / DEFER / PRUNE / RELOCATE_CANDIDATE
             / COMPRESS_CANDIDATE / DO_NOTHING
```

## Non-goals

Phase 1 does not build another cache-divergence explorer as the primary product, a KV-cache scheduler, a learned code-pruning model, a specialist compressor from scratch, a generic live compression proxy, automatic live mutation before Phase 1C, a large bespoke benchmark before public corpora are exhausted, token-conversion multipliers, semantic response caching, RAG, or long-term memory infrastructure.

## Phase boundaries

### 1A may proceed only when

- workload corpus and licensing/provenance are recorded;
- import/provenance requirements are documented;
- evaluation metrics are frozen enough to avoid post-hoc selection.

### 1B may proceed only when

- 1A ingestion is deterministic;
- workload identity/provenance round-trips;
- observations audit back to source blocks;
- `DO_NOTHING` is a valid outcome.

### 1C may proceed only when

- quality gates are defined before replay;
- recommendations are reproducible;
- load-bearing context preservation is measurable;
- abort/rollback behaviour is specified;
- provider calls, if used, are bounded and explicitly authorized.

## Stopping / pivot conditions

Stop or redesign if Prefixity cannot reliably preserve known required context, recommendations frequently mark load-bearing context removable, savings are erased by rereads/tool calls, cache disruption makes interventions worse, low-risk recommendations degrade quality materially, existing specialist systems solve the whole decision problem better, or ingestion complexity overwhelms value.

## First implementation target after design approval

**Phase 1A only:** build a narrow importer/evaluator path for a small verified subset of a public workload corpus.

Do not begin with automatic pruning or compression.
