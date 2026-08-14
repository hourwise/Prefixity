# Active Task — Phase 0 Foundation Slice (P0-L13)

Status: P0-L6C-R1 repair complete; P0-L6C Attempt 002 is preserved as
runtime-blocked / failed before inference, Attempt 003 is readiness-blocked
before inference, Attempt 004 is readiness-blocked before preflight, and
Attempt 005 is an execution-invalidated partial live run caused by a missing
generation bound in the projected request. P0-L6C remains incomplete; no
further live attempt is authorized by this run.
P0-L1 through P0-L5 and P0-L7 through P0-L13 remain complete.

## P0-L6A completion record

- Added a versioned loopback-only HTTP transport boundary for the existing
  P0-L5 llama.cpp adapter. It accepts only HTTP `127.0.0.1`, `::1`, or the
  literal `localhost`, uses explicit connect/request timeouts, disables
  redirects and retries, bounds response bodies, and emits typed bounded
  failures.
- Added an explicit `execute_live` opt-in that defaults to false, plus a
  machine-readable zero-network preflight/readiness record. Ordinary tests,
  builds, and the preflight path do not contact real sockets.
- Added versioned runtime configuration, unknown-preserving environment
  manifest types, P0-L13 certificate/pair validation, deterministic identity,
  and the fixed A1/A2/C1/C2/B1/A3/C3 control/treatment/interference sequence.
  B1 is deterministic and structurally early-different; no tuning or current
  model-specific configuration is hardcoded.
- Kept raw response telemetry, P0-L5 normalization, `CacheObservation`, and
  `ConformanceResult` as separate stages. Partial/failed runs cannot carry a
  final normalized result; no P0-L8/P0-L12 admission is performed here.
- Added focused offline tests for endpoint safety, opt-in gating, bounded
  configuration, unknown environment fields, deterministic sequence identity,
  zero-network preflight, and partial-run safety.
- Validation for this slice covers focused tests, workspace formatting/checks,
  strict clippy, full offline workspace tests, and diff review. No live
  inference, llama-server startup, GGUF loading, localhost request, benchmark
  inspection, or P0-L14 work is authorized or performed.

P0-L6 itself is not complete. A suitable local llama-server/GGUF environment,
separately authorized live execution, and recorded evidence are still pending.

## P0-L6B completion record

- Added a separate paired-mutation experiment definition; the P0-L6A
  A1/A2/C1/C2/B1/A3/C3 definition remains intact and unchanged.
- Added synthetic stable-A / volatile-V / stable-B requests with explicit
  P0-L10 metadata, a moderate bounded workload, deterministic V0→V1 content
  mutation, and explicit movement permission for the approved P0-L11 layout.
- Built C0 from A0 and C1 independently from A1 through the existing
  P0-L11/P0-L12/P0-L13 planning, evaluation, and materialization path. Both
  treatment states carry independent safety certificates.
- Added exact P0-L7 mutation/layout comparisons, P0-L10 inversion/leading-region
  records, deterministic A0/A1/B1/C0/C1 sequencing, semantic identity, a
  caller assertion for `fresh_server_for_run`, and a no-required-direction
  primary cache/prefill outcome contract.
- Reused the P0-L5 normalization and P0-L6A raw-evidence/opt-in flow; no
  duplicate observation implementation or automatic retry/admission was
  introduced. Added 20 focused offline paired-mutation tests.
- P0-L6B prepares this experiment but produces no live evidence. Exploratory
  llama.cpp mutation measurements motivate the hypothesis only; they are not
  Prefixity improvement evidence. No inference, localhost contact, model
  loading, benchmark rerun, tokenizer, statistics, ContextBench, or P0-L14
  work occurred.

## P0-L6C first live attempt record

- Baseline was verified before execution: `main`, HEAD and `origin/main` at
  `fda0e9b9ec13cc6e61d655b82a6d9d8337c224d6`, clean worktree, no Git lock, and
  the approved manually started loopback runtime present.
- Bounded non-inference metadata established `llama.cpp` build
  `b10217-ddd4ec142`, model alias `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, Q4_0
  quantization, context 8192, one slot, and metrics enabled. Threads, batch,
  KV precision, GPU offload, and other unqueried settings remain unknown.
- The P0-L6B dry-run passed with zero network/completion calls, exact
  `A0/A1/B1/C0/C1` sequence, independent C0/C1 certificates, exact P0-L7
  diffs, P0-L10 records, fresh-server assertion, bounded workload, and frozen
  identity. The live harness then failed before the first transport call
  because both A0 and C0 were classified as baseline while the conformance
  validator requires exactly one baseline case.
- Accurate final state is `preflight-blocked`/failed-before-inference with
  `completed_steps=0`, no raw response evidence, no normalized result, no
  P0-L8 diagnostics, and no P0-L12 runtime evaluation. No retry was attempted;
  no live result, performance claim, or capability promotion is supported.
- Bounded local artifacts are preserved under the ignored
  `experiments/runs/p0-l6c-first-live-paired-mutation-01/` directory. Because
  all five inference requests did not complete, no P0-L6C commit or push was
  authorized.

## P0-L6C-R1 repair record

- Isolated the root cause to paired-harness baseline classification: the
  generic conformance experiment had mapped both A0 and C0 to the baseline
  relationship, while the unchanged generic contract requires exactly one
  baseline case.
- Added the narrow backward-compatible `ArtifactOrder` mutation class. A0 is
  now the sole generic baseline; A1 is a volatile mutation of A0; B1 remains
  an interference mutation in the A0 lineage; C0 is an artifact-order/layout
  mutation of A0; and C1 is a volatile mutation of C0.
- Centralized construction in the validated
  `build_paired_mutation_conformance_experiment` helper. Both preflight and
  execution use that same five-case `ConformanceExperiment`, and construction
  runs the ordinary generic `ConformanceExperiment::validate()` path before
  readiness or transport access.
- Added focused regressions for exactly one A0 baseline, truthful C0 layout
  semantics, duplicated-baseline failure at the shared validation boundary,
  zero-network preflight, offline fake execution, deterministic sequencing,
  and preservation of P0-L7/P0-L10/P0-L13 contracts. The generic exactly-one-
  baseline invariant was not weakened.
- Attempt 001 remains historical `preflight-blocked`: zero inference
  requests, no runtime observations, no experimental cache evidence, and no
  causal result. Its ignored artifacts were not overwritten or reused. No
  tuning, ContextBench, or P0-L14 work occurred.

## P0-L6C Attempt 002 execution record

- The baseline was re-verified on `main` at
  `2d0672a96bf43cd1b4b427c29a7b44765dc5f232`, matching `origin/main`, with a
  clean worktree and unchanged Attempt 001 artifact hashes. CI run `#55`
  (`31720338764`) was green before execution.
- The shared R1 gate and full preflight passed offline with zero network calls,
  exact `A0/A1/B1/C0/C1` sequencing, one A0 baseline, independent C0/C1
  safety certificates, the Qwen3.5 Q4_0 runtime identity, 8192 context, one
  slot, metrics enabled, and all unspecified tuning fields preserved as
  unknown. Attempt 002 was frozen under the distinct ignored evidence path
  `experiments/runs/p0-l6c-first-live-paired-mutation-02/`; Attempt 001 was
  not modified.
- The single live execution attempted A0 once and stopped at the first
  transport timeout (`request_timeout`) without retry. Operator-observed
  llama.cpp output confirms that the intended server was listening, accepted
  A0, and began prompt processing before the client request timed out. The
  bounded run record is therefore `failed`, with `completed_steps=0`, one
  attempted transport request, zero complete raw response records, zero
  completed inference cases, no normalized result, and no P0-L8/P0-L12 result.
- The exact Attempt 002 client configuration was `connect_timeout_ms=1000` and
  `request_timeout_ms=30000`. The observed cancellation at roughly 31 seconds
  is consistent with that configured client request timeout. The server console
  output is execution-diagnostic evidence only; it is not P0-L5 cache telemetry
  or experimental cache evidence.
- No live conclusion, performance claim, capability promotion, tuning change,
  second experiment, ContextBench integration, or P0-L14 work is supported.
  Attempt 002 artifacts and the bounded failure status are preserved locally
  for review. Because the five-request sequence did not complete, no commit or
  push was performed.

## P0-L6C Attempt 003 execution record

- The exact Attempt 002 timeout audit was completed before this attempt:
  `connect_timeout_ms=1000` and `request_timeout_ms=30000`. The observed
  cancellation at roughly 31 seconds is consistent with that client timeout;
  the corrected diagnosis above is retained.
- Attempt 003 was prepared with the unchanged semantic experiment ID
  `0c8d479c092359941747f640552077400bc61e88b56a6ebd70c9ccfec9dd4a11`, a
  distinct intended evidence path
  `experiments/runs/p0-l6c-first-live-paired-mutation-03/`,
  `request_timeout_ms=600000`, `connect_timeout_ms=1000`, and
  `fresh_server_for_run=true`. The repaired R1 gate and full offline preflight
  passed with zero network calls and the exact five-case sequence.
- The operator-restarted `llama` process was present, but two bounded,
  non-inference listener checks over approximately 90 seconds did not observe
  a listener on `127.0.0.1:8080`. No HTTP request, connectivity completion,
  warm-up, model token, transport attempt, or inference request was sent.
  Attempt 003 therefore stopped before live execution and has no run evidence
  beyond this bounded readiness status.
- No Attempt 003 inference result, cache conclusion, P0-L8 diagnostic,
  P0-L12 evaluation, commit, or push is authorized from this readiness-blocked
  state. No tuning, second experiment, ContextBench integration, or P0-L14
  work occurred.

## P0-L6C Attempt 004 execution record

- The operator supplied the required console confirmation that a freshly
  restarted llama.cpp instance reported `model loaded` and
  `listening on http://127.0.0.1:8080`.
- The authorization permits a bounded non-inference TCP listener check, and
  that check found no active listener on `127.0.0.1:8080` despite the supplied
  console confirmation. The Attempt 004 fresh-server gate therefore failed,
  as required by the authorization, and execution stopped before the shared
  preflight.
- Attempt 004 used no transport, sent no HTTP request, performed no inference,
  processed zero model tokens, and created no evidence directory. No A0/A1/B1/
  C0/C1 case ran; no P0-L8 comparison or P0-L12 evaluation exists. No llama.cpp
  process was started, stopped, or restarted by this task.
- The approved Attempt 004 configuration was not applied: no runtime request
  timeout or execution identity was frozen, and no live attempt occurred.
- A live attempt number is consumed when substantive readiness/runtime
  observations are recorded, even if no inference request is sent. Attempt
  004 must not be reused. That earlier next-action note is superseded by the
  Attempt 005 result below; no further live attempt is authorized by this run.
  No tuning, second experiment, ContextBench integration, or P0-L14 work
  occurred.

## P0-L6C Attempt 005 execution and R2 forensic record

- The repository baseline was verified at
  `d31c6dcdbdc0f9143ffd018c1b65cba3616691da`, with `main` aligned to
  `origin/main`, only the two pre-existing status-document modifications, no
  Git lock, and the existing Attempt 005 directory containing only six bounded
  preparation/identity artifacts. The operator supplied current `model loaded`
  and `listening on http://127.0.0.1:8080` confirmation, and the one permitted
  non-inference
  listener check returned positive.
- The repaired shared preflight passed with zero network calls, exact
  `A0/A1/B1/C0/C1` sequencing, the single A0 baseline, independent C0/C1
  safety certificates, `generation_limit=1`, and the approved
  `connect_timeout_ms=1000` / `request_timeout_ms=600000` configuration. The
  semantic experiment ID remained
  `0c8d479c092359941747f640552077400bc61e88b56a6ebd70c9ccfec9dd4a11`.
- Operator console diagnostics show server task 0 prompt evaluation of 1172
  tokens in approximately 12.6 seconds, followed by 7020 generated tokens in
  approximately 503.4 seconds, filling the 8192-token context and reporting
  `truncated=1`. The generation bound was absent from the pre-repair
  serialized request because `LlamaCppLiveConfig.generation_limit` was not
  carried into `LlamaCppRequest`. The existing adapter contract uses the
  OpenAI-compatible `max_tokens` field; R2 now projects and validates
  `max_tokens=1` for every live case. The console's `graphs reused=6992` is
  retained as execution-diagnostic information only, not cache-token
  evidence.
- `ConformanceExperiment::run` submits cases sequentially: the prior
  `chat_completion()` must return successfully, normalization must complete,
  and only then is the next case submitted. Therefore task 7026, if issued by
  this runner, can only be A1. Its identity is not independently persisted,
  so the corrected accounting distinguishes evidence: one Prefixity
  transport attempt is proven; a second A1 attempt is inferred only if task
  7026 belongs to this runner; two server-side tasks were observed, one
  completed and one canceled; zero Prefixity completed cases, complete raw
  responses, and normalized case results were retained.
- No run record, P0-L8 comparison, or P0-L12 evaluation was produced. The
  paired path constructs its run record only after the full sequence returns,
  which is a documented partial-run durability gap; incremental persistence
  was not added in R2. The original six Attempt 005 artifacts remain
  unmodified, and separate ignored `forensic-r2.json` preserves their
  pre-repair hashes and bounded diagnostic reconciliation.
- Attempt 005 is `execution-invalidated` / a partial live run caused by the
  generation-bound projection defect. It is not a result for or against the
  hypothesis: `hypothesis=unmeasured / insufficient` and
  `causality=not_established`. No retry, Attempt 006, tuning, second
  experiment, ContextBench integration, or P0-L14 work occurred.

## P0-L13 completion record

- Added the neutral `materialize_candidate` boundary over the existing P0-L4
  `ConformanceRequest`, P0-L11 `ContextLayoutPlan`/`LayoutCandidate`, and
  P0-L12 `CandidateEvaluation`. The existing P0-L11 request reconstruction
  path is reused; no parallel request representation or provider projection
  was introduced.
- Added typed, bounded materialization failures for stale source, candidate or
  evaluation identity mismatch, unsafe candidates, unsupported transformations,
  artifact omission/duplication/content drift, tool or envelope changes,
  trust/provenance drift, planned/actual diff mismatch, and P0-L10 structural
  re-analysis mismatch.
- Added deterministic `MaterializationSafetyCertificate` invariants covering
  source/candidate identity, authorized transformation, model-visible content,
  count-aware artifact conservation, tools, envelope, trust, provenance,
  order-only change, P0-L7 diff agreement, and P0-L10 re-analysis. The
  certificate is an internal transformation-fidelity proof only; it makes no
  performance, cache, causal, production-safety, or runtime claim.
- Added deterministic `CandidateExperimentPair` control/treatment manifest
  with no runtime results or telemetry. The source remains control and the
  candidate remains neutral treatment for a future experiment.
- Added P0-L2 metadata fingerprints to P0-L11 segment references where
  metadata is available, allowing P0-L13 to detect revision, origin,
  content-source, trust, lifecycle, and provenance drift without changing the
  neutral request contract. Layout fingerprints remain order/content identity
  and do not claim metadata or performance semantics.
- Added focused offline tests for valid reorder, stale source/evaluation,
  malformed artifact permutation, content and metadata drift, unsafe and
  unsupported candidates, planned/actual diff mismatch, P0-L10 mismatch,
  deterministic certificate/pair identity, and input immutability.
- Validation: `cargo fmt --all -- --check`, focused P0-L13 materialization
  tests (`9` passed), `cargo check --workspace --offline --locked`, strict
  workspace clippy (`--all-targets --all-features --offline --locked
  -- -D warnings`), full `cargo test --workspace --offline --locked`, and
  `git diff --check` all passed. The full workspace suite preserved the prior
  P0-L2 through P0-L12 results and passed the new materialization suite.
- No live inference, provider/runtime call, network access, candidate
  execution, automatic application, runtime telemetry, performance claim, or
  ContextBench integration occurred. P0-L6 remains environment-blocked because
  no existing usable `llama-server` or suitable GGUF model is available.

P0-L14, ContextBench integration, live inference, runtime probing, cache
execution, benchmark scoring, statistical methodology, and automatic request
application remain deferred.

## P0-L12 completion record

- Added the versioned, bounded `CandidateEvaluation` evidence gate over the
  existing P0-L7 `RequestDiff`, P0-L8 `CacheDiagnostic`, P0-L9 capability
  profile, and P0-L11 `LayoutCandidate` contracts. No foundational contract
  was duplicated or changed.
- Kept structural merit separate from empirical merit; added deterministic
  evidence states, capability gating, observation relevance rejection,
  structural hypothesis, claim permissions, bounded blockers, next actions,
  and separate design/environment/execution readiness.
- Preserved the documented/synthetic/experimentally-observed boundary,
  P0-L8's metric and causality semantics, unknown cache impact, and the
  environment-blocked P0-L6 state. No candidate was executed or applied.
- Added `docs/phase-0/CANDIDATE_EVALUATION.md` and focused deterministic tests
  covering the evidence ladder, capability and identity gates, mixed and
  contrary evidence, synthetic evidence, readiness separation, determinism,
  and input immutability.
- Validation: focused P0-L12 tests, P0-L11/P0-L10/P0-L9/P0-L8/P0-L7/P0-L5/
  P0-L4/P0-L2 tests, workspace check, formatting, clippy, full workspace
  tests, documentation review, and `git diff --check`.

## Completion record

- P0-L1 accepted the local-first product boundary in
  [`docs/phase-0/ADR-001-LOCAL-FIRST-PRODUCT-BOUNDARY.md`](../phase-0/ADR-001-LOCAL-FIRST-PRODUCT-BOUNDARY.md)
  and aligned the product definition in `docs/SOURCE_OF_TRUTH.md`.
- P0-L2 added versioned serde contracts in
  `crates/prefixity-core/src/observation.rs`: `ContextArtifact` v1,
  `CacheObservation` v1, and `RuntimeCacheCapabilities` v1.
- P0-L3 added focused validation tests and representative observation and
  capability fixtures. Local capability examples cover llama.cpp and Ollama;
  cloud/provider examples cover DeepSeek, Meta, Mistral, Alibaba Model Studio,
  and Z.AI / GLM. Unestablished capabilities remain unknown/unverified.
- Validation: focused observation-schema tests, workspace check, formatting,
  clippy, full workspace tests, documentation review, and `git diff --check`.

The contracts are observation-only. No runtime integration, cache probing,
automatic rewriting, cache routing, KV quantisation, benchmark scoring,
ContextBench integration, or P0-L5 work was included in that completion
record.

## P0-L4 completion record

- Extended the existing `prefixity-controlled-benchmark` crate with versioned
  provider-neutral `ConformanceExperiment`, `ConformanceCase`,
  `ConformanceRequest`, `ConformanceResult`, and `ConformanceRunner` types.
- Separated model-visible context from the request envelope and added
  deterministic ordered JSON-field handling and request/context fingerprints.
- Added baseline/repeat, content, structured-content, tool, model, reasoning,
  and response-format mutation classes without encoding cache outcomes.
- Added a deterministic in-process mock runner that records P0-L2
  `CacheObservation` values with explicit `not_observed` telemetry and no
  fabricated token, cache, latency, quality, or provider values.
- Added the synthetic coding-agent fixture at
  `fixtures/conformance/coding-agent-cache-conformance-v1.json` and focused
  validation/determinism tests.
- Validation included focused conformance tests, observation-schema tests,
  research-state consistency tests, workspace check, formatting, clippy,
  full workspace tests, and `git diff --check`.

P0-L5 runtime adapters, live inference, cache probing, ContextBench
integration, benchmark scoring, and optimization work remain deferred.

## P0-L5 completion record

- Added the llama.cpp-specific adapter inside the existing
  `prefixity-controlled-benchmark` crate: deterministic request projection,
  `LlamaCppTransport`, `FakeLlamaCppTransport`,
  `LlamaCppConformanceRunner`, and response normalization into P0-L2
  `CacheObservation`.
- Preserved context/artifact/tool order and represented envelope fields. A
  generic reasoning setting is explicitly rejected as not representable rather
  than silently omitted.
- Normalized native `timings` and compatibility `usage` fields separately;
  absent telemetry remains `not_observed`, explicit zero remains known, and
  malformed or conflicting values fail safely while raw values remain bounded
  and separate.
- Added synthetic llama-server protocol fixtures, a documented-only capability
  fixture, focused adapter tests, and the P0-L5 evidence-boundary document.
- Validation included llama.cpp adapter tests, P0-L4 conformance tests,
  P0-L2 observation tests, research-state consistency tests, workspace check,
  formatting, clippy, full workspace tests, and `git diff --check`.

P0-L6 is the planned first real local llama.cpp conformance/session-cache
experiment. It remains environment-blocked because no existing usable
`llama-server` binary or suitable GGUF model was available in the inspected
environment. It was not started, completed, or failed.

## P0-L7 completion record

- Added provider-neutral `PrefixDiff`, `EnvelopeDiff`, and combined
  `RequestDiff` diagnostics beside the existing P0-L4 conformance request
  model. Prefix Diff compares model-visible context; Envelope Diff compares
  model, reasoning, and response-format settings independently.
- Added deterministic first-divergence paths, structural/common-unit counts,
  byte common-prefix measurements for comparable text, ordered artifact/tool/
  schema-field change categories, whitespace-only classification, and bounded
  value summaries containing hashes and short previews rather than full values.
- Token-level common-prefix measurements remain `not_observed` because no
  tokenizer was introduced. Every P0-L7 cache-impact assessment remains
  `unknown`; no cache hit, miss, invalidation, or performance prediction is
  made.
- Added focused diff tests derived from the existing P0-L4 fixture, covering
  exact repeats, content and whitespace changes, artifact additions/removals/
  ordering, tool/schema changes, envelope-only changes, bounded diagnostics,
  malformed requests, determinism, and independent context/envelope results.
- Added [`docs/phase-0/PREFIX_DIFF.md`](../phase-0/PREFIX_DIFF.md) and updated
  the source-of-truth and documentation index.

## P0-L8 completion record

- Added versioned, reference-based `ObservationComparison` and
  `CacheDiagnostic` results. Raw observations and raw telemetry are not copied
  into diagnostics; identity dimensions, experiment/case references, request/
  context fingerprints, and evidence source classes remain auditable.
- Compared token accounting independently for transmitted input,
  provider-cached, fresh-prefill, reconstructed-context, and output tokens.
  Preserved known zero versus unknown/not-observed; unavailable metrics remain
  unavailable. Added bounded derived ratios with explicit denominators,
  relative timing changes, and RAM/VRAM/KV directional/resource deltas.
- Added directly/partially/incomparable/insufficient comparability, bounded
  reuse assessment, deterministic evidence statements, structural/observed
  association, and an explicit `not_established` causality status. No
  better/worse score, threshold, significance test, algebraic token equation,
  or universal performance score was introduced.
- Added deterministic synthetic fixtures/tests for exact repeats, partial and
  apparent reuse changes, fresh-prefill and timing-only changes, missing
  telemetry, explicit zero, identity incompatibility, mixed signals, and
  envelope-only changes. Runtime capabilities remain documentation only.
- Added [`docs/phase-0/CACHE_OBSERVATION_DIAGNOSTICS.md`](../phase-0/CACHE_OBSERVATION_DIAGNOSTICS.md)
  documenting the three-layer Phase-0 model and evidence boundary.
- Validation: focused P0-L8 tests, P0-L7/P0-L5/P0-L4/P0-L2 tests, research
  state consistency, workspace check, formatting, clippy, full workspace
  tests, and `git diff --check`.

P0-L9, ContextBench, runtime probing, live provider work, and later phases
were not started at the time of the P0-L8 completion record.

## P0-L9 completion record

- Added `CapabilityRegistry` over the existing P0-L2
  `RuntimeCacheCapabilities` contract; no second capability schema or P0-L2
  contract extension was required.
- Added explicit offline ingestion of the eight approved capability fixtures:
  llama.cpp neutral and documented profiles, Ollama, DeepSeek, Meta, Mistral,
  Alibaba Model Studio, and Z.AI / GLM. Fixture provenance remains separate
  from documented and experimentally observed capability evidence.
- Added deterministic semantic profile fingerprints, duplicate and malformed
  profile validation, typed identity/capability/evidence queries, generated
  evidence-preserving matrix cells and Markdown rendering, and research-gap
  counts that never relabel unknown as unsupported.
- Added focused registry tests for fixture loading, identity scope, evidence
  states, deterministic queries/matrices/serialization, validation failures,
  and gap reporting. Added
  [`docs/phase-0/CACHE_CAPABILITY_REGISTRY.md`](../phase-0/CACHE_CAPABILITY_REGISTRY.md)
  with an actual approved-fixture matrix example.
- P0-L6 remains environment-blocked; P0-L8 observations remain separate from
  registry profiles; ContextBench remains pending; no P0-L10 work started.

## P0-L10 completion record

- Added ContextStabilityAnalysis over the existing P0-L4 ConformanceRequest
  and optional P0-L2 ContextArtifact metadata. No P0-L2 or P0-L4 contract
  change was required.
- Added explicit-versus-derived classification sources, separate stability and
  lifecycle handling, independent trust references, bounded segment
  fingerprints/sizes/token metadata, deterministic adjacent boundaries, and
  stability inversion findings.
- Added a stability-aligned leading-region observation that stops stronger
  conclusions at unknown classifications or stability inversions. Unknown
  sizes remain unknown; token aggregation remains not observed.
- Preserved tool order and represented individual tool metadata where supplied;
  missing dynamic tool metadata remains unknown. No context reordering,
  canonicalization, pruning, cache prediction, or optimization action was
  added.
- Added focused synthetic tests derived from the existing P0-L4 request shape
  covering explicit metadata, lifecycle/trust separation, structural defaults,
  inversions, unknown middle segments, append-only context, tools, sizes,
  determinism, immutability, malformed metadata, and the existing conformance
  fixture.
- Added docs/phase-0/CONTEXT_STABILITY.md. P0-L6 remains environment-blocked;
  ContextBench remains pending; no P0-L11 work started.

## P0-L11 completion record

- Added a versioned ContextLayoutPlan over the existing P0-L4 request and
  P0-L10 stability analysis. No foundational request, observation, diff, or
  capability contract was changed.
- Added explicit constraint forms for precedence, fixed positions, preserved
  relative order, compatible movement regions, and unknown permission. Only
  artifact-sequence reorders are considered; system, user, and tool slots stay
  fixed.
- Added conservative trust-boundary checks, unknown-safety refusal, lifecycle
  preservation, deterministic adjacent/region-local candidate generation,
  minimal-change ordering, duplicate-layout suppression, and bounded rejection
  records.
- Re-analysed candidates through P0-L10 and attached P0-L7 request diffs. Cache
  impact remains unknown and no candidate is automatically applicable. No
  request rewriting, runtime evidence, cache prediction, token-savings claim,
  provider-specific planner, or P0-L12 work was added.
- Added focused offline tests and
  [`docs/phase-0/CONTEXT_LAYOUT_PLANNER.md`](../phase-0/CONTEXT_LAYOUT_PLANNER.md).
  P0-L6 remains environment-blocked and ContextBench remains pending.

## Historical task: Phase 1B.5 Corpus and Evaluation Strategy Review

Status: complete; Phase 1B.5 closed with `PASS WITH RECORDED LIMITATIONS`.

The Phase 1B.4 task definition and completion record are retained below as
historical evidence. No Phase 1B.6 or Phase 1C work has started.

## Current task

Determine the next evidence source and evaluation strategy after the frozen
CodeTraceBench Phase 1B.4 result. Do not implement an importer, corpus
adapter, planner change, controlled benchmark, or Phase 1C work as part of
this task.

## Phase 1B.5 completion record

Completed the public-source corpus and evaluation strategy review in
[`docs/phase-1/PHASE_1B5_CORPUS_EVALUATION_STRATEGY.md`](../phase-1/PHASE_1B5_CORPUS_EVALUATION_STRATEGY.md).

- Reviewed tau2-bench, ToolSandbox, AppWorld, BrowserGym, WebArena, AgentDojo,
  and SWE-bench using their public repositories, source schemas/documentation,
  and licence files where available. No raw third-party dataset was
  downloaded and no provider/model call was made.
- Preserved the Phase 1B.4 facts: CodeTraceBench remains useful for natural
  structural observation, provenance, usage, timestamps, and partial joins;
  its frozen planner result remains 719/719 `DO_NOTHING` because the required
  safety evidence is absent.
- Classified candidate evidence as `CAPTURED_EXPLICIT`,
  `DERIVED_STRUCTURAL`, `EVALUATION_ONLY`, `INFERRED_UNSAFE`, or `ABSENT`.
  No candidate was credited with optional, stale, dependency, required, or
  removable semantics based on adjacency, age, repetition, or evaluation
  labels.
- Recommended hybrid strategy `E`: retain CodeTraceBench for natural-workload
  observational validation and add a separate controlled intervention/quality
  track using paired/ablated, provider-neutral traces and an independent
  evaluation sidecar. AppWorld is the strongest public seed/design reference;
  its public/plain-text portion is Apache-2.0, while protected task/app/API
  material is distributed in encrypted bundles under Apache-2.0 with an
  additional encrypted-redistribution requirement. Any future Prefixity use
  must pin and audit the exact material used before implementation, and should
  not copy protected raw data into the repository. ToolSandbox is a useful
  schema reference but not the preferred raw corpus.
- Licence/privacy decision: do not copy candidate raw trajectories, encrypted
  bundles, archives, prompts, screenshots, or model output into the
  repository. The next artifact must use immutable source manifests and the
  existing hash-only boundary.
- Repository implementation remained unchanged. The accepted CodeTraceBench
  corpus and counts remain 24 trajectories, 719 request traces, and 1,498
  source events; no planner rules or evaluation joins were changed.
- Assessment: `PASS WITH RECORDED LIMITATIONS`. A useful next strategy exists,
  but a small controlled benchmark and its privacy/licence/data-leakage audit
  are still unresolved.

Recommended next task:

> **Phase 1B.6 - Controlled intervention benchmark design and seed audit.**
> Pin the public environment/task reference, define the provider-neutral trace
> and evaluation schemas, specify paired/ablated cases and leakage/privacy
> rules, and approve the seed set before implementing the benchmark or adapter.

Stop here. Do not begin the recommended next task, change the planner, begin
Phase 1C, commit, or push as part of this completion.

## Historical Phase 1B.4 task record

## Objective

Implement the smallest provenance-preserving evidence-adapter revision justified
by Phase 1B.3, then re-run the frozen Phase 1B planner over the accepted
CodeTraceBench slice.

Preserve only raw facts or deterministic structural relationships that Phase
1B.3 verified at the exact pinned corpus revision.

Do not weaken or tune the planner.

This task asks:

> Does preserving the richer evidence actually present in the raw trajectories
> improve Prefixity's observation/audit model, while retaining conservative
> decision behaviour where intervention-safety evidence remains absent?

## Required context

Read only the relevant sections of:

- `../phase-1/PHASE_1A_CORPUS_CLOSEOUT.md`
- `../phase-1/PHASE_1B_DECISION_CONTRACT.md`
- `../phase-1/PHASE_1B1_CHARACTERIZATION.md`
- `../phase-1/PHASE_1B2_EVIDENCE_GAP_STUDY.md`
- `../phase-1/PHASE_1B3_RAW_SCHEMA_VERIFICATION.md`
- `../phase-1/QUALITY_GATE.md`
- `../SOURCE_OF_TRUTH.md`

Inspect the current trace model, usage model, Phase 1A importer, provenance
structures, evaluation sidecar and Phase 1B planner before changing anything.

Do not recursively read unrelated documentation.

## Frozen evidence source

Use only:

- `NJU-LINK/CodeTraceBench`
- revision `aa213b84ffb6690fc37ca15766d6ca174ec36d4d`
- split `verified`
- the same accepted 24 trajectories / 719 requests

The Phase 1B.3 raw-schema findings are the authority for what upstream fields
were actually verified.

Do not broaden the corpus.

## Frozen planner

The Phase 1B.0 planner behaviour remains frozen.

Do not change:

- intervention eligibility;
- thresholds;
- reason-code semantics;
- dependency-safety rules;
- relocation rules;
- `DO_NOTHING`;
- compression behaviour.

If richer evidence does not create a justified intervention, `DO_NOTHING`
remains the correct result.

## Evidence that MAY be added

Phase 1B.3 verified the following useful evidence classes.

### Raw message timestamp

Raw messages contain numeric timestamps.

Preserve timestamps only with explicit provenance.

Do not infer:

- staleness;
- lifetime;
- invalidation;
- supersession;
- removability

from timestamp age.

### Provider response identity

Assistant response envelopes contain explicit provider response IDs and
response/model/status metadata.

Preserve these as provider/source identity where compatible with the existing
model.

Do not treat provider IDs as dependency or safety evidence.

### Provider usage

All 719 assistant response envelopes contain explicit provider usage telemetry.

Preserve the raw provider-specific usage without converting fields across
providers when semantics differ.

Reuse the existing Prefixity usage-schema/version mechanism where possible.

Keep:

- raw provider usage;
- schema/provider identity;
- normalized fields only where the existing versioned normalizer has an exact
  supported interpretation.

Do not:

- invent universal token counts;
- introduce current provider pricing;
- interpret usage as intervention safety;
- claim cache savings merely because cache-related provider fields exist.

### Evaluation source locators

Phase 1B.3 established an exact bounded mapping for 32 of 60 labelled
evaluation steps through explicit path/line source locators.

Preserve enough upstream structural identity to reproduce this mapping in the
evaluation sidecar.

Evaluation metadata must remain external to planner inputs.

For the remaining 28 labelled steps, preserve the absence of an exact mapping.

Do not infer joins from position or count similarity.

### Provenance

Every new captured field must state whether it is:

- `source_explicit`;
- `derived_structural`;
- `unknown`.

Reuse an existing compatible provenance representation if one exists.

Do not create an unnecessary parallel provenance system.

## Evidence that MUST remain unknown

Phase 1B.3 verified that the exact raw schema does not provide usable explicit:

- action/tool-call IDs;
- observation/result IDs;
- call-result references;
- dependency edges;
- semantic/load-bearing dependencies;
- `required`;
- `optional`;
- `stale`;
- invalidation;
- supersession.

Do not derive any of these from:

- timestamps;
- message order;
- adjacency;
- provider response IDs;
- evaluation labels;
- content;
- repetition;
- token counts;
- model outcome;
- later use.

Current false/empty schema defaults must not be described as source evidence.

## Schema/model design

Prefer the smallest backward-compatible change.

Before extending `RequestTrace` or `ContextBlock`, inspect whether the verified
facts fit existing:

- metadata;
- provenance;
- usage;
- source-map;
- trace identity

structures.

If a schema extension is required:

- version it explicitly;
- preserve compatibility with existing Phase 0/Phase 1 fixtures;
- distinguish absent/unknown from explicit false;
- do not force historical fixtures to fabricate provenance;
- document migration/compatibility semantics.

Avoid turning safety-sensitive evidence into bare booleans when provenance
would be lost.

## Importer revision

Revise the Phase 1A CodeTraceBench adapter only as needed to preserve the
verified evidence.

The adapter remains an evidence adapter.

It must not contain planner policy.

At minimum consider:

1. explicit raw timestamp preservation;
2. provider response ID/model/status preservation;
3. provider-specific usage capture;
4. typed provenance for newly captured/derived evidence;
5. explicit source locator preservation needed for partial evaluation joins.

Do not change source textual privacy behaviour.

Raw prompts/reasoning/tool output must remain untracked.

## Re-import

Regenerate the accepted derivative fixture deterministically from the same
24 pinned raw trajectories.

Preserve:

- corpus identity;
- exact revision;
- trajectory selection;
- privacy/hash-only boundary;
- label isolation.

Do not silently change the accepted workload selection.

Record whether request/source-event counts remain:

- 24 trajectories;
- 719 request traces;
- 1,498 source events.

Any count change must be explained before continuing.

## Frozen recharacterization

After the importer/evidence revision is complete and tests pass, run the frozen
Phase 1B planner over the regenerated 719 traces.

Use the Phase 1B.1 characterization schema unless an evidence-only additive
schema revision is strictly required.

If the report schema changes, version it.

Record:

- decision distribution;
- evidence distribution;
- provider-evidence coverage;
- usage-schema coverage;
- timestamp coverage;
- evaluation-locator coverage;
- safety audit;
- determinism;
- source integrity.

Do not tune the planner after seeing results.

## Expected interpretation

This task does NOT require positive interventions.

Because Phase 1B.3 found no explicit optional/stale/dependency/tool-link
evidence, it is entirely plausible that the planner remains dominated by
`DO_NOTHING`.

That is not a failure by itself.

The useful question is whether Prefixity now preserves materially better
provider/provenance/evaluation evidence without weakening safety.

## Evaluation overlay

Evaluation labels remain post-hoc only.

After planner output is frozen:

- reproduce the 32/60 exact labelled-step mappings where possible;
- report the 28 unmapped steps explicitly;
- do not pass solved/incorrect/unuseful labels to planner execution;
- do not convert evaluation failures into prune/defer recommendations.

No causal quality claim is authorized.

## Tests

Add focused tests covering at minimum:

- newly captured source-explicit evidence carries provenance;
- timestamp presence does not imply `stale`;
- provider response ID does not create dependencies;
- provider usage round-trips without semantic field conflation;
- unsupported provider fields remain raw/uninterpreted;
- historical fixtures remain compatible;
- absent safety metadata remains unknown rather than fabricated;
- evaluation locators remain outside planner inputs;
- exact evaluation mapping reproduces only where explicit source locators exist;
- no positional fallback is used for the remaining labels;
- importer remains deterministic;
- planner output remains deterministic;
- source traces are not mutated.

Run:

- formatting;
- workspace check;
- clippy;
- workspace tests;
- importer-specific tests;
- characterization checks;
- `git diff --check`.

No live provider/model calls are required.

## Required outputs

Produce:

- minimal evidence-adapter/model implementation;
- importer revision;
- focused tests;
- regenerated compact accepted evidence as appropriate;
- frozen recharacterization report;
- concise Phase 1B.4 findings document;
- completion record in this file.

Suggested findings document:

`docs/phase-1/PHASE_1B4_EVIDENCE_ADAPTER_RECHARACTERIZATION.md`

Do not commit raw trajectory archives/content.

## Decision gate

Answer explicitly:

1. Were all verified B.3 evidence fields preservable without weakening the
   privacy boundary?

2. Did the adapter remain deterministic?

3. Did the accepted corpus identity/counts remain stable?

4. Is captured versus derived evidence auditable?

5. How many requests now contain explicit provider usage?

6. Which provider usage schemas/fields were exactly interpretable?

7. How many messages contain explicit timestamps?

8. Did timestamp evidence alter any stale decision? It should not by itself.

9. Can the 32 exact evaluation joins be reproduced?

10. Are the remaining 28 still correctly unresolved?

11. Did any safety-sensitive field become known from legitimate source
    evidence?

12. What is the new Phase 1B decision distribution?

13. Did the hard safety audit remain clean?

14. Did deterministic repeated characterization match?

15. Does the richer evidence materially improve Prefixity's audit/evaluation
    capability even if intervention coverage remains zero?

16. Is another CodeTraceBench planner characterization justified, or has this
    corpus now exhausted its useful Phase 1B evidence?

## Assessment outcomes

Choose one:

### `PASS`

The narrow adapter revision truthfully preserves useful raw evidence,
recharacterization is deterministic/safety-clean, and the new evidence
materially improves Phase 1B evaluation or decision analysis.

### `PASS WITH RECORDED LIMITATIONS`

The evidence adapter improves provenance/provider/evaluation coverage and
remains safe, but intervention-relevant evidence remains substantially absent.

### `PIVOT`

The revision is technically sound but does not materially advance the central
Phase 1B decision hypothesis; recommend a separately reviewed corpus/evaluation
strategy change.

### `STOP`

The evidence revision introduces provenance ambiguity, privacy regression,
unsafe semantics, nondeterminism or another hard failure.

Do not choose the outcome based on intervention count alone.

## Stop conditions

Do not:

- tune or change the planner;
- fabricate optional/required/stale metadata;
- infer dependencies or tool links;
- infer staleness from timestamps;
- use evaluation outcomes as planner input;
- change the accepted trajectory selection;
- commit raw trajectory material;
- add current provider pricing;
- make live provider/model calls;
- begin Phase 1C;
- replay/mutate prompts;
- implement compression;
- start the next task;
- commit or push.

## Completion record

On completion record:

- model/schema changes;
- provenance design;
- importer changes;
- corpus/count integrity;
- provider usage coverage;
- timestamp coverage;
- evaluation-locator coverage;
- tests/checks;
- frozen recharacterization distribution;
- safety audit;
- determinism;
- decision-gate answers;
- Phase 1B.4 assessment;
- remaining limitations;
- recommended next task.

Do not begin the recommended next task.

## Phase 1B.4 completion record

Completed against the frozen source `NJU-LINK/CodeTraceBench`, revision
`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, split `verified`, and the existing
24-trajectory selection.

### Implementation

- Model/schema: retained trace format v2 compatibility and added optional
  evidence schema version 1. Added typed `EvidenceOrigin`, bounded
  `SourceLocator`, `EvidenceProvenance`, `ProviderFieldState`, and bounded
  `ProviderResponseMetadata`. `ContextBlock` now accepts a source-explicit
  numeric timestamp and block provenance; `RequestTrace` accepts evidence
  version, provider response metadata, and trace provenance. Historical
  fixtures remain readable without fabricating evidence.
- Provenance: every new captured/derived field is represented as
  `source_explicit`, `derived_structural`, or `unknown`, with bounded source
  locators. Generated message IDs, role-only source/zone projections, paths,
  and unique evaluation locator joins are marked derived. Provider response
  IDs are identity only, not dependency or tool relationships.
- Importer: corrected message classification to use role only; preserved raw
  numeric timestamps, response metadata, provider-specific raw usage, and
  hash-only source provenance; retained explicit evaluation locators and exact
  path/line-span joins. No content-marker, adjacency, timestamp-age,
  evaluation-label, or repetition inference creates safety evidence.
- Privacy: no raw prompts, reasoning, tool output, trajectory JSON, or archive
  was added to the repository. Decision-input traces remain hash-only.

### Evidence and recharacterization

- Corpus/count integrity: deterministic re-import produced 24 trajectories,
  719 request traces, 1,498 source events, and 724 derivative files. Two
  independent imports had identical file sets and SHA-256 hashes.
- Provider usage: 719/719 traces retain explicit raw usage. Schema counts are
  268 Anthropic custom, 118 DeepSeek custom, and 333 OpenAI Chat Completions.
  Existing exact normalization is claimed only for the OpenAI fields
  `prompt_tokens`, `prompt_tokens_details.cached_tokens`, `completion_tokens`,
  and `total_tokens`; unsupported Anthropic/DeepSeek fields remain raw.
- Provider response metadata: 719/719 traces preserve explicit response ID,
  model, `created`, and finish-reason metadata.
- Timestamp coverage: 1,498/1,498 source events preserve explicit numeric
  timestamps. Timestamp age was not used as staleness evidence and did not
  alter a planner decision.
- Evaluation join coverage: 60 labeled steps contain 63 explicit locator
  references; 32 steps have an exact bounded source-event join and 28 remain
  unresolved. No positional fallback was used. Labels were loaded only after
  both planner passes and were never planner inputs.
- Frozen planner distribution: 719/719 `DO_NOTHING`; `KEEP`, `DEFER`, `PRUNE`,
  `RELOCATE_CANDIDATE`, and `COMPRESS_CANDIDATE` each 0. This remains valid
  because no intervention-safety evidence was established.
- Safety audit: all failure counts are zero, including no source trace
  mutation, no destructive recommendation against current/required/protocol
  blocks, no missing/cyclic dependency-evidence violation, no unsafe
  relocation, no contradictory destructive recommendations, and no
  non-hypothetical recommendations.
- Determinism: both planner passes produced 719 plans, zero validation
  failures, and aggregate hash
  `5157f5a4a8b59d58d8898bf3df3fc4ad9bea60f08ccf9d920b87f41734e806fb`.
  Repeated characterization matched. No live provider/model calls were made.

### Checks and decision gate

Focused importer and characterization tests passed (`3` evidence-adapter
tests, `7` characterization tests, Python compilation). `cargo fmt --all
-- --check`, `cargo check --workspace`, `cargo clippy --workspace
--all-targets --all-features -- -D warnings`, and `cargo test --workspace`
passed (workspace test groups: 5, 59, 12, 2, 24, 12, 86, 22, plus zero-test
groups). The deterministic re-import produced identical hashes for both 724-
file outputs and preserved 24/719/1,498 counts; no raw material was present.
The final frozen characterization passed with 719 plans and zero validation
failures. Privacy/hash-only review found no raw archives or trajectory-content
files in the repository, and `git diff --check` passed.

Decision-gate answers: all verified B.3 fields were preservable without a
privacy regression; the adapter and planner were deterministic; corpus identity
and counts were stable; captured versus derived evidence is auditable; all 719
requests have provider usage; only the existing OpenAI schema fields listed
above are exactly interpretable; all 1,498 messages/events have timestamps;
timestamp evidence changed no stale decision; 32 exact evaluation joins are
reproducible and 28 remain unresolved; no safety-sensitive field became known;
the decision distribution is 719 `DO_NOTHING`; the hard safety audit is clean;
repeated characterization matches; audit/evaluation capability materially
improved even though intervention coverage remains zero; and another
CodeTraceBench planner characterization is not justified without new evidence.

Assessment: `PASS WITH RECORDED LIMITATIONS`. Remaining limitations are the
absence of explicit action/result/dependency/removability evidence, the 28
unresolved evaluation joins, lack of causal quality joins, and no replay,
savings, latency, or task-success evidence.

Recommended next task: separately review a corpus/evaluation artifact with
explicit action/result/dependency/removability identity and task-quality joins.
Do not begin that task or Phase 1C as part of this completion.

Findings: `docs/phase-1/PHASE_1B4_EVIDENCE_ADAPTER_RECHARACTERIZATION.md`.
