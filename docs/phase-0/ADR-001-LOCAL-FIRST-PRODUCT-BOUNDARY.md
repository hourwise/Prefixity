# ADR-001 — Local-First Product Boundary

- Status: Accepted
- Date: 2026-08-12
- Scope: P0-L1

## Context

Prefixity currently provides provider-neutral observation, structural context
analysis, offline policy simulation, controlled research infrastructure, and
bounded live-validation infrastructure. Its research question spans local
self-hosted inference and proprietary/provider APIs, but the hardest practical
constraints are concentrated in local and resource-constrained deployments:
RAM, VRAM, KV-cache capacity, context capacity, CPU-heavy inference,
CPU/GPU offload, and prefill time.

Prefixity must therefore choose a centre of gravity without removing cloud
compatibility or pretending to own model execution.

## Decision

Prefixity is local-first context and inference-efficiency infrastructure for
resource-constrained and self-hosted LLM use, while retaining broad
compatibility with proprietary and cloud model APIs through adapters.

When a design trade-off cannot satisfy every environment equally, resource-
constrained local inference is the tie-breaker unless an existing accepted
project decision explicitly says otherwise.

Prefixity is not an inference engine and is not a replacement for llama.cpp,
Ollama, LM Studio, vLLM, SGLang, or provider inference infrastructure. Those
systems own model execution, kernels, scheduling, and backend-specific KV
machinery. Prefixity operates above or alongside them.

The conceptual product boundary is:

    Prefixity
    ├── Context Compiler
    ├── Context/Artifact Identity Layer
    ├── Cache Planner / Cache Intelligence
    ├── Inference Budget + Telemetry
    └── Runtime / Provider Adapters

This is a scope boundary and research direction, not authorization to
implement every component in P0-L1–L3.

## Responsibilities in scope

Future Prefixity work may optimize or observe context selection, duplicate or
superseded context removal, deterministic/canonical representation,
stable-versus-volatile placement, tool-schema canonicalisation, model-visible
tool projection, context compaction, structured continuation/checkpoint state,
reusable-prefix identity, cache residency, restore-versus-rebuild cost,
prompt/prefill time, RAM/VRAM/KV pressure, provider/runtime cache telemetry,
request-envelope changes, and cloud cache economics where applicable.

The current foundation only defines neutral observation and capability
contracts. It does not implement those optimisations or integrations.

## Safety boundary

Lossless and lossy transformations remain distinct. Prefixity may eventually
automate transformations demonstrated to be semantically lossless and
policy-safe. Potentially lossy changes, including KV-cache quantisation,
aggressive semantic compaction, dropping potentially relevant context,
changing model precision, or changing reasoning/inference settings, require
explicit policy and, where relevant, measured task-quality regression evidence.
They must never occur silently for performance.

Optimising cache reuse must never silently weaken instruction hierarchy,
provenance, trust boundaries, security controls, or model-visible
temporal/authority semantics.

## Runtime priority

The current local integration research/implementation priority is:

1. llama.cpp
2. Ollama
3. LM Studio
4. vLLM
5. SGLang

This is a priority for research and implementation, not a permanent
compatibility restriction. Cloud/provider adapters remain desirable wherever
reasonably supportable.

## Consequences

- Local resource constraints are the tie-breaker for future architecture
  decisions.
- Provider/model/protocol/runtime identity must remain explicit; one provider
  name cannot stand in for all cache semantics.
- Runtime adapters remain responsible for execution and native KV machinery.
- Unknown or unverified capability claims remain explicit rather than being
  encoded as unsupported.
- P0-L4 and later work must be separately authorized.

## Deferred by this ADR

This decision does not authorize automatic context rewriting, tool selection,
cache-aware routing, runtime integrations, cache restoration, KV quantisation,
cache simulation, benchmark scoring, ContextBench integration, or public
performance claims.
