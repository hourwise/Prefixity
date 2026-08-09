# Prefixity - Source of Truth

> Current authoritative description of what Prefixity is, what has actually
> been implemented, and which product/architecture decisions have been accepted.

## Status

Prefixity is experimental research software. At the audit point, `main` is at
commit `72f06b0` (the Phase 1 design set). Phase 0A and Phase 0A.1 offline
work are implemented; the separate Phase 0B controlled live-validation
harness is implemented, and the controlled DeepSeek sequence is closed as
`PASS WITH RECORDED LIMITATIONS`. OpenAI and Anthropic adapters are tested
offline but have not been live-validated.

Phase 1A corpus validation is implemented and closed as `PASS` for the accepted
CodeTraceBench evidence path. The thin importer/adapter and offline observer
evidence are present. Phase 1B.0 now has a deterministic, conservative offline
intervention-plan contract and baseline planner. Phase 1C replay/runtime work
remains unimplemented and is not authorized by the current task.

Phase 1B.1 ran the frozen Phase 1B.0 planner over all 719 accepted offline
request traces. All 719 plans produced `DO_NOTHING`; the run was deterministic
and safety-clean. The accepted derivative representation contains no evidenced
true `optional`, `required` or `stale` flags, no dependency edges, no provider
usage, and insufficient structural identity for exact step-level evaluation
joining.

Phase 1B.2 established that the currently available accepted derivative
artifacts are insufficient to justify a positive importer revision or planner
tuning. The exact upstream raw trajectory schema has not yet been verified
because the raw artifacts were not available in the accepted local fixture.
This finding applies to the inspected derivative representation; CodeTraceBench
itself has not been proven unsuitable.

## Product definition

The current product is a provider-neutral, deterministic context-efficiency
profiler and offline policy simulator for recorded or synthetic LLM/agent
request traces. It explains structural context cost, prefix divergence,
provider-reported usage and hypothetical policy effects before any live prompt
mutation.

The implemented product boundary is the Phase 0 observer/simulator plus the
disposable Phase 0B validation harness. The charter's possible future
"context compiler" is a conditional design direction, not an implemented
product capability.

## Problem being solved

The hypothesis is that a deterministic tool can make context-management
decisions testable by answering, for an observed workload:

- where context cost is incurred;
- where consecutive request structures first diverge;
- what prefix is structurally reusable between recorded requests;
- what the provider actually reported as cache read/write/fresh input;
- which context is a heuristic fresh-input driver;
- whether a hypothetical policy could improve token or profile-based cost;
- whether an apparent optimization is economically negative; and
- when the evidence is too weak and `DO_NOTHING` is the correct result.

The Phase 1A CodeTraceBench run establishes only that the observer can process
one accepted natural multi-turn trajectory slice offline and emit heuristic
structural observations. It does not establish intervention safety, preserve
task quality under intervention, or produce end-to-end savings.

## Current architecture

The workspace has three crates:

1. `prefixity-core` is authoritative for the trace model, validation, bounded
   input handling, structural zones/fingerprints, token estimation, the
   explainable `prefixity` heuristic, single-trace analysis, trace comparison,
   provider-usage normalization, cost arithmetic, non-mutating policy
   simulation and the Phase 1B offline intervention decision contract.
2. `prefixity-cli` is a thin offline command-line layer over the core. It
   exposes `validate`, `analyse`, `compare`, `simulate` and `plan`, with
   deterministic human and JSON output. It reads trace/profile files and does
   not mutate live requests.
3. `prefixity-live` is disposable experimental infrastructure for controlled
   provider calls. It generates deterministic synthetic scenarios, uses
   allowlisted provider endpoints and environment-only credentials, converts
   responses into trace v2, preserves raw usage, reconciles pair ratios and
   writes sanitized local artifacts. It delegates analysis and normalization
   to `prefixity-core`.

Phase 1A tooling is repository-level evidence tooling rather than a new runtime
crate: the existing thin importer/adapter in `tools/phase1a_tracebench.py`
preserves source provenance and keeps evaluation labels outside observer inputs.

The trace format is version 2. Blocks carry ordered structural metadata,
content hashes, optional token/content data, flags and dependencies. Raw usage
is retained with an explicit versioned API-surface schema. Known offline
normalizers cover synthetic, OpenAI Chat Completions, Anthropic Messages and
DeepSeek Chat Completions. OpenAI Responses is recognized as reserved but is
not interpreted.

## Implemented

- Structural validation: format/version checks, non-empty identities,
  contiguous positions, unique bounded IDs, SHA-256/content consistency,
  UTF-8 byte-count checks and metadata/dependency/content limits.
- Explainable prefixity scoring: source-type baselines, optional/stale
  penalties, lifetime adjustment and required-block reasons. The score is a
  deterministic heuristic, not a probability, prediction or ML result.
- Single-trace analysis: block summaries, estimated tokens, candidate-prefix
  accounting, heuristic volatile-block attribution, schema-aware usage
  normalization, reconciliation notes, optional profile cost and conservative
  recommendations.
- Trace comparison: first structural divergence, changed/added/removed/
  reordered positions, observed reusable prefix estimate and separate
  provider-reported cache-read values.
- Cost modeling: externally supplied profiles and a deliberately simple,
  labelled hypothetical cache-economics model. All committed profiles are
  synthetic.
- Policy simulation: baseline, within-zone stable-first, optional volatile
  deferral, stale tool-output pruning and combined simulation. Decisions use
  indices and do not mutate the input trace. Required blocks are retained;
  chronological message order and zone constraints are enforced; unsafe moves
  are reported as deferred. Compression is reserved.
- Phase 1B.0 decision layer: versioned `InterventionPlan` contract with exactly
  `KEEP`, `DEFER`, `PRUNE`, `RELOCATE_CANDIDATE`, `COMPRESS_CANDIDATE` and
  `DO_NOTHING`; deterministic audit fields; fail-open dependency closure;
  required/protocol/current-request protection; explicit-metadata-only prune
  and defer cases; hypothetical within-zone relocation candidates; and a
  `prefixity plan <trace> --json` CLI path. Compression is contract-only.
- Phase 0B harness: schema-smoke, stable-prefix, early-divergence and
  late-divergence plans; request-count and local estimate ceilings; explicit
  `--execute-live` opt-in; no automatic retries; TLS verification; no redirects;
  environment-only credentials; raw usage and sanitized trace/result artifact
  writing.
- Validation material: 21 documented fixture scenarios represented by 26
  trace files (including sanitized DeepSeek-derived fixtures), synthetic
  profiles, unit/integration tests, mock-transport live-pipeline tests and
  recorded DeepSeek artifacts that are ignored by Git.
- Phase 1A corpus evidence: the existing thin importer/adapter accepted the
  `NJU-LINK/CodeTraceBench` `verified` slice at revision
  `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`. Deterministic import,
  provenance and evaluation-label separation passed; 24 trajectories produced
  719 offline request traces, the observer processed 719/719 successfully,
  and 712 structural candidates plus 7 `DO_NOTHING` cases were observed.
  These are heuristic structural observations only, not validated safe
  interventions, provider cache reuse, monetary savings, latency improvement
  or task-quality preservation.
- Phase 1B characterization evidence: the frozen planner produced
  `DO_NOTHING` for all 719 accepted request traces in Phase 1B.1. The result was
  deterministic and safety-clean; this does not establish positive intervention
  coverage.

## Incomplete and not established

- No Phase 1C controlled replay, task-quality evaluator, gold-context
  retention measurement or end-to-end quality/cost report exists.
- OpenAI and Anthropic live behavior remains untested in this repository;
  their adapters are exercised with mocks/offline schemas only.
- OpenAI Responses usage normalization/live adapter is reserved and absent.
- No audited current provider pricing profiles, provider tokenizer, universal
  token conversion, latency benchmark, performance benchmark, intervention
  quality result or end-to-end natural-agent workload benefit result exists.
- The live evidence is one controlled DeepSeek sequence per scenario on a
  synthetic corpus. It does not prove production value, causation,
  determinism, cross-provider behavior, model generality or cost savings.

## Accepted near-term direction

Phase 1A is complete for the accepted `NJU-LINK/CodeTraceBench` artifact-bearing
dataset revision `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`: use a narrow,
verified subset of a public agent-workload corpus, preserve task/trajectory
provenance, keep evaluation labels separate from decision inputs, and produce
deterministic offline observations before any mutation or replay. The Phase 1
plan's Phase 1B prerequisites for ingestion, provenance, label separation and
deterministic offline observation are now met. Phase 1B.0 establishes the
offline contract and conservative baseline only; it does not establish
intervention quality, savings or replay readiness.

The next phase should validate the central decision hypothesis rather than
expand the feature surface. The Phase 1 quality gate requires structural
safety, required/dependency retention, reproducibility, task-quality evidence
and end-to-end accounting. `DO_NOTHING` is a valid success outcome.

The accepted corpus declaration and exact source revision are recorded in the
Phase 1A closeout and fixture provenance. The missing README-linked `LICENSE`
file remains a licence-evidence limitation; its text was not recreated or
inferred. The observed candidates remain heuristic and do not establish
provider cache reuse, monetary savings, latency improvement or task-quality
preservation.

The next authorized research question is raw-artifact access and upstream-schema
verification for the exact pinned CodeTraceBench revision. It must establish
whether explicit step, action, observation or tool-reference identity exists
before any importer revision is considered.

## Explicitly deferred

- Automatic live prompt mutation, daemon/proxy/GUI/authentication/telemetry
  and persistent storage.
- Automatic compression, semantic response caching, KV-cache management,
  RAG/repository indexing and long-term memory infrastructure.
- Reimplementing provider-native or server-side KV/prefix caching.
- Learned pruning/compression as the core architecture.
- Token-conversion multipliers and hard-coded current provider pricing.
- OpenAI Responses support until its exact versioned schema is implemented and
  validated.
- Phase 1C runtime and replay work remains deferred until separately
  authorized after Phase 1B characterization and quality-gate preparation.
  Automatic compression remains deferred; Phase 1B.0 only supports its
  contract class.

## Constraints and invariants

- Original source/provider state outranks derived Prefixity state; any future
  Prefixity storage must be disposable and rebuildable.
- Observation precedes transformation, and simulation precedes automatic
  optimization. The future optimizer must fail open to the original request.
- A single trace cannot prove reuse. Prefixity score, observed structural
  reuse and provider-reported cache reuse remain separate concepts.
- Provider-reported usage outranks heuristic candidates when describing what
  actually happened. Absolute counts from different tokenizers are not
  silently subtracted; live reconciliation is ratio-based and explicitly
  labelled.
- Required blocks are never removed. Policies do not move blocks across
  incompatible semantic zones, do not reorder chronological message content,
  and label applied within-zone reorders experimental.
- Unknown usage schemas do not manufacture values. Raw provider usage is
  preserved verbatim, while normalization is schema-aware.
- Committed fixtures contain no credentials or private source. Content may be
  omitted in favor of hashes/metadata. Terminal output sanitizes untrusted
  strings and input handling is bounded.
- Repository profiles are synthetic. Their prices are test data, not current
  provider pricing.
- Live calls require explicit opt-in, sequential bounded requests, a local
  Prefixity-estimate ceiling, environment-only credentials, no automatic
  retries and fixed/allowlisted artifact behavior.

## Known uncertainties

- Whether structural reuse potential is predictive or operationally useful on
  natural multi-turn agent workloads.
- Whether any intervention reduces end-to-end fresh input, latency, tool calls
  and economic cost after rereads, recovery turns, output and lost cache reuse
  are counted.
- Whether recommendations preserve task success, protocol validity and
  load-bearing/dependency-required context. The current Phase 0 flags are not
  quality labels.
- How provider serialization, hidden cache-unit boundaries, tokenization,
  persistence and expiry affect structural comparisons across providers,
  models, regions and time.
- Whether the observed DeepSeek late-divergence persistence result was caused
  by the changed request, cache construction, the settle interval or a
  combination. The repository explicitly does not isolate causation.
- Whether OpenAI and Anthropic live usage semantics and cache behavior align
  with the existing adapters.
- Whether a provider-neutral decision layer offers material value beyond
  provider-native diagnostics and overlapping pruning/compression/cache tools.
- Whether the missing README-linked `LICENSE` file at the accepted
  CodeTraceBench revision can be recovered from upstream without inference;
  the exact revision, metadata declaration and README evidence are recorded,
  and the missing text has not been recreated.
- Whether the 712 structural candidates survive intervention-quality,
  provider-cache, monetary-savings, latency and task-quality evaluation.

## Documented disagreements and drift

- The Phase 0A charter, plan, experiments note and threat model contain wording
  that says live calls/credentials are out of scope or future work. That was
  true of the original offline phase, but it is not a complete description of
  the current tree because `prefixity-live` and the Phase 0B closeout exist.
- The Phase 0 plan describes eight fixture scenarios; the current fixture map
  documents 21 scenarios across 26 trace files. This summary follows the
  current fixture directory and tests; the older deliverable count is retained
  as historical plan text.
- `prefixity-live` comments in `lib.rs` and `experiment.rs` still mention a
  default request count of 3, while the current CLI constant and live protocol
  use a default of 4 to support the four-turn DeepSeek late-divergence plan.
- The Phase 0B closeout and findings are the authoritative record for the
  controlled DeepSeek result; ignored `experiments/runs` artifacts are useful
  evidence but are not tracked benchmark outputs.
