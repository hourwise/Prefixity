# Prefixity - Source of Truth

> Current authoritative description of what Prefixity is, what has actually
> been implemented, and which product/architecture decisions have been accepted.

## Status

Prefixity is experimental research software. At the current audit checkpoint,
the bounded current-state marker below is the machine-checkable status aid for
this document. Its semantic checkpoint ID is intentionally stable across
commits that preserve these conclusions; it is not a claim that every later
Git HEAD has the same SHA. Phase 0A and Phase 0A.1 offline work are
implemented. The separate Phase 0B
controlled live-validation harness is implemented, and the controlled
DeepSeek sequence is closed as `PASS WITH RECORDED LIMITATIONS`. OpenAI and
Anthropic adapters remain offline-tested rather than live-validated.

<!-- PREFIXITY_CURRENT_STATE_BEGIN -->
checkpoint_id = phase-1c-research-state-v1
workspace_crates = 4
workspace_members = crates/prefixity-core,crates/prefixity-cli,crates/prefixity-live,crates/prefixity-controlled-benchmark
phase_1c_stage_0 = CERTIFIED
phase_1c_stage_1 = BLOCKED
phase_1c_live_replay = NOT_STARTED
external_front_half = EXTERNAL_TRAJECTORY_PERMISSION_PENDING
controlled_policy_name = controlled-evidence-policy-v1
controlled_policy_scope = CONTROLLED_ONLY
artifact_admission_schema = prefixity.external-artifact-admission.v1
<!-- PREFIXITY_CURRENT_STATE_END -->

Phase 1A natural-workload observation is complete for the accepted
CodeTraceBench evidence path. Phase 1B is complete through the controlled
benchmark design, chronological-world correction, controlled review, and the
1B.9 held-out intervention-recall study. The frozen
`controlled-evidence-policy-v1` remains research-only with scope
`CONTROLLED_ONLY`; the bounded 4/4 positive held-out result is not population
or generalization evidence.

Phase 1C design authorization is complete, and Stage 0 offline replay
certification is complete with result
`CERTIFIED — READY FOR SEPARATE STAGE 1 AUTHORIZATION`. Stage 0 remains valid:
it made no provider/model calls, read no credentials, and did not establish
provider behavior, task-quality preservation, cache behavior, or production
benefit. The later external-evidence gate leaves Phase 1C Stage 1
`BLOCKED`.

Phase 1B.4 completed a narrow evidence-adapter revision justified by the pinned
raw-schema verification. The accepted derivative now preserves source-explicit
message timestamps, bounded provider response metadata, provider-specific raw
usage, field-level provenance and bounded evaluation source locators. The
frozen planner remains conservative: all 719 plans are `DO_NOTHING`, with no
new optional/required/stale/dependency/tool-link evidence synthesized.

The immediate research blocker is obtaining an explicit reuse basis for a
suitable pre-existing external trajectory artifact, currently Tracebench.
This is an external research dependency, not a global project blocker; other
offline and research-infrastructure work remains possible. No
ContextBench/Tracebench adapter or scoring study is authorized or started.

## Product definition

The current product is an experimental, provider-neutral context analysis and
decision-research system. It includes deterministic trace observation,
prefix/cache structural analysis, offline policy simulation, conservative
intervention planning, research-only controlled benchmark/evaluator
infrastructure, bounded live-validation infrastructure from Phase 0B, and
certified offline Phase 1C replay machinery.

The current product boundary does not include production context
optimization, natural-trace intervention safety, universal savings, provider
superiority, live Phase 1C success, or automatic context mutation. The
charter's possible future "context compiler" remains a conditional design
direction, not an implemented production capability.

### Accepted P0-L1 product boundary

Prefixity is local-first context and inference-efficiency infrastructure for
resource-constrained and self-hosted LLM use, while retaining broad
compatibility with proprietary and cloud model APIs through adapters. Local
resource constraints are the tie-breaker when a design trade-off cannot serve
every environment equally, unless an existing accepted decision says
otherwise.

Prefixity is not an inference engine and does not replace llama.cpp, Ollama,
LM Studio, vLLM, SGLang, or provider inference infrastructure. Those systems
own model execution, kernels, scheduling, and backend-specific KV machinery.
Prefixity operates above or alongside them.

The conceptual boundary is Context Compiler, Context/Artifact Identity Layer,
Cache Planner / Cache Intelligence, Inference Budget + Telemetry, and Runtime /
Provider Adapters. This is a research scope boundary, not a claim that all
components are implemented.

The accepted runtime integration priority is llama.cpp, Ollama, LM Studio,
vLLM, then SGLang. This is a research/implementation priority, not a
permanent compatibility restriction. Cloud/provider adapters remain desirable.

Lossless and potentially lossy transformations remain separate. No lossy
change may silently weaken instruction hierarchy, provenance, trust
boundaries, security controls, or model-visible temporal/authority semantics.

## Problem being solved

The current research question is:

> Can a provider-neutral, auditable decision layer determine when accumulated
> agent context should be retained or changed, and when `DO_NOTHING` is
> preferable after accounting for quality, structural evidence, provider
> behavior, and cache economics?

This remains a research hypothesis, not a scientifically established result.
The decision layer makes context-management claims testable by answering, for
an observed workload:

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

The neutral P0 observation vocabulary is implemented in
`prefixity-core::observation`: versioned `ContextArtifact`,
`CacheObservation`, and `RuntimeCacheCapabilities` contracts. They are
observation-only and preserve explicit known, unknown, and not-observed
states. The capability contract distinguishes supported, unsupported, and
unknown from documented, experimentally observed, and unverified evidence.

The P0-L4 cache-conformance foundation extends the existing
`prefixity-controlled-benchmark` crate with versioned provider-neutral
experiment, mutation-case, request, result, and runner-boundary types. Its
in-process mock transport records only traceable identities and explicit
`not_observed` metrics; it is not runtime integration or cache evidence.

The P0-L5 llama.cpp adapter extends that same runner boundary with a typed,
offline-only llama-server request projection and response observer. It uses
synthetic fake transport fixtures and does not establish experimentally
observed llama.cpp behavior.

The P0-L7 diagnostics extend the same neutral request model with versioned
Prefix Diff, Request Envelope Diff, and combined Request Diff results. They
describe model-visible structural divergence and envelope changes with bounded
summaries; they do not rewrite requests or predict cache outcomes.

The P0-L8 diagnostics add a separate, versioned comparison of references to
two P0-L2 `CacheObservation` records and an association-only combination with
P0-L7 `RequestDiff`. They preserve identity/fingerprint references, explicit
known/unknown/not-observed states, independent token/timing/resource deltas,
safe derived ratios with named denominators, bounded directional assessment,
and deterministic evidence statements whose causality remains
`not_established`. They do not copy raw observations, infer cache algebra,
produce a universal score, substitute runtime capabilities for observations,
or fabricate live runtime evidence. See
`docs/phase-0/CACHE_OBSERVATION_DIAGNOSTICS.md`.

P0-L9 adds a separate, versioned `CapabilityRegistry` over the existing
P0-L2 `RuntimeCacheCapabilities` contract. It loads the explicit approved
local/cloud capability fixtures through one bounded offline path, retains
provider/model/protocol/runtime/version identity, computes deterministic
semantic profile fingerprints, and exposes typed queries, evidence-preserving
matrix cells, and research-gap counts. Unknown remains distinct from
unsupported; documented remains distinct from experimentally observed; and
synthetic fixture ingestion does not promote any capability to live evidence.
P0-L9 does not derive registry entries from P0-L8 observations, perform live
runtime/provider work, add ContextBench, or implement P0-L10. See
`docs/phase-0/CACHE_CAPABILITY_REGISTRY.md`.

The workspace has four crates:

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
4. `prefixity-controlled-benchmark` is isolated, offline, research-only
   controlled benchmark and evaluator infrastructure. It owns the
   self-authored controlled envelope, deterministic world/oracle, blinded
   planner projection, Phase 1B.9 study, and Phase 1C Stage 0 certification.
   It is not production runtime infrastructure and does not authorize
   provider calls or live replay.

P0-L10 adds a separate, versioned ContextStabilityAnalysis over the existing
P0-L4 neutral request and optional P0-L2 ContextArtifact metadata. It
preserves stability, lifecycle, trust, explicit-versus-derived classification
source, bounded segment fingerprints/sizes, deterministic boundaries,
stability inversions, and a stability-aligned leading-region observation.
Unknown classifications remain unknown; token analysis remains
not_observed; and no cache outcome, causal claim, or optimization action is
produced. P0-L10 does not reinterpret P0-L9 capabilities or P0-L8
observations. See docs/phase-0/CONTEXT_STABILITY.md.

P0-L11 adds a separate, versioned ContextLayoutPlan over the existing P0-L4
request and P0-L10 analysis. It proposes only bounded reorders of
independently represented context artifacts when explicit movement permission,
ordering, semantic, chronology, and trust constraints establish that the move
is safe to describe. System, user, and tool slots remain fixed; unknown safety
is rejected; lifecycle remains metadata rather than a placement heuristic.
Candidates are re-analysed through P0-L10 and diffed through P0-L7, while every
cache impact remains unknown. P0-L11 does not apply requests, attach runtime
telemetry, predict savings, or introduce provider-specific planning. See
docs/phase-0/CONTEXT_LAYOUT_PLANNER.md.

Phase 1A tooling is repository-level evidence tooling rather than a new runtime
crate: the existing thin importer/adapter in `tools/phase1a_tracebench.py`
preserves source provenance and keeps evaluation labels outside observer inputs.

The trace format is version 2. Blocks carry ordered structural metadata,
content hashes, optional token/content data, flags and dependencies. The
additive Phase 1B.4 evidence schema preserves source-explicit timestamps,
bounded response metadata and typed provenance without changing trace-v2
compatibility. Raw usage is retained with an explicit versioned API-surface
schema. Known offline normalizers cover synthetic, OpenAI Chat Completions,
Anthropic Messages and DeepSeek Chat Completions. OpenAI Responses is
recognized as reserved but is not interpreted.

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
- Phase 1B.4 evidence-adapter evidence: the pinned raw source preserves
  provider response metadata and usage on 719/719 request traces, numeric
  timestamps on 1,498/1,498 source events, and 32/60 exact evaluation-step
  joins from explicit bounded locators. The remaining 28 joins remain
  unresolved; labels stay outside planner inputs. Re-import and repeated
  planner execution are deterministic and safety-clean. See
  `docs/phase-1/PHASE_1B4_EVIDENCE_ADAPTER_RECHARACTERIZATION.md`.
- Phase 1B controlled-evidence path: the corrected chronological-world
  benchmark review established a deterministic bounded measurement mechanism,
  and the 1B.9 held-out study selected four of four authored positive cases
  with no observed false positives, false negatives, unsafe actions, or
  regressions under the frozen controlled construction. This is controlled
  evidence only; it is not population/generalization evidence.
- Phase 1C design and Stage 0: the replay design/authorization gate is
  complete, and the offline Stage 0 runner is certified as
  `CERTIFIED — READY FOR SEPARATE STAGE 1 AUTHORIZATION`. Stage 0 certifies
  offline harness behavior only and remains valid; it made zero network calls,
  read zero credentials, and incurred zero spend.
- Phase 1C external evidence: the corrected ContextBench source is
  EuniAI/ContextBench at revision
  `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`; its pinned artifact provides
  human gold-context/task material but not a permission-cleared external
  trajectory, so its result is
  `NO-GO — BENCHMARK ADMISSION/TRAJECTORY INSUFFICIENT`. The strongest
  technical candidate is Contextbench/Tracebench at observed revision
  `7da2e4f45b330be8b6e8f1cff835247723cb3341`, with 376 exact ContextBench
  task joins and 596 trajectory rows in the bounded metadata join. Raw
  trajectory admission remains blocked because no explicit reuse/licence
  basis was established; its result is
  `NO-GO — NO PERMISSION-CLEARED EXTERNAL TRAJECTORY ARTIFACT FOUND`.
- P0-L2/L3 observation foundation: versioned neutral context-artifact,
  cache-observation, and runtime-capability contracts are implemented with
  serde-compatible Rust types, bounded validation, focused tests, and
  representative local/cloud capability examples. The examples are not
  provider validation; unestablished capabilities remain unknown/unverified.
- P0-L4 cache-conformance foundation: deterministic experiment/case/request
  and result contracts, controlled mutation vocabulary, an in-process mock
  runner, focused validation tests, and one synthetic coding-agent-style
  fixture. It establishes reproducible experiment structure only; it does not
  establish cache behavior or provider/runtime capability.
- P0-L5 llama.cpp adapter foundation: deterministic projection of supported
  neutral requests, explicit rejection of unsupported reasoning settings,
  synthetic response normalization into `CacheObservation`, conflict-safe
  native/usage handling, a fake transport, and documented-only capability
  metadata. This is protocol validation, not llama.cpp runtime evidence.
- P0-L7 request-difference diagnostics: versioned Prefix Diff, Request
  Envelope Diff, and combined Request Diff contracts with deterministic common
  prefix measurements, first-divergence paths, ordered-change taxonomy,
  bounded value summaries, and conservative unknown cache impact. This is
  structural diagnosis, not optimization or cache prediction.
- P0-L8 observation diagnostics: versioned reference-based observation
  comparison with independent deltas, bounded association-only assessment, and
  explicit non-causality.
- P0-L9 capability registry: deterministic ingestion of the approved local and
  cloud capability fixtures, typed identity/evidence queries, generated matrix
  cells, and research-gap reporting. No capability is promoted to observed by
  fixture loading or synthetic protocol tests.

## Incomplete and not established

- No Phase 1C controlled replay, task-quality evaluator, gold-context
  retention measurement or end-to-end quality/cost report exists.
- Phase 1C Stage 1 schema smoke and Stage 2 replay are blocked. The blocker is
  front-half external evidence admission, not Stage 0 harness correctness.
- No permission-cleared external trajectory artifact has been admitted. The
  Tracebench clarification is an external dependency; no raw trajectory data
  has been downloaded, vendored, or committed.
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

Phase 1A remains complete for the accepted `NJU-LINK/CodeTraceBench`
artifact-bearing dataset revision
`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`. Its importer preserves
task/trajectory provenance, keeps evaluation labels separate from decision
inputs, and produces deterministic offline observations. The missing
README-linked `LICENSE` text remains a licence-evidence limitation; it was
not recreated or inferred.

Phase 1B is complete as a controlled research path through the corrected
benchmark review and 1B.9 held-out recall. The result supports bounded
measurement and a conservative fail-open policy, not a production policy,
natural-workload generalization, intervention safety, or universal savings.
The Phase 1 quality gate still requires structural safety, required/dependency
retention, reproducibility, task-quality evidence, and end-to-end accounting.
`DO_NOTHING` remains a valid success outcome.

Phase 1C Stage 0 is certified and valid, but Stage 1 remains blocked. The
current central dependency is explicit reuse clarification or an explicitly
licensed, immutable external trajectory artifact, currently Tracebench. Until
that admission boundary changes, no ContextBench/Tracebench adapter, raw-data
acquisition, scoring study, schema smoke, or replay should begin. This does
not prevent unrelated offline or research-infrastructure work.

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
- Phase 1C Stage 1 schema smoke and Stage 2 replay remain deferred and blocked
  until the external trajectory admission dependency is resolved and each
  later execution scope is separately authorized. Stage 0 is complete;
  automatic compression remains deferred, and Phase 1B.0 only supports its
  contract class.
- P0-L6 runtime integration and cache probing, automatic context
  rewriting, cache routing, KV quantisation, cache simulation, benchmark
  scoring, and public performance claims remain separately deferred. P0-L6 is
  environment-blocked because no existing usable `llama-server` or suitable
  GGUF was available in the inspected environment. P0-L4 and P0-L5 fake
  runners are not live inference, P0-L7 diagnostics do not create runtime
  evidence, and P0-L9 registry knowledge does not create runtime observations.

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
- Observation schemas preserve explicit unknown/not-observed values and do not
  equate cached tokens with tokens removed. Provider/model/protocol/runtime
  identity remains distinct, as do volatility versus lifecycle and cache
  persistence versus conversation chaining or KV caching.
- Conformance mutation classes describe controlled changes only; they do not
  imply cache hits, misses, invalidation, or provider behavior. Mock results
  keep cache/token/timing values `not_observed` and retain synthetic transport
  telemetry separately.
- The llama.cpp adapter does not infer cache hits, residency, persistence,
  slots, reconstructed context, wall time, or performance. Absent values stay
  `not_observed`; explicit zero remains known; conflicting native and usage
  values fail rather than being silently reconciled.
- Prefix and envelope diagnostics describe differences only. They preserve
  request fingerprints, report token-level common prefixes as `not_observed`
  without a tokenizer, bound changed-value summaries, and keep cache impact
  `unknown` unless a future runtime observation explicitly supports a claim.
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
- Whether Tracebench or an equivalent external trajectory artifact has an
  explicit reuse basis for bounded local research; the current no-go is a
  conservative research-admission decision, not a legal finding.
- Whether ContextBench's gold-context/task material and the underlying source
  repositories can be used for any future slice without copying or
  redistributing restricted benchmark material.

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
- Earlier Phase 1 documents intentionally preserve the design or checkpoint
  state in which they were written. The current Phase 1B/1C status and
  external-evidence boundary are recorded here and in the later evidence
  documents; historical wording is not retroactively rewritten.
- The Phase 0B closeout and findings are the authoritative record for the
  controlled DeepSeek result; ignored `experiments/runs` artifacts are useful
  evidence but are not tracked benchmark outputs.
