# Phase 1 Prior-Art Decisions

## Purpose

Record what recent public work means for Prefixity scope so the project does not rebuild solved components.

Re-check source versions and licences before integrating external code or datasets.

## Decision summary

Prefixity should **not** become another cache divergence viewer, prompt compressor, learned context pruner, KV-cache scheduler or semantic response cache.

Its Phase 1 role is the **provider-neutral decision layer** that determines which intervention, if any, is justified.

## ContextBench

**Decision: REUSE / BENCHMARK AGAINST.**

Use as a primary Phase 1A workload/evaluation source where licence and dataset terms permit.

Prefixity still adds structural churn, cache implications, realized provider evidence where available, economics and intervention selection.

## Tool-output pruning systems (e.g. Squeez-style work)

**Decision: DO NOT REIMPLEMENT AS CORE.**

Use later as an external baseline or optional intervention adapter.

Prefixity's job is deciding **whether** pruning is justified.

## ACON

**Decision: REUSE EVALUATION IDEAS; DO NOT ADOPT AS CORE ARCHITECTURE.**

Its key lesson is that context compression must be measured against task outcome, not token count alone.

## LaMR-style structured pruning

**Decision: REFERENCE / BENCHMARK, NOT REIMPLEMENT.**

Key lesson: semantic irrelevance does not imply structural dispensability. Prefixity must model protocol/dependency necessity separately from semantic relevance.

## Cache-aware prompt compression (CAPC-style work)

**Decision: ADOPT THE ECONOMIC PRINCIPLE; DO NOT DUPLICATE ONE COMPRESSION METHOD.**

A smaller prompt can be economically worse if it destroys useful prefix reuse.

Prefixity must distinguish structural reuse potential, realized provider reuse, fresh processing, transformation effect and end-to-end outcome.

## CacheWise / server-side KV management

**Decision: DIFFERENT LAYER — DO NOT IMPLEMENT.**

Observe/integrate with host/provider cache systems where useful; do not become a serving-layer KV manager.

## VS Code cache diagnostics

**Decision: CACHE-DIVERGENCE OBSERVABILITY ALONE IS NOT A PRODUCT DIFFERENTIATOR.**

Prefixity must answer not only “where did the cache break?” but “what should be done, if anything, and what are the quality/economic consequences?”

## Provider-native caching / vLLM / LMCache

**Decision: OBSERVE / INTEGRATE, NEVER REIMPLEMENT THEIR KV LAYER.**

Provider-reported usage remains source-of-truth evidence for what actually happened.

## Semantic response caching

**Decision: OUT OF SCOPE.**

## Reuse / integrate / differentiate matrix

| Area | Reuse | Integrate | Differentiate |
| --- | --- | --- | --- |
| Gold-context benchmarks | Yes | Import/evaluate | Add structural/cache/economic decision layer |
| Tool-output pruners | Baseline | Optional adapter | Decide whether pruning is justified |
| Learned compressors | Evaluation ideas | Comparator | Deterministic decision layer |
| Structured pruning | Benchmark | Dependency lessons | Auditable provider-neutral decisions |
| Cache-aware compression | Economic lesson | Compare outcomes | Generalize beyond one compressor |
| KV-cache management | No | Observe metrics | Stay above serving layer |
| Cache explorers | Diagnostic reference | Import signals | Go beyond divergence diagnosis |
| Semantic response cache | No | No | Explicitly out of scope |

## Phase 1 differentiation statement

> Prefixity evaluates observed context and decides which context-management intervention — keep, defer, prune, relocate, compress, or do nothing — is justified by structural evidence, dependency/quality risk, realized provider state and end-to-end economics.

This is a design target, not a validated product claim.

## Integration policy

Before importing external code/data:

- record exact source/version;
- verify licence;
- prefer adapters over forks;
- keep specialist integrations outside `prefixity-core`;
- do not make learned systems mandatory;
- ensure Prefixity remains useful offline without provider credentials.

## Sources to retain in research notes

Maintain original/official references for ContextBench, tool-output pruning work, ACON, structured context/memory reduction, cache-aware prompt compression, CacheWise, current VS Code cache diagnostics, vLLM/LMCache and provider-native prompt caching.
