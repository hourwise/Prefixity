# Active Task — Phase 0 Foundation Slice (P0-L13)

Status: P0-L6C-R1 repair complete; P0-L6C Attempt 002 is preserved as
runtime-blocked / failed before inference, Phase 1C Attempt 003 is accepted as
invalid before readiness/inference, and Attempt 004 is prepared offline only;
Attempt
005 is an execution-invalidated partial live run caused by a missing generation
bound in the projected request, and Attempt 006 is the first complete bounded
live paired-mutation run. Attempt 006 produced bounded structural/cache
accounting observations, but P0-L8 causality remains not_established and
P0-L12 remains structural_only; no capability, performance, or causal claim is
supported. P0-L6C live execution is complete for this bounded run; no further
live attempt is authorized by this run.
P0-L1 through P0-L5 and P0-L7 through P0-L13 remain complete. P0-L6D is an
offline evidence-admission and experiment-adequacy audit; it does not authorize
another live attempt.

P0-L6E fresh-arm deconfounded preparation is now complete. It defines separate
control `A0/A1` and treatment `C0/C1` epochs, removes `B1` from that design,
adds explicit epoch/config/runtime identity gates, and permits each arm to be
finalized and persisted independently. This is offline preparation only; no
Attempt 007 was executed or authorized by this slice. A future live run still
requires a separately authorized operator-confirmed fresh listener for each
arm boundary.

The separately authorized Attempt 008 control and treatment epochs are now
complete and durably persisted. The permitted deterministic aggregation is
also persisted. No additional inference, B1 execution, or causal claim is
permitted from this bounded result.

Current P0-L6 status: `P0-L6 CLOSED — STRUCTURAL EVIDENCE ONLY — NEXT ACTION:
GATHER CAPABILITY EVIDENCE`. Attempt 008 contains four certified inference
requests total, exactly once each: A0, A1, C0, and C1. No B1 request was
issued.

The next capability-evidence design is the existing **Phase 1C — Quality-
gated controlled replay** programme, not a new P0-L14 phase. The design-only
addendum is being prepared on branch
`agent/phase-1c-capability-evidence-design`, rooted at the sealed P0-L6
commit `748e4673e8454d2ac3e27cefabee9259992038aa`, in
`docs/phase-1/PHASE_1C_CAPABILITY_EVIDENCE_DESIGN.md`. It makes task
capability primary, keeps token/cache/timing measurements secondary, reuses
the certified Phase 1C Stage 0 infrastructure, and leaves the external
evidence blocker and all Stage 1/Stage 2 execution authorization gates intact.
No capability inference, provider call, implementation change, structured
state work, or later phase is authorized by this design work.

Phase 1C design promotion is complete: accepted design commit
`36f960579e55c7ad48e54e4cb2670cc55cd1ef3e` was fast-forwarded from
`748e4673e8454d2ac3e27cefabee9259992038aa` to `main` and verified directly on
`origin/main`. The separate execution branch
`agent/phase-1c-capability-stage1-smoke` was created from that canonical main.
Offline preflight and full offline workspace validation passed, with no
network or credential reads. Stage 1 is blocked before readiness because the
accepted design and Stage 0 manifest leave provider, model, API surface,
endpoint, account/region, credential boundary, model settings, cache controls,
timeout, request/token/spend ceilings, and pricing unresolved; the existing
`prefixity-live` schema-smoke is a Phase 0B harness and does not select a
Phase 1C runtime. No readiness probe, provider call, or inference request has
been made; automatic retries remain zero. A separate exact runtime contract
and operator confirmation are required before any readiness check.

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

## P0-L6C Attempt 006 execution record

- Before runtime contact, the baseline was verified on `main` at
  `ef41dd45805fb2da17afe38b9c324f1866333e1e`, matching `origin/main`, with a
  clean tracked worktree, no Git lock, and the prior Attempt 001/002/005
  evidence preserved. CI run `#58` was successful. The operator supplied a
  fresh `model loaded` / `listening on http://127.0.0.1:8080` confirmation; the
  existing llama.cpp process was not started, stopped, or reconfigured by this
  task.
- The zero-network preflight passed with the unchanged semantic experiment ID
  `0c8d479c092359941747f640552077400bc61e88b56a6ebd70c9ccfec9dd4a11`, exact
  sequence `A0/A1/B1/C0/C1`, one A0 baseline, independent C0/C1 safety
  certificates, `generation_limit=1`, and
  `connect_timeout_ms=1000` / `request_timeout_ms=600000`. The frozen run
  identity is `f51be70e8c68f1648700e7822cca04d5c8623b155574286ff47a66e5d124079`,
  live identity is
  `6a2a7fd3cbe5a1469c85cca435394e4b2935af484763b2c40f2f1fb4072a74df`, and
  runtime configuration fingerprint is
  `70d4495d9fdd8e42632556df083541d66205bd7f696c6ae08579655a785f50f1`.
- After the permitted bounded TCP listener check passed, exactly five
  sequential transport requests were sent. Every projected request contained
  `max_tokens=1`; all five returned complete HTTP responses, normalized
  successfully, and completed. No retry, second run, unbounded generation, or
  additional request occurred. The bounded case accounting was:
  `A0=1172 transmitted / 0 cached / 1172 fresh / 1 output`,
  `A1=1172 / 656 / 516 / 1`, `B1=1191 / 0 / 1191 / 1`,
  `C0=1172 / 656 / 516 / 1`, and `C1=1172 / 656 / 516 / 1`.
  Prompt-processing timings were approximately 30980, 10643, 64478, 12906,
  and 19573 ms respectively. Time-to-first-token, wall duration, reconstructed
  context, and resource metrics were not observed.
- P0-L8 classified A0-to-A1 as `mixed_observations` with an observed reuse
  signal (`provider_cached_tokens` increased from 0 to 656 and fresh prefill
  decreased from 1172 to 516), C0-to-C1 as
  `no_observed_cache_reuse_change`, and A1-vs-C1 as
  `no_observed_cache_reuse_change`. The latter two pairs had unchanged
  `656 cached / 516 fresh` accounting; the C0 observation is retained as
  observed runtime behavior, not generalized cache evidence. All comparisons
  were directly comparable and all causality fields are
  `not_established`.
- P0-L12 evaluated the materialized A1-to-C1 candidate through the existing
  evaluator and returned `evidence_state=structural_only`. The observations
  were rejected for candidate/envelope mismatch or insufficient observation
  fields, capability evidence was not provided, and causal/performance claims
  remain disallowed. The bounded interpretation is
  `hypothesis=bounded_observed_result_requires_scoped_interpretation` and
  `causality=not_established`; this is not a capability or general cache
  conclusion.
- The complete bounded evidence is preserved under the ignored
  `experiments/runs/p0-l6c-first-live-paired-mutation-06/` directory. The
  historical Attempt 001, Attempt 002, and Attempt 005 artifacts remain
  unchanged. P0-L6C live execution is complete for this authorized run, with
  no Attempt 007, ContextBench integration, or P0-L14 work started.

## P0-L6D Attempt 006 evidence audit

- The audit baseline was `main` at
  `bd57147a85e7908c75678af8ee4daccf0256b98b`, matching `origin/main`, with a
  clean tracked worktree, no Git lock or concurrent Prefixity writer, all
  recorded Attempt 001-006 history present, and no Attempt 007 directory.
  The fifteen original Attempt 006 artifacts were enumerated and hashed before
  analysis. They remain byte-for-byte unchanged under the ignored run
  directory. The separately identified derived record is
  `docs/phase-0/evidence/p0-l6d-attempt-006-derived-reconciliation.json`.
- The original P0-L12 rejection was audited field-by-field. P0-L8 runtime,
  model, provider, protocol, runtime-version, identity-fingerprint,
  comparability, alignment, and observation accounting fields were directly
  comparable and passed. The candidate reference is exactly
  `layout-e7fc579745aeabf52c215ec124df66d5c5c4cf97e2af8336d63fabb0f6ead97b`,
  with source request fingerprint
  `a6945fde5281b924592b91179c1523cc17ec7dfd0c0b2ff2ad68922890d9aff3`,
  candidate request fingerprint
  `c176a81ed1de20882cdc402b72870f5ac6900b8df24dc0c1333ee3cefbf8d32b`, and
  RequestDiff fingerprint
  `b74a3d636a3cd73c1bcf69965e88b1d503331ff77ed1b5bcf7c0336dd84766a2`.
- The original A0-to-A1 diagnostic used request fingerprints
  `83867458364518c6193f25e2eaca4d56e168d05bf20b9922df731781e61b4168` to
  `a6945fde5281b924592b91179c1523cc17ec7dfd0c0b2ff2ad68922890d9aff3`; the
  original C0-to-C1 diagnostic used
  `51fdc731e55f13c05eb9607b244e5b9161da5afbfff3e9c0e8fe12b7c1d1632d` to
  `c176a81ed1de20882cdc402b72870f5ac6900b8df24dc0c1333ee3cefbf8d32b`.
  Both were correctly rejected as unrelated to the candidate mutation. Their
  envelopes were nevertheless semantically identical; the old evaluator's
  additional `envelope_compatibility_mismatch` was an incorrect mapping caused
  by comparing the complete RequestDiff, including request-pair fingerprints.
  The A1-to-C1 diagnostic is the semantically correct candidate relationship.
- The original diagnostics were labelled `synthetic_protocol_test` because
  the conformance-reference constructor hard-coded that source class. The
  live runner's provenance did not reach the diagnostic records. The bounded
  repair adds an explicit source-aware constructor and diagnostic function;
  the existing default remains synthetic. P0-L12's safety criteria were not
  weakened. The separate envelope comparison now ignores surrounding request
  fingerprints while still rejecting genuine schema, envelope-state, field,
  or cache-impact differences.
- The P0-L9 documented llama.cpp protocol profile was selected by an exact
  protocol query at its existing evidence strength:
  profile `02e24385723dc1df1c57d4a0bca0fd92d75bb57ac269429e1068b7349efa4e47`,
  `prefix_reuse=supported_documented`, model and runtime-version unknown, and
  documented protocol scope only. Attempt 006 did not promote capability to
  experimentally observed.
- Offline production projection reconstructed A0/A1/B1/C0/C1 exactly. Every
  projected request had `max_tokens=1`; normalized neutral request identities
  and persisted provider-body SHA-256 identities all matched the historical
  records. Serialized provider-body sizes were 8766, 8766, 8798, 8766, and
  8766 bytes. Byte common prefixes were A0/A1=5592, C0/C1=8442, and
  A1/C1=5582; the first difference was at each reported boundary. These are
  byte-level quantities only, not token-level estimates.
- The derived P0-L12 record admits only A1-to-C1 as relevant. A0-to-A1 and
  C0-to-C1 remain rejected for `candidate_mutation_mismatch`; the corrected
  diagnostic source is experimentally observed runtime, while causality stays
  `not_established`. With the documented capability profile, the natural
  derived state is `unsupported_by_current_evidence` because A1-to-C1 is
  `no_observed_cache_reuse_change`. Performance, causal, and automatic
  application claims remain disallowed. The original Attempt 006 state remains
  `structural_only`; the derived result is not a rewritten historical result.
- The experiment was structurally discriminating but runtime-accounting
  interpretation is insufficient/mixed for the stable-before-volatile
  hypothesis. The A0/A1, B1, C0, and C1 order was certified, but the one-slot
  sequence `A0/A1/B1/C0/C1`, C0 immediately preceding C1, and prior-state
  carryover make equal A1/C1 accounting insufficient to establish a causal
  layout effect. The partial-run durability gap remains deferred; it was not
  required for identity correctness in this audit. No Attempt 007, runtime
  contact, inference, ContextBench integration, or P0-L14 work began.

## P0-L6E fresh-arm deconfounded preparation

- Added an independent two-arm preparation path. The control arm is exactly
  `A0` → `A1`; the treatment arm is exactly `C0` → `C1`. `B1` is not included,
  because the fresh-server boundary replaces the in-sequence interference
  case for this design. The two arms use distinct caller-supplied epoch IDs
  and each asserts `fresh_server_for_arm=true`.
- Reused the existing P0-L7, P0-L10, P0-L11, P0-L12, and P0-L13 contracts.
  The candidate diagnostic is the certified A1-to-C1 relationship, while the
  within-arm C0-to-C1 comparison remains the treatment mutation diagnostic.
  Existing P0-L13 certificates and materialization pair identities are not
  regenerated or altered.
- Added exact Attempt 006 configuration enforcement: llama.cpp family and
  retained Q4_0 model contract, context 8192, one slot, metrics enabled,
  `connect_timeout_ms=1000`, `request_timeout_ms=600000`, and generation
  bound `max_tokens=1` on every projected request. Preflight is explicitly
  zero-network and records all four step IDs.
- Added arm-local run records with explicit state, epoch identity, request
  fingerprints, transport/response accounting, raw evidence, normalized
  result, failure, and provenance. A completed control record can be
  finalized and persisted before the operator confirms a fresh runtime for
  treatment. Partial records cannot carry complete normalized results.
- Added deterministic semantic identity covering the fresh-arm design,
  parent experiment, exact diffs, P0-L13 pair identity, runtime profile, and
  configuration fingerprint. Epoch IDs are deliberately excluded from the
  semantic identity. Offline aggregation requires both independently valid
  arms and constructs source-aware P0-L8 control, treatment, and A1-to-C1
  candidate diagnostics before P0-L12 evaluation.
- Added 14 focused offline fresh-arm tests. The suite confirms no composite
  score, `causality=not_established`, and disallowed performance/application
  claims; it does not promote capability beyond the existing documented
  profile. No listener check, localhost contact, inference, llama.cpp process
  control, evidence-directory mutation, Attempt 007, ContextBench, or P0-L14
  work occurred.

## P0-L6E Attempt 008 control epoch

- Authorization was limited to fresh control epoch
  `attempt-008-control-epoch-01`, with exactly two requests: `A0` then `A1`.
  Attempt 007 remains permanently closed as `INCONCLUSIVE / AMBIGUOUS` and was
  not retried, reconstructed, or aggregated.
- Pre-live gates passed: branch `main`; HEAD and local `origin/main` both
  `63653c7a45916f3ac9e0319b0b13b01bad227f30`; tracked worktree clean; no Git
  lock; no concurrent Prefixity/cargo/rustc experiment writer detected; the
  Attempt 008 directory initially contained only `prepared.json`; and the
  certified Attempt 008 inference count was `0`.
- The operator confirmed a fresh, unused llama.cpp server with model
  `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0` loaded, context 8192, one slot, and
  listening at `http://127.0.0.1:8080`. The single permitted non-inference
  TCP listener check passed and recorded zero HTTP/inference requests.
- The direct already-built executable performed an offline preflight with
  `network_calls=0`, then sent exactly A0 and A1 under the frozen
  `max_tokens=1`, `connect_timeout_ms=1000`, and
  `request_timeout_ms=600000` contract. No `cargo run`, live compilation,
  retry, warmup, calibration request, automatic restart, or treatment request
  occurred. Both responses were complete HTTP 200 responses and both cases
  normalized successfully. The control record final state is `normalized`.

Measured control telemetry:

| Case | Input/transmitted | Cached prompt | Fresh prefill | Output | Prompt time | Generation time | HTTP | Body bytes | Transport elapsed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| A0 | 1172 | 0 | 1172 | 1 | 60635 ms | 0 ms | 200 | 658 | 60720 ms |
| A1 | 1172 | 656 | 516 | 1 | 26129 ms | 0 ms | 200 | 660 | 26233 ms |

Native llama.cpp prompt timings were A0 `60635.306 ms` and A1
`26128.949 ms`; native predicted timings were `0.001 ms` with one predicted
token in each response. TTFT and wall-duration observations were not
observed. Raw response-body fingerprints and wire-request fingerprints remain
in the persisted control record.

- The A0 -> A1 P0-L8 diagnostic is aligned and classifies the result as
  `mixed_observations`, with association
  `structural_difference_with_observed_reuse_signal` and
  `causality=not_established`. The structural change is
  `artifact_content_changed`; cached prompt tokens increased from 0 to 656,
  fresh prefill decreased from 1172 to 516, transmitted input and output
  tokens were unchanged, and prompt-processing time decreased. Both
  observation references are classified as
  `experimentally_observed_runtime`. This is a bounded association only, not
  a causal, performance, capability, or application claim.
- The prepared metadata SHA-256 is unchanged:
  `19372E833090C1C3A7EDB38E338C377640CF44D69E206E379E475DB788C24317`.
  Persisted Attempt 008 evidence is:
  `experiments/runs/p0-l6-fresh-arm-attempt-008/prepared.json` (same SHA),
  `experiments/runs/p0-l6-fresh-arm-attempt-008/control-arm.json`
  (`F90B86A611EB99ADB692155BA53DB79C0E28AAA095AE682291169BF3E79029FA`),
  and `experiments/runs/p0-l6-fresh-arm-attempt-008/control-diagnostic.json`
  (`7E4FC6B23CF1CD93C59DF4A153B0E838A5A2610F2761B728644F029A2933E713`).
  The directory contains no treatment, C0, or C1 evidence.
- Candidate identity remains
  `e7fc579745aeabf52c215ec124df66d5c5c4cf97e2af8336d63fabb0f6ead97b`;
  pair identity remains
  `fed1a73f84a4a06bc7d6516b5ea47ddadb91da07ca06894a4729e5d5aa36ff1d`;
  safety certificate remains
  `7375cebd045e529e879ab4dbe50379e4d6f024dc4ca9b018c3833e6e3889a55d`.
  Semantic experiment ID remains
  `2c80b9273970af54289acd9b7d8a4e0cffc3d7e56596d71247b27d56377388a9`,
  parent ID remains
  `730c9785aee03483ba8d169e68d8c41a4788abce6c08a3ec83de73017fa539bd`, and
  runtime configuration fingerprint remains
  `43038980202a0d6054df881cd76130a9ebe49c23ada628153f1b03547ef8dea5`.
- Final repository verification: HEAD and local `origin/main` remain
  `63653c7a45916f3ac9e0319b0b13b01bad227f30` on `main`; tracked worktree is
  clean before this required task-record update; no Git lock is present; and
  `git diff --check` passes. Mandatory stop: do not execute
  `attempt-008-treatment-epoch-01` until separately authorized.

## P0-L6E Attempt 008 treatment epoch

- Separate authorization was received for exactly
  `attempt-008-treatment-epoch-01`: execute C0 followed by C1 on a fresh
  server. A0/A1 were not re-executed; B1 was absent and prohibited; Attempt
  007 was not recovered or reused; and no retry, warmup, calibration,
  diagnostic inference, additional probe, automatic restart, runtime/config
  substitution, commit, push, or later task was performed.
- The operator confirmed the prior control server was stopped and a fresh
  unchanged server was started with model
  `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, context 8192, one parallel slot, metrics
  enabled, and endpoint `http://127.0.0.1:8080`; the operator confirmed zero
  inference requests since startup. The one permitted non-inference TCP
  listener readiness check completed before dispatch and sent no HTTP request.
- The direct already-built executor passed offline preflight with
  `network_calls=0`, the frozen runtime configuration fingerprint, and the
  exact C0/C1 projection with `max_tokens=1`. It then issued exactly two
  transport requests in order, C0 then C1, and persisted the normalized
  treatment record before writing diagnostics or aggregation. No `cargo run`,
  live compilation, retry, timeout recovery, or extra live request occurred.

Measured treatment telemetry:

| Case | Request fingerprint | Input/transmitted | Cached prompt | Fresh prefill | Output | Normalized prompt | Native prompt | Generation | HTTP | Body bytes | Transport elapsed |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| C0 | `51fdc731e55f13c05eb9607b244e5b9161da5afbfff3e9c0e8fe12b7c1d1632d` | 1172 | 0 | 1172 | 1 | 53435 ms | 53435.469 ms | 0 ms | 200 | 658 | 53498 ms |
| C1 | `c176a81ed1de20882cdc402b72870f5ac6900b8df24dc0c1333ee3cefbf8d32b` | 1172 | 656 | 516 | 1 | 18580 ms | 18580.499 ms | 0 ms | 200 | 666 | 18774 ms |

- Native llama.cpp telemetry was C0 `prompt_n=1172`, `cache_n=0`,
  `predicted_n=1`, `predicted_ms=0.001`; and C1 `prompt_n=516`,
  `cache_n=656`, `predicted_n=1`, `predicted_ms=0.001`. TTFT and wall-duration
  observations were not observed. C0 wire-request fingerprint is
  `78915eb8e77e1c96def07b38f75a449a3bd7332be0d77d6c7414880ac43ca11e` and raw
  response-body fingerprint is
  `36f4e81838b3ae3877ea3fcf18e21bf4a9291ea3fcccdcbcbd8ab9b87516e845`; C1
  wire-request fingerprint is
  `4ed5e162e155567764c531fac1b1dee5f7023c578cbf8de2485791f61595c50e` and
  raw response-body fingerprint is
  `ba92400d69ed529da29162505195d53cc5cd05dec74eb0e19bcdc6e2e553a722`.
- The treatment record is `state=normalized`, with
  `expected_steps=2`, `completed_steps=2`, `transport_attempts=2`,
  `complete_http_responses=2`, and `normalized_cases=2`. C0 and C1 both
  carried one completion token, with total usage 1173 tokens. The result
  preserves semantic experiment ID
  `2c80b9273970af54289acd9b7d8a4e0cffc3d7e56596d71247b27d56377388a9`, parent
  ID `730c9785aee03483ba8d169e68d8c41a4788abce6c08a3ec83de73017fa539bd`, and
  runtime configuration fingerprint
  `43038980202a0d6054df881cd76130a9ebe49c23ada628153f1b03547ef8dea5`.

Treatment interpretation and permitted offline derivations:

- The C0 -> C1 P0-L8 treatment diagnostic is aligned and classifies the
  result as `mixed_observations`, with association
  `structural_difference_with_observed_reuse_signal` and
  `causality=not_established`. The structural category is
  `artifact_content_changed`; cached prompt tokens increased from 0 to 656,
  fresh prefill decreased from 1172 to 516, transmitted input and output
  tokens were unchanged, and prompt-processing time decreased. Both
  observation references are `experimentally_observed_runtime`. This is a
  bounded association, not a causal, performance, capability, or application
  claim.
- The frozen A1 -> C1 candidate diagnostic is aligned and classifies the
  result as `no_observed_cache_reuse_change`, with association
  `structural_difference_with_observed_metric_change` and
  `causality=not_established`. Its structural category is
  `artifact_order_changed`; transmitted input, cached prompt, fresh prefill,
  and output tokens were unchanged, while prompt-processing time decreased.
  Both observation references are `experimentally_observed_runtime`.
- The explicitly permitted P0-L12 aggregation was deterministic and
  non-inference over persisted A0/A1/C0/C1 only. It reports control mutation
  `mixed_observations`, treatment mutation `mixed_observations`, candidate
  comparison `no_observed_cache_reuse_change`, and candidate evaluation
  `evidence_state=structural_only`, `next_action=gather_capability_evidence`.
  The evaluator retained blockers `runtime_capability_pending`,
  `observation_identity_mismatch`, `no_experimental_observation`, and
  `insufficient_observation_fields`; no causal, performance, capability, or
  application permission was upgraded.
- The seven runtime evidence files remain intentionally ignored under the
  repository's existing `experiments/runs/` local-artifact policy. They are
  preserved locally by their sealed hashes; none was force-added to Git.

Durable treatment evidence and hashes:

- `experiments/runs/p0-l6-fresh-arm-attempt-008/treatment-arm.json`:
  `7DBD5DE87E4847580195EC52CFA236C9893E654F69FD49F63920A64D9BD0E0BB`.
- `experiments/runs/p0-l6-fresh-arm-attempt-008/treatment-diagnostic.json`:
  `CFF57EBF889F51BA7A54D66B726C8E3684AB2C70B001EBEF3E91F39A8CF6CEE7`.
- `experiments/runs/p0-l6-fresh-arm-attempt-008/candidate-diagnostic.json`:
  `5AA4B923A10402CC2CF923C75234D758286A9DA32C8FFFAE9A994A34F06ED7A5`.
- `experiments/runs/p0-l6-fresh-arm-attempt-008/aggregation.json`:
  `75C50BBB712BCFD1B0A61C75BD20C98B9006501C6133F5F8D3089312369C1504`.
- Accepted control evidence remained byte-for-byte unchanged:
  `prepared.json` `19372E833090C1C3A7EDB38E338C377640CF44D69E206E379E475DB788C24317`,
  `control-arm.json` `F90B86A611EB99ADB692155BA53DB79C0E28AAA095AE682291169BF3E79029FA`,
  and `control-diagnostic.json`
  `7E4FC6B23CF1CD93C59DF4A153B0E838A5A2610F2761B728644F029A2933E713`.
- The candidate identity remains
  `e7fc579745aeabf52c215ec124df66d5c5c4cf97e2af8336d63fabb0f6ead97b`, pair
  identity remains
  `fed1a73f84a4a06bc7d6516b5ea47ddadb91da07ca06894a4729e5d5aa36ff1d`, and
  safety certificate remains
  `7375cebd045e529e879ab4dbe50379e4d6f024dc4ca9b018c3833e6e3889a55d`.

- Final validation: the focused fresh-arm regression passed all 14 tests;
  `cargo fmt --all -- --check` passed; the pre-treatment `ACTIVE.md` hash
  `D874B2161345E5BAC21900B161D8DAEDC40918A23EDFC39F43AEAD79069CFB85` was
  unchanged throughout the live epoch; and the treatment executor source,
  build artifact, and temporary executable were removed after use. Mandatory
  stop: no more Attempt 008 requests, no B1, no Attempt 007 recovery, no
  ContextBench/P0-L14 work, and no Attempt 009 may begin from this result.

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

## Phase 1C Stage 1 local Qwen preparation record

The explicitly authorized local runtime contract is now tracked at
`docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT.json`, with its
canonical SHA-256 persisted in the adjacent `.sha256` file. The dedicated,
non-scored fixture is
`phase1c-stage1-schema-smoke-01`; it is not one of `h001` through `h012` and
contains no evaluator-answer material. Request and output schemas are tracked
under `docs/phase-1/schemas/`.

An isolated llama.cpp OpenAI-compatible Chat Completions adapter was added to
`prefixity-controlled-benchmark`; `prefixity-live` and production Prefixity
provider, candidate, arm, and planner behavior were not changed. The adapter
freezes model `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, Q4_0, context 8192, one slot,
metrics enabled, temperature 0, top-p 1, max-tokens 256, stream false, seed 1,
one request maximum, zero retries/fallbacks/replicates, and the authorized
loopback endpoint. It performs at most one non-inference TCP listener check and
will not perform it until fresh-runtime confirmation is explicitly supplied.

Offline evidence: the real-file preflight passed with contract SHA
`6d32375c24d51b98d82a8873d6e0f03f1d8c9b59799b843cd3a3d69c05b097b3`, fixture
SHA `98b6141de86b56c0e8f2f75633e67c88c4c26f344a020a347c004b704cd68584`,
request-schema SHA `23d7c3202b12d5f5517f2f0e0156bfa1523a0ecd363cddc7835dc42716acceca`,
output-schema SHA `d8abcc102303dc8e19b80d9647ece9103909a88accf620dd0efdc04e1b7bf51b`,
request-projection SHA `ec8ffc0d3ffba782c48f0d49f12a43a357098f7a2f6bdda7bc6146c00a2b6c2d`,
and wire-request SHA
`ec97953bbb9e9115d9ae63885092fbc0d606daf192dcaafc47c069d45900592d`.
Estimated input size is 123 tokens against the 4096-token bound. Package tests
(40 library tests and all controlled-benchmark integration groups) and strict
offline clippy passed; formatting and diff checks passed. This preparation
performed zero network calls, zero readiness checks, and zero inference
requests. It does not authorize or claim a schema-smoke result.

The execution branch remains
`agent/phase-1c-capability-stage1-smoke`, separate from canonical `main`.
Execution is pending the operator's explicit current confirmation that the
fresh server has the exact model loaded, context 8192, one parallel slot,
metrics enabled, endpoint base `http://127.0.0.1:8080`, and zero inference
requests since startup. After that confirmation, the next execution turn may
perform exactly one listener check and exactly one non-scored schema-smoke
request, with no retry.

## Phase 1C Stage 1 local Qwen execution record

The operator supplied the explicit fresh-runtime confirmation for the exact
local llama.cpp selection: model `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, context
8192, one parallel slot, metrics enabled, loopback base
`http://127.0.0.1:8080`, and zero requests since startup. The isolated runner
performed exactly one non-inference TCP listener check; it passed in 6 ms.

The dedicated fixture `phase1c-stage1-schema-smoke-01` then received exactly
one HTTP Chat Completions request. The request used the frozen projection
fingerprint `ec97953bbb9e9115d9ae63885092fbc0d606daf192dcaafc47c069d45900592d`,
with one transport attempt, one inference request, and zero automatic retries,
fallbacks, or replicate requests. No scored task was consumed and no second
request was issued.

The server returned HTTP 200 with a complete 1,909-byte response. Its response
body SHA-256 is
`223006f5bed5fec81dae02097e3e723d8c8d3a5997c8ebb116841e95914eb048`.
Recorded usage is `prompt_tokens=79`, `completion_tokens=256`,
`total_tokens=335`, with `cached_tokens=0`; recorded timings remain raw
llama.cpp telemetry. The exact extractor found an empty
`choices[0].message.content`, so no semantic JSON payload could be parsed or
normalized. The response also reported `finish_reason=length`; this is an
execution observation only and does not establish a capability or performance
claim.

Final Stage 1 state is `FAILED` for the schema pipeline, not a pass and not a
capability result. Evidence is preserved under the ignored directory
`experiments/runs/phase1c-stage1-schema-smoke-01/`, including the preflight,
readiness, request, raw response body, and final result record. The raw body
was not repaired, the empty assistant content was not substituted, and no
retry or additional inference is authorized from this result. Phase 1C
inference accounting is now one request for this non-scored smoke; the scored
capability cohort remains at zero requests. P0-L6 evidence and production
provider behavior remain unchanged.

## Phase 1C Stage 1 failed-smoke forensic review

The first authorized Stage 1 local-Qwen schema smoke is confirmed as a valid
bounded failure and is now reviewed offline in
`docs/phase-1/PHASE_1C_STAGE1_FAILED_SMOKE_FORENSIC_REVIEW.md`. The preserved
HTTP 200 response contains a non-empty `choices[0].message.reasoning_content`
field (1,226 UTF-8 bytes; exact field SHA recorded in the review) while
`choices[0].message.content` is an empty string. It reports
`completion_tokens=256`, `timings.predicted_n=256`, and `finish_reason=length`.
The primary classification is `THINKING_BUDGET_EXHAUSTION`, established at the
response level with high confidence; the content-only Stage 1 contract is also
incompatible with the observed reasoning-bearing response shape. The adapter
behaved correctly under its registered contract and did not repair or fall
back to reasoning text.

No live action occurred during the review: new inference requests `0`, review
network calls `0`, retries `0`, scored capability requests `0`, and Prefixity
behavioral changes `0`. The raw evidence, runtime contract, and adapter remain
unchanged. Remediation options are documented for separate authorization only.

## Phase 1C Stage 1 reasoning-off remediation preparation

The accepted Stage 1 preparation, failed smoke, and offline forensic review
lineage was promoted to `main` by fast-forward. Canonical `main` and
`origin/main` both resolve to `74f4db764d398711006fd20017e186febeb3ee14`.
The remediation branch was created from that promoted tip as
`agent/phase-1c-stage1-reasoning-off-smoke`; it is intentionally not merged
back to `main`.

Smoke 01 remains permanently consumed and frozen. Its preserved response
SHA-256 remains
`223006f5bed5fec81dae02097e3e723d8c8d3a5997c8ebb116841e95914eb048`, and the
accepted forensic-review SHA-256 remains
`f9ef96b36ba42da544adaeccafc9e38830e1efa9d7e456d4214c7df3e9a8705c`.

The new offline-only V2 runtime contract is
`docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT_V2.json`, with
canonical SHA-256
`b57462a22c91649e9e629b22d65214cdea7a47a38570506256c0b945467fe882`.
It retains the V1 model, Q4_0 quantization, llama.cpp engine, context `8192`,
parallelism `1`, metrics, loopback endpoint, generation settings, request and
output contracts, retry ceiling `0`, and request ceiling `1`. Its only runtime
behavioral delta is the explicit llama.cpp setting `reasoning=off`, selected
to address the reviewed `THINKING_BUDGET_EXHAUSTION` failure while preserving
the content-only response contract. The V2 sidecar records the same
canonical fingerprint.

The distinct non-scored Smoke 02 identity is
`phase-1c-stage1-schema-smoke-02`, with fixture
`fixtures/phase1c/phase1c-stage1-schema-smoke-02.json`. Its semantic request
content and generation parameters are unchanged from Smoke 01; its request
projection and wire-request SHA-256 are both
`49cbcae3c82924542434d5e8f8f92994096d9ce032b99e475a89cdcdc8805b94`.
The estimated input size is `122` tokens against the `4096` bound. The fixture
has no scored task IDs, and its planned evidence location is
`experiments/runs/phase1c-stage1-schema-smoke-02`.

The checked-in Rust helper
`phase1c_stage1_reasoning_off_preflight` performs only bounded offline
contract, fixture, lineage, fingerprint, and evidence-integrity validation.
The existing V1 live adapter was not changed; no production behavior was
changed. The preflight returned `PREPARED` with `network_calls=0` and
`inference_requests=0`, verified that the Smoke 02 evidence location is
absent, and revalidated the frozen Smoke 01 and forensic-review hashes.

Validation completed offline: package tests passed (`42` library tests plus
all controlled-benchmark integration groups), formatting passed, and the
reasoning-off preflight binary passed. No server was started, no listener
check was performed, and no inference request was issued during this
preparation. The scored capability cohort remains at zero requests, and the
scored reasoning decision remains undecided.

This branch is ready only for a separately authorized Smoke 02 execution
gate. Before any live action, the operator must freshly confirm the exact
model `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, Q4_0 quantization, context `8192`,
parallel slots `1`, metrics enabled, `reasoning=off`, endpoint base
`http://127.0.0.1:8080`, and zero inference requests since startup. A future
authorized execution may then perform exactly one permitted listener check
and exactly one Smoke 02 request, with no retry or additional inference.

## Phase 1C Stage 1 Smoke 02 reasoning-off execution record

The operator supplied fresh current confirmation for Smoke 02: model
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, Q4_0 quantization, context `8192`, one
parallel slot, metrics enabled, `reasoning=off`, endpoint base
`http://127.0.0.1:8080`, a server freshly started specifically for Smoke 02,
zero inference requests since startup, and no manual probe, warmup, completion,
chat, browser, or other model request since startup.

The dedicated reasoning-off runner performed exactly one non-inference TCP
listener check. It passed in `2` ms, and readiness recorded
`listener_check_attempts=1`, `network_calls=1`, and
`inference_requests=0`. It then issued exactly one registered Smoke 02 HTTP
request with one transport attempt and zero retries, fallbacks, or replicates.

The request used the canonical projection and wire-request SHA-256
`49cbcae3c82924542434d5e8f8f92994096d9ce032b99e475a89cdcdc8805b94`, with
temperature `0`, top-p `1`, max tokens `256`, stream `false`, seed `1`, and
estimated input size `122` tokens. No scored task was included or consumed.

The request completed with HTTP `200` and a complete `754`-byte response in
`4274` ms transport time. The raw response body is preserved at
`experiments/runs/phase1c-stage1-schema-smoke-02/response-body.bin` with
SHA-256
`20514424872eaee321405697287816fcb02d16f5a4be5e1f51aa5ee3c860b978`.
The response reported `finish_reason=stop`,
`prompt_tokens=81`, `completion_tokens=29`, `total_tokens=110`, and
`cached_tokens=0`. Raw llama.cpp timing telemetry is preserved: `prompt_n=81`,
`prompt_ms=1218.0`, `predicted_n=29`, `predicted_ms=3017.873`, and
`cache_n=0`, together with the other reported timing fields. No telemetry was
fabricated or inferred.

`choices[0].message.content` was non-empty at `94` bytes and `94` Unicode
characters. No `reasoning_content` field was present. The content parsed as
the exact required JSON and normalized to:

```json
{
  "schema_version": "phase1c-stage1-smoke-v1",
  "status": "ok",
  "marker": "PREFIXITY_PHASE1C_STAGE1"
}
```

The evidence record reports successful JSON parsing, exact semantic payload
validation, schema normalization, and no scored-task consumption. The offline
post-run validator returned `VALIDATED`, rechecked the V2 contract and fixture
fingerprints, request identity, response-body hash, raw response fields,
usage, timings, finish reason, and single-request accounting. No localhost or
model contact occurred during post-run validation.

Final Smoke 02 state is `PASSED`. Allowed conclusion:
`PHASE 1C STAGE 1 LOCAL QWEN SCHEMA SMOKE 02 PASSED`. This establishes only
that the registered reasoning-off local-Qwen runtime traversed the Stage 1
schema evidence pipeline successfully; it is not a scored capability or
performance result.

Phase 1C live inference accounting is now exactly `2`: Smoke 01 `1` failed
request plus Smoke 02 `1` passed request. Smoke 02 requests are exactly `1`,
automatic retries are `0`, BASELINE/NO_OP/INTERVENTION scored requests are
`0`, scored replicates are `0`, ContextBench work is `0`, and the scored
capability programme remains separately gated. Smoke 01 evidence modification
is `0`, P0-L6 evidence modification is `0`, and no production Prefixity
behavior changed. No further localhost/model contact or scored work is
authorized under this record.

## Phase 1C scored-runtime decision and pilot design record

Stage 1 Smoke 02 was sealed and promoted offline before this design work. The
exact verified SHA-256 for
`experiments/runs/phase1c-stage1-schema-smoke-02/readiness.json` is
`43227aba1bc695f3a7f13dba6203ce67c7f37b90be63fc6b932eb53d229072e9`.
The prior embedded space was transcription-only; the underlying preserved
readiness artifact matched the exact 64-character hash. The other preserved
Smoke 02 artifact hashes are: preflight
`1eb0d8deb21ba317a3da3bfab02495282cb70448c1dbf8b17bf55a6b4afd8c69`, request
`49cbcae3c82924542434d5e8f8f92994096d9ce032b99e475a89cdcdc8805b94`, response
`20514424872eaee321405697287816fcb02d16f5a4be5e1f51aa5ee3c860b978`, and
stage1-result
`9049869b3f2ab6e4f49fe801d97dd07cfcdc94483c6984a472ae84636107592c`.
The accepted execution was fast-forwarded to `main`, pushed, and directly
verified at `origin/main`:
`fea5b829a7c80bd3ad5feb23318309fecaf7a3ad`. No evidence was rewritten.

The scored reasoning decision is **Option C: explicit `reasoning=on` with
reasoning-aware evidence capture**. Option A (`reasoning=off`) is useful for
bounded schema plumbing but could suppress capability. Option B
(enabled/auto) preserves reasoning but leaves the runtime behavior less
explicit and harder to match. Option C fixes the enabled setting identically
for BASELINE, NO_OP, INTERVENTION, and all replicates, while retaining
`reasoning_content` in a separate diagnostic field. Only terminal
`choices[0].message.content` and declared task/tool outcomes are scored.
Reasoning is never fed to a later turn, arm, treatment, evaluator, task
identity, or hidden answer label. This is a capability-fairness decision, not
an inference from Smoke 02's schema-only pass.

The scored output ceiling is `2048` tokens. This is an offline design
estimate, not a measurement: it is eight times the `256`-token Smoke 01
ceiling, leaves a declared `512`-token final-answer allowance within the
estimate, and reserves the remainder for enabled reasoning. A response that
ends at the ceiling without terminal final content is `INCONCLUSIVE`; there
is no retry or post-outcome ceiling increase. The exact new contract is
`docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V1.json`, with canonical
SHA-256
`2f05440eb66d195d5362b9ad3a5dfc1708c4b8902cf8cd5774d26076c9aa1520` and
contract-schema canonical SHA-256
`ad42524c6fb2f75056391754cfedd3d84658d797199429f0b68827fdb33d80fb`.
It fixes local `llama.cpp`/`llama-server`, model
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, Q4_0, context `8192`, one slot, metrics,
endpoint `http://127.0.0.1:8080/v1/chat/completions`, temperature `0`, top-p
`1`, seed `1`, stream `false`, connect timeout `1000` ms, request timeout
`600000` ms, supervisor timeout `660000` ms, and zero retries/fallbacks/
adaptive replicates. Each arm/replicate requires a fresh server process with
zero prior inference and no warmup; local telemetry does not support a
provider-cache or billed-cost claim by itself.

The first scored slice is preregistered in
`docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V1.json`, canonical SHA-256
`201c21d7472dcf274bb912488f9334ac264bbba4968262e4e9fa4939a2597552`.
Its fixed six cases are `h001`, `h004`, `h006`, `h007`, `h009`, and `h010`;
the fixed arm order is BASELINE -> NO_OP -> INTERVENTION, with replicate `1`
only and at most three model turns per arm/replicate. The maximum is
`6 × 3 × 1 × 3 = 54` model inference requests. Four positive frozen
intervention paths and two dependency/protocol no-op controls make this the
smallest useful process-and-quality slice; it does not authorize scaling to
the planned twelve-case, two-replicate, 216-request cohort. Any hard safety
failure, baseline-pass to intervention-fail, identity/contract/hash drift,
reasoning leakage, missing accounting/content/tool evidence, retry, redirect,
fallback, warmup, undeclared diff, or post-outcome denominator change
invalidates or makes the affected result inconclusive as specified in the
manifest.

Added isolated experiment-only Rust validation at
`crates/prefixity-controlled-benchmark/src/phase1c_scored_design.rs` and its
`prefixity-phase1c-scored-design` binary. It parses only the checked-in
contract, schema, manifest, and fingerprint sidecars; it opens no socket,
starts no runtime, reads no credentials, and reads no live evidence. The
recovered official Rustup stable toolchain is `cargo 1.97.1` and `rustc
1.97.1`, with `rustfmt` and `clippy` installed; the repository expectation is
stable Rust with MSRV `1.86`.

The focused controlled-benchmark package tests passed (`45` library tests and
all package integration suites), the checked-in scored-design validator
returned `VALIDATED_OFFLINE` with network calls, credential reads, and
inference requests all `0`, 18 fixed arm boundaries, pilot ceiling `54`, and
full-cohort ceiling `216`. Strict workspace Clippy with `-D warnings`,
`cargo fmt --all -- --check`, and full `cargo test --workspace --offline
--locked` all passed. The formatter required a minimal experiment-validator-
only source ordering/wrapping fix; no contract, manifest, runtime, scoring,
or Prefixity behavior changed. No P0-L6 evidence, ContextBench material, or
live evidence was changed.

## Phase 1C scored pilot h001 pre-live seal

The scored-runtime design passed all authorized offline validation after
recovery of the existing official Rustup installation. The validator-only
formatting successor commit
`46289a6c5f16bca0415ac7cd1419dfcbcf9e8f2e` was pushed on
`agent/phase-1c-scored-runtime-design`, then fast-forwarded from the prior
`main` commit `fea5b829a7c80bd3ad5feb23318309fecaf7a3ad`. The promoted commit
was pushed and directly verified at `origin/main` with the same exact SHA.
No contract, schema, manifest, prior smoke evidence, or production behavior
was changed during promotion.

The pre-live branch is now
`agent/phase-1c-scored-pilot-h001`, created from the promoted `main` commit.
The next and only authorized live slice is `h001`, in fixed order
BASELINE -> NO_OP -> INTERVENTION, replicate `1`, with at most three model
turns per arm and at most nine total inference requests. It must use the
sealed scored contract: explicit `reasoning=on`, the fixed local Qwen runtime,
fresh server process per arm, zero inference since startup, no warmup, and no
retry. The slice must stop after `h001`; no other case, arm, replicate, or
follow-up request is authorized by this preparation record.

This branch is prepared only; no llama.cpp server was started, no listener
check was performed, and no live or localhost request was issued during
promotion or preparation. Phase 1C live inference accounting remains exactly
`2` historical schema-smoke requests, scored capability requests remain `0`,
automatic retries remain `0`, and P0-L6 evidence modification remains `0`.

## Phase 1C scored pilot h001 offline task materialization

The preregistered h001 scored capability slice is now materialized as a
checked-in, experiment-only task package on
`agent/phase-1c-scored-pilot-h001`. This is an offline preparation record only:
the Stage 0 mock payloads were not promoted as model results, no ContextBench
material was admitted, and the reviewed h001 structural intent was converted
into a separate model-visible task artifact plus separate evaluator-only state.

The frozen h001 task tests explicit dependency retention under duplicate
content. Model-visible records contain message events `e001` and `e002` with
the same content hash, plus action `a003` whose explicit
`reference_event_ids` value is `[e001]`; the intervention removes only `e002`.
The evaluator-only expected terminal object is exactly:

```json
{
  "action_id": "a003",
  "required_event_id": "e001",
  "required_content_hash": "16fb8d61491c85785ed6c26ba9f8ddb10596e45b5cc073d2eec140ccb8bee075"
}
```

The evaluator-only relation/state material records the `e003 -> e001`
explicit dependency and required context source IDs. It is never included in
the model-visible prompt. The tool contract is a no-tool contract with one
terminal JSON answer; `reasoning_content`, if returned by the runtime, is
retained only as a diagnostic and is never scored or appended to another turn
or arm.

The checked-in artifacts and canonical SHA-256 identities are:

- task artifact
  `docs/phase-1/PHASE_1C_H001_SCORED_TASK_V1.json` —
  `4597b25cac114899d7e52ff67e7fb2297442c954cb10bffba2aab94cc20ef7c8`;
- source manifest
  `docs/phase-1/PHASE_1C_H001_SCORED_SOURCE_MANIFEST_V1.json` —
  `da397d5ebaa4bbbc48f7f84cb1a0c3f6480d20c7cadfad96231b596b5074cc4a`;
- required state
  `docs/phase-1/PHASE_1C_H001_REQUIRED_STATE_V1.json` —
  `459f5e093e540e146179399b3263e168642b2044578db59239badc9bfb5e0b8a`;
- tool contract
  `docs/phase-1/PHASE_1C_H001_TOOL_CONTRACT_V1.json` —
  `1105e16039acc4d9f1db6d784f6a5eb0653d4601117aeefcc8686338df7552a9`;
- evaluator
  `docs/phase-1/PHASE_1C_H001_EVALUATOR_V1.json` —
  `5fc19f47bd36e8625535e63939caebbb0e9ca1a458bb41b357de9d3f6d1f7fe3`;
- arm materialization
  `docs/phase-1/PHASE_1C_H001_ARM_PROJECTIONS_V1.json` —
  `1e09aaf51d1b93bddadb54e198b987481490083405a0de8d480b18393cb81416`;
- h001 task manifest
  `docs/phase-1/PHASE_1C_H001_SCORED_TASK_MANIFEST_V1.json` —
  `a107f741a632b0865717b1a81412fbf2dc464e11edb3c90376fe8bd90946b8d8`.

The source manifest contains seven exact permission-cleared source records,
and its source revision file is
`crates/prefixity-controlled-benchmark/src/phase1b9.rs` with SHA-256
`2f1dae56606815034530b9a7114170eea08b9269eed57b18a4fb5d9747830a33`.
The task prompt fingerprint is
`79691addddf3fdaaa9e5364bec578d4eae307fdcc2fc1af34d0bf7445d5e0d5f`.
The BASELINE and NO_OP projections are canonical-identical at
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`; the
INTERVENTION projection is
`9fa2765fd2e0701d9d588dbcb1e60834eff6ea0750be1e7852bf322d3c90b1d2`.
The h001 package is bound to scored runtime contract
`2f05440eb66d195d5362b9ad3a5dfc1708c4b8902cf8cd5774d26076c9aa1520`, its
contract schema fingerprint
`ad42524c6fb2f75056391754cfedd3d84658d797199429f0b68827fdb33d80fb`, and
the frozen pilot manifest
`201c21d7472dcf274bb912488f9334ac264bbba4968262e4e9fa4939a2597552`.

The isolated executor is
`crates/prefixity-controlled-benchmark/src/phase1c_h001.rs` with binary
`prefixity-phase1c-h001`. It supports offline `preflight`, `fingerprint`, and
per-arm `dry-run` commands, plus an explicitly confirmed single-arm live
command for a later authorization. It never starts or restarts llama.cpp,
performs no HTTP readiness probe, retries no request, follows no redirect,
reads no credential, auto-advances no arm, and writes no h001 evidence during
offline preparation. The later live command permits only one bounded TCP
listener check and one operator-selected arm request; ambiguous transport
stops with an explicit ambiguous result.

Offline evidence is `PREPARED`: h001 preflight and all three arm dry-runs
returned `network_calls=0`, `credential_reads=0`, and `inference_requests=0`.
The dry-run matrix verified the required BASELINE/NO_OP equality and exact
INTERVENTION difference. Focused h001 tests passed (`4`); the full workspace
test suite passed (`49` controlled-benchmark unit tests plus all workspace
integration/doc suites), strict workspace Clippy with `-D warnings` passed,
and `cargo fmt --all -- --check` passed. `git diff --check` passed. The h001
evidence directory remains absent, so no live response or evaluator-result
artifact exists.

No current or prior llama.cpp server was contacted, no listener check was
performed, and no inference request was issued in this materialization epoch.
Phase 1C live inference accounting therefore remains exactly `2` historical
schema-smoke requests; h001 scored requests are `0`, h001 scored evidence is
`0`, automatic retries are `0`, ContextBench work is `0`, and P0-L6 evidence
modification remains `0`. No production Prefixity behavior changed.

The preparation gate is now closed at:

`H001 BASELINE READY FOR SEPARATE LIVE AUTHORIZATION`

No live authorization is implied by this record, and no later h001 arm or
pilot case may be started from this preparation step.

## Phase 1C h001 BASELINE live execution record

The separately authorized first live arm was executed on the dedicated
`agent/phase-1c-scored-pilot-h001` branch only. The operator supplied a new
current confirmation for a fresh llama.cpp server specifically for
`h001 / replicate 1 / BASELINE`: model
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, Q4_0, context `8192`, one slot, metrics
enabled, explicit `reasoning=on`, endpoint `127.0.0.1:8080`, zero requests
since startup, and no manual prompt, browser request, warmup, or
model-generating endpoint contact. The supplied startup log confirmed model
load, `n_slots=1`, `n_ctx_slot=8192`, and the listener on
`http://127.0.0.1:8080`.

The checked-in runner performed exactly one bounded TCP listener check. It
passed in `2` ms, with `listener_check_attempts=1` and readiness
`inference_requests=0`. It then dispatched exactly one BASELINE request using
the frozen projection and wire request SHA-256
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`.
Automatic retries, fallback requests, adaptive replicates, padding turns,
and arm advancement were all `0`.

The request did not produce a certifiable completion. The runner preserved the
request and waited for the registered `600000` ms request ceiling; completion
then resolved as an ambiguous dispatch/completion error. The arm result is
therefore conservatively `AMBIGUOUS` / contract classification
`INCONCLUSIVE`, with HTTP status `null`, response body bytes `null`, no raw
response body, no normalized message content, no final answer, no usage or
timing telemetry, and no reasoning field available. No retry was attempted.
The recorded trajectory has `completed_turns=0`, `max_turns=3`,
`transport_attempts=1`, `inference_requests=1`, and `next_arm=null`.

The frozen deterministic evaluator was not invoked because there was no
certifiable terminal response or normalized final content. Accordingly,
task success, required-context recall, dependency/protocol validity, and
critical-regression assessment are unavailable rather than fabricated;
evaluator completeness is not met. This is an arm-level inconclusive result,
not a Prefixity capability conclusion and not a cross-arm comparison.

The persisted ignored evidence directory is
`experiments/runs/phase1c-scored-capability-v1/h001/replicate-1/baseline/`.
It contains exactly `preflight.json`, `runtime-confirmation.json`,
`readiness.json`, `request-turn-1.json`, `trajectory-state.json`, and
`arm-result.json`. Ordinary SHA-256 hashes are:

- `preflight.json` —
  `1072ff8ecdd503e665216ca3544ba4381b60f431eb4df47d05ebf5a219444f9`;
- `runtime-confirmation.json` —
  `bec37fc5982d2d775700a344dbac7a5a41bf4025d0f3ceede78196bde23441e6`;
- `readiness.json` —
  `1b96fcf761d07e5577d66d389d343d46b3ac87d7d047195c2110f181e02e6c7c`;
- `request-turn-1.json` —
  `26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`;
- `trajectory-state.json` —
  `dd5f1ce80a9f95f3216ab5b220658e94bc4ed0c25134e834f40a51fc6c59183c`;
- `arm-result.json` —
  `012b3dea5859e5279b3ddabf6a6ab7571104f9f4194975d2d9285e7b0ed453e5`.

No response, normalized-turn, or evaluator-result file exists because no
response completed. The evidence directory is ignored by
`experiments/.gitignore`; no runtime evidence was force-added.

Post-run checks were offline-only: bounded evidence listing and ordinary
file hashing, frozen-identity-preserving repository checks, focused h001
tests, workspace tests, strict Clippy, formatting, and `git diff --check`.
No forensic PowerShell transformation was used, no Defender control was
weakened, and no preserved prior evidence was modified.

Phase 1C accounting is now: historical schema-smoke requests `2`; h001
BASELINE requests `1`; h001 NO_OP requests `0`; h001 INTERVENTION requests
`0`; automatic retries `0`; ContextBench work `0`; Prefixity behavioral
changes `0`; and P0-L6 evidence changes `0`. The mandatory stop applies now:
do not start or execute NO_OP, INTERVENTION, h004, or another replicate.
NO_OP requires a separate fresh-runtime authorization after review of this
inconclusive BASELINE record.

## Phase 1C h001 BASELINE timeout forensic review

The authorized timeout review is complete and preserved in
`docs/phase-1/PHASE_1C_H001_BASELINE_TIMEOUT_FORENSIC_REVIEW.md`. The six
BASELINE evidence files were revalidated with ordinary SHA-256 hashing and
remain unchanged. The expected absent files remain absent:
`response-turn-1.bin`, `normalized-turn-1.json`, and
`evaluator-result.json`.

The exact frozen request was `1310` bytes, with two model-visible messages,
`max_tokens=2048`, `temperature=0`, `top_p=1`, `seed=1`, `stream=false`, the
frozen Qwen model identifier, and no tool definitions. The request and wire
fingerprint both remained
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`. No
model-independent h001 token estimate was persisted or introduced during
review.

The runner used the frozen `1000 ms` connect timeout and `600000 ms` reqwest
request timeout. After one successful TCP readiness check, the request was
dispatched once. `send()` returned an ambiguous error at exactly
`600000 ms`; no response object, status, headers, body, usage, timing, or
reasoning field was available. The runner then persisted the ambiguous
trajectory and arm result as designed, with `completed_turns=0`,
`transport_attempts=1`, `inference_requests=1`, and `next_arm=null`. Had a
response object arrived before a later body timeout, the implementation would
have preserved any bounded partial body, but that alternate path was not
observed.

The contract declares a `660000 ms` supervisor timeout, but the h001 binary
does not implement a supervisor timer and the executed Cargo invocation did
not provide an independently verified 60-second persistence margin.
Compilation completed before the live invocation, so this is distinct from
the Attempt 007 compilation/supervisor failure. The missing supervisor
enforcement is recorded as an implementation/design gap, not asserted as
the cause of the observed timeout.

No server-side completion log was persisted. The required forensic boundary
is therefore:
`SERVER-SIDE COMPLETION STATE NOT ESTABLISHED`.
The strongest supported classification is
`REQUEST_TIMEOUT_BUDGET_EXHAUSTED`, with high confidence for the client-side
deadline event but low confidence for the underlying server-side cause.
Exceeding 600 seconds for a 2048-token reasoning-enabled execution is
plausible but not established from the sparse prior Smoke 01/02 and P0-L6
observations. Timeout/budget adequacy is classified `NOT_ESTABLISHED`, not a
capability failure.

The original BASELINE remains `AMBIGUOUS / INCONCLUSIVE`; the frozen
deterministic evaluator was not invoked because no certifiable terminal
response or normalized final content existed. NO_OP and INTERVENTION remain
`BLOCKED`, and no valid h001 comparison exists.

Remediation ranking is: (1) a new timeout-only V2 contract preserving all
task/model/reasoning/generation semantics while increasing the documented
request allowance and actually enforcing the supervisor margin; (2) reduced
`max_tokens`, which changes capability semantics; (3) reasoning off, which
also changes scored capability configuration; and (4) a different
model/runtime/tuning, which is highest impact. No remediation was
implemented or authorized. Any future corrected BASELINE requires a new
runtime-contract version and distinct attempt/evidence identity; the original
`h001 / replicate-1 / baseline` evidence must not be overwritten.

Accounting remains: historical schema-smoke requests `2`, h001 BASELINE
requests `1`, h001 NO_OP `0`, h001 INTERVENTION `0`, total Phase 1C live
requests `3`, automatic retries `0`, ContextBench work `0`, Prefixity
behavioral changes `0`, and P0-L6 evidence changes `0`. No new inference or
localhost contact occurred during review.

## Phase 1C h001 timeout-only V2 preparation record

The approved offline-only timeout remediation preparation is complete on
branch `agent/phase-1c-h001-timeout-v2-prep`. Accepted h001 V1 history was
first promoted to `main` at `80152eb5ef55fa8e1c38d6ea97602cccbc615e63`; the V2
branch is rooted at that promoted commit and has not been merged to `main`.

The new V2 runtime contract, pilot manifest, and h001 BASELINE execution
identity are checked in under `docs/phase-1/` with SHA-256 sidecars:

- contract:
  `75dcc6a8a4db162e38557487c516ebe102ffebaa55c3e10a89a33d7d2c76b620`;
- pilot manifest:
  `f8548455180f0e75d3e35a5662bbef3f79e2aca4ef506e281800c82a8feefb9e`;
- h001 V2 BASELINE identity:
  `dea4c693bcea61e7951c49d8d95b7d83294d10409d5c606b6d0f6a004f154bcc`.

The V2 validator proves that model, quantization, context, parallelism,
metrics, reasoning mode, sampling, seed, model-visible request, task/source/
required/tool/evaluator/arm identities, pilot cases, arm order, replicate,
generation bound, retry policy, and freshness/cache semantics are unchanged
from V1. The only runtime budget delta is `complete_request_timeout_ms`
`600000 -> 1200000`; the outer supervisor is implemented and actually
enforced at `1320000` ms. Connect timeout remains `1000` ms. The V2 h001
BASELINE projection remains exactly 1310 bytes and hash
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`.

The experiment-only `prefixity-phase1c-live-supervisor` binary launches one
already-built child runner, performs no network/inference/retry/arm advance,
and distinguishes `COMPLETED`, `CHILD_FAILED`, and
`SUPERVISOR_TIMEOUT`. Bounded tests cover normal completion, child failure,
and timeout termination without waiting 22 minutes. The future V2 h001 runner
is bound to the V2 contract and evidence root
`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline/`.

Offline V2 fingerprint, preflight, dry-run, focused V2 tests, supervisor
tests, formatting, and strict Clippy passed. The preflight/dry-run accounting
is zero network calls, zero credential reads, and zero inference requests.
No V2 evidence directory was created; V1 evidence remains frozen and was not
modified. No large inline PowerShell forensic command was used and endpoint
protection was not weakened.

This preparation stops before any Qwen start, listener check, V2 BASELINE
request, NO_OP, INTERVENTION, other case, or additional replicate. The exact
remaining state is:

`H001 V2 BASELINE READY FOR SEPARATE LIVE AUTHORIZATION`

## Phase 1C h001 V2 BASELINE live execution record

The separately authorized h001 V2 BASELINE launch was attempted once through
the checked-in `prefixity-phase1c-live-supervisor` using the already-built
`prefixity-phase1c-h001-v2` child. The operator had supplied the required
fresh-runtime confirmation for Qwen3.5-0.8B Q4_0, context 8192, one slot,
metrics enabled, reasoning on, and `127.0.0.1:8080`, with zero requests since
startup.

The supervisor launched exactly one child with no retry. The child failed
before V2 preflight and before the single TCP readiness check because the V2
CLI parser expects the confirmation flag in the wrong tuple position at
`crates/prefixity-controlled-benchmark/src/bin/phase1c_h001_v2.rs:13-14`.
It printed usage and exited with code `1`. The supervisor persisted
`CHILD_FAILED`; supervisor network calls and inference requests were `0`, and
no localhost readiness check or HTTP request occurred.

The bounded record is
`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline/supervisor-result.json`
with ordinary SHA-256
`bf65a2b4e9a326354566df8852d4d05e858f263e628cd46e046cb8d2f0492e5`. No
request, response, normalized, trajectory, or arm-result artifact was
fabricated. The V1 evidence tree and V2 contract/manifest/identity remain
unchanged.

This is a pre-inference runner failure, not a capability result. The V2
BASELINE launch is consumed; no retry, patch-and-rerun, NO_OP, INTERVENTION,
h004, or additional replicate is authorized by this record. Separate offline
remediation and fresh live authorization are required for any corrected run.

Accounting remains: historical schema-smoke requests `2`, V1 h001 BASELINE
requests `1`, V2 h001 BASELINE requests `0`, NO_OP `0`, INTERVENTION `0`, h004
`0`, retries `0`, and Prefixity behavioral changes `0`.

`H001 V2 BASELINE CHILD_FAILED BEFORE MODEL CONTACT — STOP`

## Phase 1C h001 V2 BASELINE attempt 002 preparation record

The approved offline remediation of V2 BASELINE launch attempt 001 is complete
on branch `agent/phase-1c-h001-v2-cli-fix`. Attempt 001 remains permanently
frozen at
`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline/supervisor-result.json`
with SHA-256
`bf65a2b4e9a326354566df8852d4d05e858f263e628cd46e046cb8d2f0492e5` and
classification `CHILD_FAILED` before model contact. Its one child launch,
zero readiness checks, zero HTTP requests, and zero inference requests are
not retried or rewritten.

The experiment-only V2 child CLI defect was repaired narrowly. The prior
parser expected `(None, Some(flag), None)` after `run`; it now accepts the
canonical `run --confirm-fresh-runtime` argv through a pure shared parser.
Missing confirmation and malformed placement fail closed. The supervisor's
registered child argv is tested against that parser, catching the exact
attempt-001 defect. No task, request body, serializer, runtime contract,
timeout, model, reasoning, generation, evaluator, evidence, or Prefixity
behavior changed.

New attempt-002 identity:
`docs/phase-1/PHASE_1C_H001_V2_BASELINE_ATTEMPT_002_IDENTITY_V1.json`, canonical
SHA-256
`e8a28fb795f6c0ea635f2a8d9b5533f39b0ed33b7284d71449e8aa3b88f1a95c`. It binds
V2 contract/pilot hashes, all frozen h001 artifact hashes, the exact 1310-byte
BASELINE projection
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`,
replicate 1, BASELINE, attempt 002, the corrected CLI identity, and attempt-001
lineage. Its distinct future evidence path is
`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline-attempt-002/`;
that directory is absent. The old attempt-001 directory is not reused.

Offline parser, supervisor, V2 identity, attempt-002 preflight, and dry-run
checks pass. Preflight reports `PREPARED` with zero network calls, zero
credential reads, zero listener checks, and zero inference requests. Dry-run
reports the unchanged 1310-byte request and frozen projection hash. Attempt
002 is prepared but not live-authorized: do not start Qwen, perform readiness,
execute BASELINE, or run NO_OP, INTERVENTION, h004, or another replicate.

Accounting remains: historical schema-smoke inference requests `2`, V1 h001
BASELINE inference requests `1`, V2 launch attempt 001 inference requests `0`,
V2 BASELINE attempt 002 inference requests `0`, NO_OP `0`, INTERVENTION `0`,
h004 `0`, retries `0`, and Prefixity behavioral changes `0`.

`H001 V2 BASELINE ATTEMPT 002 READY FOR SEPARATE LIVE AUTHORIZATION`

## Phase 1C h001 V2 BASELINE attempt 002 live execution record

The separately authorized h001 V2 BASELINE attempt 002 was executed once on
2026-09-02 from branch `agent/phase-1c-h001-v2-baseline-attempt-002` at
`984b075cfe5eb776030c7bcc62b91043becb679c`. The operator confirmed a fresh
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0` llama.cpp server with context `8192`, one
parallel slot, metrics enabled, reasoning explicitly `on`, endpoint
`127.0.0.1:8080`, and zero prior inference requests or model-generating
probes. The V2 contract, pilot, h001 manifest, attempt-002 identity, and
frozen BASELINE projection matched their accepted hashes before model contact.

The checked-in `prefixity-phase1c-live-supervisor` launched the child exactly
once with canonical argv `run --confirm-fresh-runtime`. The one permitted TCP
listener check passed in `3 ms` with `inference_requests=0`. The child exited
with code `0`; the supervisor classified the run `COMPLETED`, enforced the
`1320000 ms` outer deadline, and recorded zero supervisor retries and zero
supervisor network calls.

BASELINE dispatched exactly one request and completed one turn. The request
and wire request fingerprints were both
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a` for the
frozen 1310-byte projection. The response was HTTP `200`, 5387 bytes, with
SHA-256
`deb00a0e9d3579a93023166498a7763e7b5cd9222d29afc81912940a453eba70` and
transport elapsed `598717 ms`. Prompt tokens were `389`, completion tokens
were `2048`, cached tokens were reported as `0`, and the finish reason was
`length`. Reasoning content was present as diagnostic evidence only and was
not scored; final content was available, but terminal final content was
false, with no tool call. The trajectory therefore classified the BASELINE
as `INCONCLUSIVE` after one completed turn, with no recovery turn and no
automatic retry.

The deterministic evaluator was not run because the BASELINE did not reach a
certifiable terminal state. The exact reason recorded by the runner is
`terminal final content was unavailable before the registered ceiling`.
No task-success or cross-arm claim is made.

Attempt-002 evidence is preserved only under
`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline-attempt-002/`.
The persisted artifacts and ordinary SHA-256 values are:

- `supervisor-result.json`: `0f99a2936e369a155708d8001157280911cf02af53b14c7b04222ab39bd13f32`;
- `replicate-1/baseline/preflight.json`: `1ef65defa6a40eaeaf76323131b51cad2c2bada0dce3eb4784e4a4624b766e00`;
- `replicate-1/baseline/runtime-confirmation.json`: `5305f9507af6e21d3042229c6a84c72470bce7da772cc5524a3026bc54d5470c`;
- `replicate-1/baseline/readiness.json`: `f10f17b2edf567bdc7701fd668818322eac6b71e986282d2f81b2da938d84565`;
- `replicate-1/baseline/request-turn-1.json`: `26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`;
- `replicate-1/baseline/response-turn-1.bin`: `deb00a0e9d3579a93023166498a7763e7b5cd9222d29afc81912940a453eba70`;
- `replicate-1/baseline/normalized-turn-1.json`: `6a34df1215aa0a69dd79d31976a984ffb4246d68b0a91b9501b58c6a9e15bfb2`;
- `replicate-1/baseline/trajectory-state.json`: `93c35767e59d64f2cd06a5516f9e6005c9575070ef3f613284aefa29a78e0e0b`;
- `replicate-1/baseline/arm-result.json`: `bc2ab460fa6546987ab02b8c6d48f47c75646a39e38d47e18312033896d4f6d5`.

The runtime evidence remains ignored experiment output and was not force-added.
Attempt 001 and all V1 evidence remain unchanged. Post-run offline focused V2
tests (10), supervisor tests (4), formatting, and strict Clippy passed. The
three lifecycle-sensitive preparation tests now explicitly verify the
attempt-002 evidence guard after the canonical run; no runtime behavior was
changed. No further localhost contact occurred after the BASELINE completed.

Accounting is: historical schema-smoke inference requests `2`, V1 h001
BASELINE requests `1`, V2 attempt 001 requests `0`, V2 attempt 002 BASELINE
requests `1`, NO_OP `0`, INTERVENTION `0`, h004 `0`, automatic retries `0`,
and Prefixity behavioral changes `0`.

`H001 V2 BASELINE ATTEMPT 002 INCONCLUSIVE - DO NOT RETRY OR EXTEND TIMEOUT`

## Phase 1C h001 V2 closeout and reasoning-budget calibration preparation

The accepted h001 V2 BASELINE result is closed as
`PHASE 1C h001 V2 - INCOMPLETE / NON-COMPARABLE`. BASELINE is
`INCONCLUSIVE - GENERATION CEILING EXHAUSTED`; it returned HTTP `200` with
2048 completion tokens, `finish_reason=length`, present reasoning content,
and no terminal final content. The deterministic evaluator was not run.
NO_OP and INTERVENTION were not executed. V1 and V2 evidence remain
immutable, and no future calibration result may be combined with either
scored lineage.

The accepted execution record commit
`f430688b3e327a42bd407185f2a847a4af318dcf` was fast-forward promoted to
`main` and directly verified at `origin/main` with the same SHA. The current
preparation branch is `agent/phase-1c-reasoning-budget-calibration`, created
from that promoted main. No h001 retry, timeout extension, V3, NO_OP, or
INTERVENTION was started.

Offline inspection of the installed local llama.cpp executable reported build
`b10217-ddd4ec142`. Its `serve --help` exposes server-side
`--reasoning-budget N` with semantics `-1=unrestricted`, `0=immediate end`,
and `N>0=token budget for thinking`, plus
`--reasoning-budget-message MESSAGE`. The calibration uses only the budget
flag, leaves the message unset, and keeps explicit `--reasoning on`.
No server was started and no localhost contact occurred during this design.

The independent non-scored calibration manifest is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.json` with
canonical SHA-256
`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b` and
sidecar
`docs/phase-1/PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.sha256`.
It freezes exactly `rbcal-001`, `rbcal-002`, and `rbcal-003`, in that order,
with request hashes
`f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461`,
`e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3`, and
`2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e`.
The only candidates are `1024 -> 512 -> 256`; maximum requests are `9`.
Each candidate requires a fresh server, reasoning on, total `max_tokens=2048`,
and zero retries. The candidate passes only if all three cases return HTTP
200, complete non-length terminal output, and the exact structural JSON
response.

The experiment-only runner is
`crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs`
with binary
`prefixity-phase1c-reasoning-budget-calibration`. Its offline commands are
`fingerprint`, `preflight`, and `dry-run`; the future live command is
`run --budget {1024|512|256} --confirm-fresh-runtime`. It keeps the budget out
of the model-visible request, captures reasoning in a separate diagnostic
artifact, never scores reasoning, enforces candidate order/stopping, and
never retries.

Offline `fingerprint`, `preflight`, and all-nine-combination `dry-run` checks
passed with network calls `0` and inference requests `0`. Focused runner tests
cover manifest independence, request hashes, candidate order, stopping,
server-budget argument binding, malformed CLI rejection, zero retry, and
reasoning/final-content isolation. Full workspace tests, strict Clippy,
rustfmt, and `git diff --check` all passed before the preparation branch was
pushed.

This calibration preparation stops before server startup, listener readiness,
HTTP, inference, warmup, or any candidate execution. A later authorization
must name the frozen manifest and candidate, provide a fresh-runtime
confirmation, and explicitly authorize the bounded live calibration.

`PHASE 1C REASONING-BUDGET CALIBRATION READY FOR SEPARATE LIVE AUTHORIZATION`

## Phase 1C reasoning-budget calibration 1024 execution stop

The separately authorized candidate-1024 execution was stopped after the
llama.cpp server crashed during the first case. The dedicated live branch is
`agent/phase-1c-reasoning-budget-calibration-1024`, based on promoted commit
`cecc6368b5e1255fecb333ff2d2712417fc7e961`. Before execution,
`origin/main`, the preparation branch, and local HEAD were directly verified
at that commit. The frozen manifest SHA remained
`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b` and all
three registered case request hashes remained unchanged.

The operator confirmed model
`ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, Q4_0, context `8192`, one slot, metrics
enabled, reasoning `on`, reasoning budget `1024`, no reasoning-budget message,
endpoint `127.0.0.1:8080`, fresh candidate-specific startup, zero prior
inference requests, and no warmup or manual/browser/API request. The server
was started with only the registered flags. The single registered listener
check passed once in `2 ms`, with inference requests `0`.

The runner then created exactly one `rbcal-001` request record using the
frozen request projection. No response, normalized response, reasoning
diagnostic, or structural result was persisted. The server failure left
dispatch and transport completion unverified. The runner was stopped
immediately; no retry, second listener check, or additional inference was
issued. Residual llama processes were then stopped, and no llama process
remained. No Defender or other endpoint-security intervention occurred.

Case accounting is:

- `rbcal-001`: one request attempt; HTTP status, prompt tokens, completion
  tokens, reasoning presence, finish reason, terminal content, transport
  elapsed, response hash, and structural result are unavailable because no
  response artifact exists; classification `INVALID / AMBIGUOUS`.
- `rbcal-002`: not run after the mandatory infrastructure stop.
- `rbcal-003`: not run after the mandatory infrastructure stop.

The candidate result is exactly
`REASONING BUDGET 1024 - INVALID / AMBIGUOUS`. This is an infrastructure /
transport-ambiguity result, not a structural feasibility failure and not a
Prefixity capability claim. Candidate `512`, candidate `256`, V3, h001,
NO_OP, INTERVENTION, and h004 remain unexecuted and unauthorized.

Preserved ignored evidence is under
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024/`.
Ordinary SHA-256 values are:

- `preflight.json`: `51145B675531A9E42E6C2FF2A9F560ADF256D3CBB7512757CB536A276C61A2B8`;
- `runtime-confirmation.json`: `2F6C917AED5B938418F295827B46E295F3AEC0FBE23AD606F17C06203F61527E`;
- `readiness.json`: `DCCCD1282BBE076DD6AE2E1461D7C7ED9A3556F39A22E78D1688E6698FDD49AD`;
- `rbcal-001/request-turn-1.json`:
  `F32863DFB1DA27C00A61D54986D4984569C87E9636CF5C6263C69906CB336461`.

No response hash exists because no response file was persisted. The
evidence root remains ignored and no evidence was force-added. Post-stop
offline validation and a narrowly scoped execution-record commit are required
before this branch is pushed.

## Phase 1C reasoning-budget 1024 launch attempt-001 closeout and attempt-002 preparation

The first live 1024 calibration launch is permanently recorded as
`1024 / LAUNCH ATTEMPT 001 - INVALID / AMBIGUOUS` for
`LLAMA_RUNTIME_FAILURE_WITH_UNCERTAIN_REQUEST_COMPLETION`. Its request
attempt count is `1`; `rbcal-001` has no structural result because no response
was persisted; `rbcal-002` and `rbcal-003` were not run. No structural
feasibility conclusion is drawn, and the root cause remains
`ROOT CAUSE NOT ESTABLISHED`. Attempt-001 evidence remains immutable.

The accepted invalid-attempt history is prepared for fast-forward promotion to
`main` at:

`e5c6bec173d9c1b1e371fbc32624ca74adb0605a`

The direct remote check in this closeout still reports `origin/main` at
`cecc6368b5e1255fecb333ff2d2712417fc7e961`. The requested fast-forward was
blocked by the execution-policy boundary, so no remote `main` mutation was
completed here.

The new preparation branch is
`agent/phase-1c-reasoning-budget-calibration-1024-attempt-002-prep`, created
from the accepted attempt-001 commit above. The tracked attempt-002 identity is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_002_IDENTITY_V1.json`
with canonical SHA-256
`7d9dd05ed5c855f02dc5b37a70e7cac257af03ce5dc87686acf1e64590f37610` and a
sidecar at
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_002_IDENTITY_V1.sha256`.
It binds manifest SHA
`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b`, budget
`1024`, the unchanged case order and request hashes, the attempt-001 root and
classification, llama.cpp build `b10217-ddd4ec142`, maximum requests `3`,
zero retries, fresh-server requirement, and runtime exclusivity.

Attempt-002 uses the distinct evidence root
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-002/`.
It must begin absent; attempt-001's
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024/` root must
not be reused or overwritten. The manifest, cases, request hashes, runtime
settings, generation settings, timeout policy, candidate order, and retry
policy are unchanged.

The checked-in runner now exposes offline `attempt-002-fingerprint`,
`attempt-002-preflight`, and `attempt-002-dry-run` commands. The dry-run
projects exactly the three `1024` cases with unchanged request bytes and
hashes, the isolated root, network calls `0`, listener checks `0`, and
inference requests `0`.

Before a future server startup, the required OS-only exclusivity preflight
uses bounded `tasklist /FO CSV /NH` and `netstat -ano -p tcp` inspection to
require no `llama.exe`, no listener on port `8080`, and no other Prefixity or
Qwen runner. It also requires explicit operator attestation that no Luna,
Codex, helper, browser, terminal, or automation workflow is configured to
interact with the runtime. After startup and before inference, the future
runner records the single expected server PID, registered executable path,
port owner, and fresh-start identity; any multiple-PID or ownership mismatch
stops before inference.

The exclusive live-window rule is registered: from fresh-server confirmation
until candidate completion or stop, no other agent, terminal workflow,
automation, browser, Luna process, Codex helper, or manual action may probe,
invoke, restart, stop, or otherwise interact with the Qwen/llama runtime or
port `8080`.

Offline implementation checks include workspace all-target check, identity
fingerprinting, attempt-002 preflight, and attempt-002 dry-run, all with zero
model contact. Focused calibration tests (`12`), attempt-identity tests,
exclusivity parser tests, serialized supervisor tests (`4`), full workspace
tests, strict Clippy, rustfmt, and `git diff --check` all passed. No Qwen
startup, listener check, localhost contact, or inference occurred in this
preparation.

The preparation commits are `e243fb0c653ea9858a12ee434d58865bcca41285` and
the documentation closeout at `e8d6c022cff4312ceac33829a3581573cf6a159d`, and the
preparation branch has been pushed. No live execution is authorized by this
closeout.

`ROOT CAUSE NOT ESTABLISHED`

`REASONING BUDGET 1024 ATTEMPT 002 READY FOR SEPARATE LIVE AUTHORIZATION`

## Phase 1C reasoning-budget 1024 Attempt 002 pre-server exclusivity stop

The explicitly authorized Attempt 002 live branch is
`agent/phase-1c-reasoning-budget-calibration-1024-attempt-002`, created from
preparation commit `bdcc357d62236da9b2511c4aa60fc967d99cc8f8`, pushed before
any runtime action. The authorized fast-forward was completed and directly
verified: `origin/main` is
`e5c6bec173d9c1b1e371fbc32624ca74adb0605a`.

On 2026-09-05 the checked-in, registered pre-server exclusivity command was
invoked with the required no-other-workflow confirmation. Its bounded
`tasklist /FO CSV /NH` inspection returned `ERROR: Access denied`, so process
exclusivity could not be established. Per the live authorization, execution
stopped before Qwen startup. No port inspection was completed after that
failure, and there was no server start, listener check, localhost contact,
warmup, inference request, or retry.

Attempt 002 therefore has zero inference requests and no new evidence root;
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-002/`
remains absent. Attempt 001 remains immutable. No structural candidate result
is claimed, and no alternate shell syntax or unregistered runtime probe was
used to bypass the failed exclusivity gate.

`REASONING BUDGET 1024 ATTEMPT 002 INVALID — STOP FOR RUNTIME INVESTIGATION`

## Phase 1C Attempt 002 closeout and Attempt 003 Windows exclusivity remediation

Attempt 002 is accepted as `INVALID BEFORE QWEN STARTUP` for
`PROCESS_INSPECTION_PERMISSION_FAILURE`. The registered `tasklist /FO CSV /NH`
mechanism returned `ERROR: Access denied`. Its execution record is commit
`a27b4fb411a284a6503d9a1c5525e30b1ae8862c`; the accounting is zero Qwen
startup, zero listener checks, zero localhost contact, zero inference requests,
zero calibration evidence-root creation, and zero retries. Attempt 001 remains
immutable.

The remediation branch is
`agent/phase-1c-windows-runtime-exclusivity-remediation`, created from
canonical `main` and fast-forwarded through the reviewed Attempt 002
preparation/stop-record history. No live runtime action is authorized on this
branch.

The checked-in remediation replaces shell-dependent `tasklist` and `netstat`
inspection with read-only Windows-native process, TCP owner-PID, and executable
path inspection. It uses Toolhelp32 process enumeration,
`GetExtendedTcpTable` filtered to port `8080`, and
`QueryFullProcessImageNameW` under `PROCESS_QUERY_LIMITED_INFORMATION`. It
does not elevate, terminate, modify, or contact localhost. Pure tests cover
exclusive prestart, multiple llama processes, unrelated port ownership,
PID/port mismatch, inspection failure, and executable-path failure. Luna,
Codex, browser, terminal, and automation exclusivity remains an explicit
operator attestation.

Attempt 003 identity:
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_003_IDENTITY_V1.json`

Identity SHA-256:
`c40e4528d6dc895a6e688a77dd1159b4c4e740b6a1b78ab2fc5259a0bc02655f`

Native implementation SHA-256:
`019ed0c7b066773b570f29adb6130b4447b1e9610e13724933b79a9e40374e93`

Attempt 003 evidence root:
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-003/`

The root remains absent. Offline fingerprint, preflight, and dry-run passed.
The bounded native local probe completed without administrator elevation and
reported no `llama.exe` process, no port-8080 listener, and
`EXCLUSIVE_PRESTART`. OS-table inspections were separate from endpoint
contact: network calls `0`, listener checks `0`, and inference requests `0`.
The dry run projected exactly the three unchanged 1024 cases and request
hashes.

No Attempt 003 live branch was created and no Qwen runtime, readiness check,
or calibration request was executed. The next live action requires separate
authorization.

`REASONING BUDGET 1024 ATTEMPT 003 READY FOR SEPARATE LIVE AUTHORIZATION`

## Phase 1C reasoning-budget 1024 Attempt 003 live closeout

The accepted remediation was promoted without a merge commit:
`origin/main` is `74f8380dc62e622cd48baebd21d5ea5bebbf91c9`. The live branch is
`agent/phase-1c-reasoning-budget-calibration-1024-attempt-003`, initially at
that same commit. The registered native prestart gate passed before Qwen
startup: zero llama processes, zero port-8080 listeners, zero competing
registered workflows, successful process/TCP inspection, no elevation, and
zero network/listener/inference contacts.

The fresh server was started with the frozen Attempt 003 parameters. Build
`b10217-ddd4ec142` was confirmed. Native post-start inspection found llama PID
`28772` at
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`, and port 8080
was owned by that PID. The post-start gate then failed closed because the
registered supervisor process `prefixity-phase1c-live-supervisor.exe` was
identified as a competing Prefixity workflow process. No readiness check and
no inference request was dispatched. The supervisor completed with one child
launch, zero retries, and zero supervisor inference requests. The Qwen server
was stopped and native shutdown verification found zero llama processes and
zero port-8080 listeners.

Attempt 003 is therefore:

`REASONING BUDGET 1024 ATTEMPT 003 INVALID / AMBIGUOUS`

No case was dispatched: rbcal-001, rbcal-002, and rbcal-003 have no HTTP,
prompt-token, completion-token, reasoning, finish-reason, terminal-content,
or response-hash result. Attempt 003 inference requests are `0`; readiness
attempts are `0`; retries are `0`. No Attempt 004 or lower-budget candidate is
prepared or executed under this authorization.

Attempt 003 evidence files are ignored experiment artifacts:

- `experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-003/preflight.json`
  SHA-256 `73921f286b141a48802499b1081c660500425fbf5c59d0e54880e1a4588e00e8`
- `experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-003/runtime-ownership.json`
  SHA-256 `333c4fbcbbf90d24e28ce44890a478d06d233ced785261c7e08b5f0cb233337c`
- `experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-003/candidate-result.json`
  SHA-256 `b756093101fb7683ea74ff4166bd66c6e1129eea26e7e8a6f7791fcec449d62b`
- `experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-003/supervisor.json`
  SHA-256 `7a396f58b156bb7abb2eda565be1ed15541310717eac597dab8a7902150e9387`

The next action is runtime investigation of the supervisor/process-exclusivity
policy. No further localhost/model contact is authorized by this record.

## Phase 1C Attempt 003 closeout and Attempt 004 expected-workflow remediation

Attempt 003 is accepted as `INVALID BEFORE READINESS / INFERENCE` with root
cause `EXPECTED_SUPERVISOR_MISCLASSIFIED_AS_COMPETING_WORKFLOW`. Its accepted
execution record was fast-forwarded to `main` and pushed as
`dc0a8801cee4ddf6f3bc928ecb6bfd954ec7eeec`. The preserved evidence remains
unchanged and hashes to:

- `preflight.json`: `73921f286b141a48802499b1081c660500425fbf5c59d0e54880e1a4588e00e8`
- `runtime-ownership.json`: `333c4fbcbbf90d24e28ce44890a478d06d233ced785261c7e08b5f0cb233337c`
- `candidate-result.json`: `b756093101fb7683ea74ff4166bd66c6e1129eea26e7e8a6f7791fcec449d62b`
- `supervisor.json`: `7a396f58b156bb7abb2eda565be1ed15541310717eac597dab8a7902150e9387`

The remediation branch is
`agent/phase-1c-expected-workflow-exclusivity-remediation`. The repaired
classifier binds the supervisor PID/path, calibration-child PID/path and
parent relationship, optional inspector identity, and a supervisor-generated
launch identity. It never globally whitelists `prefixity-*` or
`*-supervisor.exe`; unregistered or extra workflow processes remain
`UNEXPECTED_WORKFLOW_PROCESS`, and missing/path-mismatched expected identities
fail closed. The required successful poststart state is
`NO_UNEXPECTED_WORKFLOW_PROCESSES`.

Attempt 004 preparation is tracked by
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_004_IDENTITY_V1.json`,
canonical SHA-256
`7e59288ccc2847298482dfe6aa4dfe0e2d3e4197fdfbe72031f9551e51c675c9`, and
repaired native implementation SHA-256
`9dba33fdc0c4c9279e5df4eb12b3be0a0f5f99492f74efa548310ebf4163c37b`.
The Attempt 004 root remains absent. Offline fingerprint, preflight, and
three-case dry-run commands passed with the frozen manifest/request hashes,
budget 1024, `EXCLUSIVE_PRESTART`, network calls `0`, listener checks `0`, and
inference requests `0`. The live branch may be created from a promoted
remediation commit, but no Qwen startup, readiness check, or inference is
permitted in this task.

`REASONING BUDGET 1024 ATTEMPT 004 READY FOR SEPARATE LIVE AUTHORIZATION`

## Phase 1C Attempt 004 live closeout

Attempt 004 was authorized for the fresh Qwen runtime with reasoning on and
reasoning budget 1024. The pre-live repository gate passed at commit
56cdcc879b0488ac9459ca234bb5c8ccd1dd70cc. The native prestart gate passed
with EXCLUSIVE_PRESTART, zero llama processes, zero port-8080 listeners, zero
unexpected processes, successful process/TCP inspection, no elevation, and
zero network, listener, or inference contacts.

The exact authorized llama.cpp command reached model-loaded and local-listener
state with the frozen model, context, one slot, metrics, reasoning, budget,
host, and port settings. Startup also reported an HTTPLIB repository-commit
lookup failure while resolving the hf reference, but continued to local model
load and listening. This was not a localhost readiness or inference request.

The mandatory native poststart gate failed closed before readiness with
EXPECTED_WORKFLOW_IDENTITY_INVALID: the expected workflow launch identity was
not handed off by the supervisor. The server had been started directly from
the authorized shell command, so no supervisor or calibration-child identity
existed. No identity was fabricated and no second workflow was started. The
observed server was stopped, and native shutdown verification returned
EXCLUSIVE_PRESTART with zero llama processes and zero port-8080 listeners.

Attempt 004 is INVALID / AMBIGUOUS before readiness and inference. No case
was dispatched; rbcal-001, rbcal-002, and rbcal-003 remain unrun. Inference
requests are 0, readiness attempts are 0, automatic retries are 0, and no
request or response artifacts exist. The 512 and 256 candidates, NO_OP,
INTERVENTION, V3, and Attempt 005 were not executed or authorized.

Preserved ignored evidence is in
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-004/.
The execution record and evidence hashes are documented in
docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_004_EXECUTION_RECORD.md.
Prior Attempt 003 evidence was not modified. Focused calibration, supervisor,
and native exclusivity tests passed after closeout bookkeeping; final
formatting, clippy, and diff checks also passed before the scoped execution
record was committed and pushed to the Attempt 004 branch. It will not be
merged to main.

The next permitted action is runtime investigation of the missing supervisor
handoff. No further localhost/model contact is authorized by this record.

REASONING BUDGET 1024 ATTEMPT 004 INVALID — STOP FOR RUNTIME INVESTIGATION

## Phase 1C Attempt 004 handoff remediation and Attempt 005 preparation

The accepted Attempt 004 history was fast-forward-promoted to main and
verified on origin/main at
5b16161babe015a66f0a2651610963ac7b7ec735. Attempt 004 remains
INVALID BEFORE READINESS / INFERENCE with immediate cause
SUPERVISOR_LAUNCH_IDENTITY_HANDOFF_MISSING. Accounting remains Qwen startup 1,
readiness 0, inference 0, and retries 0. Its preserved evidence remains
unchanged.

The exact defect was traced to the real
prefixity-phase1c-live-supervisor binary calling the generic run_supervised
path with no handoff. The remediation replaces the three unused environment
variables with one canonical serialized WorkflowLaunchMetadata handoff,
PREFIXITY_PHASE1C_WORKFLOW_HANDOFF. The supervisor generates the launch
identity from the registered Attempt 005 identity and passes its own PID/path,
candidate identity, evidence-root identity, and exact child binding. The child
validates that metadata against the registered identity; native exclusivity
then verifies the parent process, exact executable paths, llama PID, and port
owner. All mismatches fail closed.

Attempt 005 identity:
docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_005_IDENTITY_V1.json

Attempt 005 identity SHA-256:
5d247fe0c5b29d99b1652254743c5ee3d38833244d30c2ef4e64795f7159a8c6

Attempt 005 evidence root:
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-005/

The root remains absent. Manifest and all three frozen request hashes are
unchanged. Offline fingerprint, preflight, and dry-run passed with network
calls 0, listener checks 0, inference requests 0, and three cases. Focused
calibration 22/22, supervisor 6/6, native exclusivity 12/12, full workspace
tests, strict Clippy, rustfmt, and git diff --check passed. No Qwen startup,
localhost contact, readiness, or inference was performed.

The remediation branch is to be committed and pushed, then fast-forward
promoted to main. After promotion, create
agent/phase-1c-reasoning-budget-calibration-1024-attempt-005 as preparation
only. Do not execute Attempt 005 under this authorization.

REASONING BUDGET 1024 ATTEMPT 005 READY FOR SEPARATE LIVE AUTHORIZATION

## Phase 1C Attempt 005 hash forensics and Attempt 006 preparation

Attempt 005 is permanently frozen as `1024 / ATTEMPT 005 — NOT STARTED`.
Its pre-live repository gate failed before Qwen startup because the sealed
supervisor source identity was `df74a196b79e0407c0d4d83f4a8aede974904a505a6462a72a9e868eda8e8744`,
while the Windows checkout hash was `d18e56fb68c0be8e58a10c94cc8333186614c7ef2f928bc3c252d143207d273f`.
No startup, localhost contact, readiness, request, inference, or retry
occurred, and Attempt 005 identity/evidence were not rewritten.

Offline history established that `df74…` is the LF-preserving source hash at
`bab45ca7cf8bb859de8923e6f6097443915425e9` and `d18…` is a CRLF checkout
representation. The shared fingerprint path now normalizes CRLF and lone CR
to LF and rejects a UTF-8 BOM, with focused regression coverage. Attempt 006
also adds only the attempt-specific evidence-root lookup needed to bind its
new identity; no model-facing or runtime-contract semantics changed.

Attempt 006 identity:
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_006_IDENTITY_V1.json`

Attempt 006 canonical identity SHA-256:
`1d326578deb5a5280eafbed5d844a754d1468429c4990b25a0ee207652ca28a1`

Attempt 006 authoritative supervisor source SHA-256:
`76f122f4f6cf096a6a972d5770122b5590767467972aa43c7469d51b1b2656ef`

Native exclusivity source remains:
`9dba33fdc0c4c9279e5df4eb12b3be0a0f5f99492f74efa548310ebf4163c37b`.

Attempt 006 evidence root
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-006/`
remains absent. Its offline fingerprint, preflight, and dry-run passed with
the unchanged manifest/request hashes, `EXCLUSIVE_PRESTART`, and zero network,
listener, or inference activity. Focused identity, source-fingerprint, and
handoff tests passed. No Qwen startup or localhost contact occurred.

Attempt 006 requires separate live authorization and is not to be executed by
this preparation record.

REASONING BUDGET 1024 ATTEMPT 006 READY FOR SEPARATE LIVE AUTHORIZATION

## Phase 1C Attempt 006 live closeout

Attempt 006 was authorized for exactly three ordered budget-1024 cases with
zero retries. The repository and canonical fingerprint gates passed, the
native prestart gate returned `EXCLUSIVE_PRESTART`, and the fresh Qwen server
loaded and announced `127.0.0.1:8080` with the frozen runtime settings.

The registered supervisor generated and transmitted the canonical
`PREFIXITY_PHASE1C_WORKFLOW_HANDOFF` metadata for Attempt 006. The child/native
poststart gate then failed closed with `EXPECTED_CHILD_PATH_MISMATCH`. This is
an infrastructure-invalid stop before readiness and inference. No readiness
check, calibration request, request artifact, response artifact, or candidate
result was produced. The server was stopped and native verification confirmed
zero llama processes and no port-8080 listener.

Attempt 006 accounting is Qwen startup `1`, supervisor launch `1`, poststart
check `1`, readiness `0`, inference `0`, requests `0`, and retries `0`. The
only persisted artifact is
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-006/supervisor.json`
with SHA-256
`c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07`.

The precise child-path representation mismatch requires offline forensic
review. Do not rerun the poststart gate, repair Attempt 006 in place, create
Attempt 007, or execute 512/256 under this authorization.

Execution record:
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_006_EXECUTION_RECORD.md`

REASONING BUDGET 1024 ATTEMPT 006 INVALID — STOP FOR REVIEW

## Phase 1C CI portability remediation before Attempt 007

The accepted pre-Attempt-007 CI diagnosis identified three pre-existing
portability failures: an un-gated Windows-only exclusivity variant on Unix,
raw line-ending-sensitive h001 source hashing on Windows, and ordinary tests
that required ignored local evidence. Remediation is isolated on branch
`agent/phase-1c-ci-portability-remediation`, rooted at canonical main
`3ca383f1760560510803008c209168267c2860f9`.

The Windows-only classification is now platform-gated without
`allow(dead_code)`. Source hashing now rejects BOM/non-UTF-8 input and
normalizes CRLF/lone CR to LF; the frozen h001 expected hash is unchanged.
Default tests use tracked deterministic inputs and portable identity/schema
checks. The checked-in command
`prefixity-phase1c-local-evidence-certification` separately validates actual
ignored Stage 1 and calibration artifacts, fails closed when they are absent,
and performs no writes or runtime contact.

Attempt 006 evidence, classifications, identity files, and sidecars remain
unchanged. Strict current-source identity checks therefore remain a deliberate
offline workflow-identity certification boundary after CI remediation.

Local fmt, full workspace tests, strict Clippy, and preserved-evidence
certification pass. The final clean-checkout Actions matrix for commit
`653fb5f66219e41191d0ba5f6398a8da0549af2b` is green in run #118 / database
`34040641843`: Windows job `101506640835`, macOS job `101506640961`, Ubuntu
job `101506641025`, and MSRV job `101506641017` all passed. Only the
non-blocking checkout Node.js deprecation annotation remains. No Attempt 007
has been prepared.

The first post-remediation Actions run passed Windows and MSRV but exposed one
remaining Unix-only Clippy diagnostic for the ungated
`ExclusivityOutcome::ExecutablePathFailed` variant. That variant and its
stringification arm are now platform-gated; the frozen Attempt 006 identities,
sidecars, evidence, and runtime accounting remain unchanged. The current
source fingerprint unit test records only the new source revision.

The final clean Actions run exposed a POSIX test-helper path defect in the
supervisor handoff test: the supervisor canonicalizes the child executable
path, so the relative `sh` path was not resolvable on a clean Unix runner. The
helper now uses `/bin/sh` and double quotes for environment expansion; the
Windows helper and production supervisor transport are unchanged.

## Phase 1C workflow-identity certification

The offline workflow-identity remediation is implemented in the current
worktree. The stable executable binding records the raw path for diagnostics,
resolved final path, file size, executable SHA-256, and Windows file identity
from `GetFileInformationByHandle`; path spelling is no longer the sole
security identity. The supervisor handoff carries supervisor and child
executable identities, and the child/native gate validates both PIDs, the
parent relationship, registered identity, evidence root, candidate budget,
and unexpected workflow absence. Different files, hashes/file IDs, parents,
stale launch identities, and missing handoffs fail closed.

Attempt-006 remains unchanged. Its preserved `supervisor.json` still hashes
to `c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07`.
Forensics record the expected `\\?\D:\Users\fleur\Prefixity\target\debug\prefixity-phase1c-reasoning-budget-calibration.exe`
representation generated by `canonicalize`, recover child PID `33848`, and
identify the native API as
`OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) + QueryFullProcessImageNameW`.
The historical raw observed path was not persisted; the same real launch chain
reproduced the native DOS spelling and matching final path, hash, size, and
file ID. The strongest-supported root cause is
`PATH_REPRESENTATION_MISMATCH`, not a wrong child executable.

The dedicated non-live identity is attempt-style `900`, not Attempt 007.
The actual checked-in supervisor launched the actual calibration child in
`workflow-identity-certification` mode and returned
`WORKFLOW_IDENTITY_CERTIFIED`. The result had supervisor PID `22464`, child
PID `22340`, child parent PID `22464`, zero llama records, zero llama startups,
zero port/model contacts, zero TCP readiness contacts, zero HTTP requests, and
zero inference requests. The deterministic record is
`docs/phase-1/PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION_V1.json` with canonical
SHA-256
`824b65a0f93e18a12e917fd49662790bdb2dce945c6e4f8f0c8b72a9c41524a4`.

Local validation completed on the final source state:

- `cargo fmt --all -- --check` passed;
- focused workflow-identity tests passed (`79/79` Phase 1C tests);
- `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings` passed;
- `cargo test --workspace --locked --offline` passed;
- `git diff --check` passed;
- clean-checkout-equivalent offline workspace validation remains portable and uses no ignored evidence in the default suite;
- no Git lock, llama process, or port-8080 owner was present at closeout.

No Attempt-007 identity or evidence root was created. No Qwen startup,
readiness, localhost contact, inference, 512/256 calibration, NO_OP,
INTERVENTION, or V3 execution occurred.

The first published candidate commit `e0df48c9265d54d7bd94a4fcf6b5a220990c5078`
was not promotion-eligible: GitHub Actions run `34047686452` / `#121` failed
Ubuntu and macOS Clippy on the non-Windows fail-closed certification branch.
The narrow cfg-portability fix was committed as replacement candidate
`ef3a56728222b28f430606f1ba1db60f2d42469d`. GitHub Actions run
`34048230074` / `#122` passed Ubuntu, Windows, macOS, and MSRV for that exact
SHA. Promotion to `main` remains the final publication gate for this task.

## Phase 1C Attempt 007 preparation gate — current task

The accepted preparation baseline was rechecked before making bounded changes:
`main` and `origin/main` both resolved to
`f311b237f6f3bbde3e4f002177d5a80ed9572afe`, with the expected remote and a
clean worktree. The accepted workflow certification and preserved Attempt 006
artifact were treated as immutable inputs.

Attempt 007 here means the independent
`phase1c-reasoning-budget-calibration` namespace. It is not the historical
P0-L6 fresh-arm Attempt 007 record, which remains closed in its own lineage.
The prepared evidence root is
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-007/`.

The frozen preparation identity is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.json`
with canonical SHA-256
`fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8` and
sidecar
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.sha256`.
The accompanying preparation record is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_PREPARATION.md`.

The identity binds the accepted baseline, calibration manifest
`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b`, frozen
request hashes, candidate budget `1024`, fixed case order, maximum three
requests, zero retries, fresh-runtime requirement, the preserved Attempt 006
supervisor hash `c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07`,
and the accepted certification manifest hash
`824b65a0f93e18a12e917fd49662790bdb2dce945c6e4f8f0c8b72a9c41524a4`. The
certification identity remains `900`, distinct from Attempt 007.

The current executable and source fingerprints, exact future server command,
and exact future supervisor command are recorded in the identity and
preparation record. The future commands are `NOT_EXECUTED`. Offline
`attempt-007-fingerprint`, non-contact `attempt-007-preflight`, and
`attempt-007-dry-run` checks passed. The focused Phase 1C suite passed with
`82 passed; 0 failed; 33 filtered out`, and the offline locked build passed.
Strict Clippy, the full offline workspace test suite, formatting check, and
`git diff --check` also passed.
Preflight found no llama process, no port-8080 listener, and no competing
workflow process; it performed no TCP connection or readiness probe.

The preparation contact count is zero for model/server startups, port-8080
contacts, TCP readiness, HTTP, inference, Attempt 007 execution, and retries.
The Attempt 007 root and result artifacts remain absent. Final classification:

```text
PHASE_1C_ATTEMPT_007_PREPARATION_ACCEPTED
ATTEMPT_007_PREPARED
ATTEMPT_007_NOT_EXECUTED
```

## Phase 1C Attempt 007 live execution closeout

The separately authorized live boundary was crossed exactly once from the
accepted baseline `c3059a1f2933ce0a4c9107ae03ec634b55c3fd69`. The prestart gate
was virgin and exclusive, with `EXCLUSIVE_PRESTART`; the exact frozen server
was started once, readiness passed once, and the exact certified supervisor
completed one child launch. The frozen cases were dispatched in order:
`rbcal-001`, `rbcal-002`, `rbcal-003`. There were three inference requests,
zero retries, zero fallbacks, zero adaptive replicates, and no fourth request.

The raw Attempt 007 evidence is preserved at
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-007/`
and is inventoried in
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_EXECUTION_RECORD.md`.
The candidate result is protocol state `FAIL`: `rbcal-001` passed, while
`rbcal-002` and `rbcal-003` returned HTTP 200 with `finish_reason: length` and
failed structural validation. The protocol points to `next_budget: 512`, but
512 and 256 were not run and remain unauthorized.

The supervisor and child passed the repository's runtime handoff,
parent/child, and poststart ownership checks. However, the executable objects
recorded at runtime did not match the supervisor and child hashes/file IDs
bound into the accepted preparation identity. The existing validator records
the live objects but does not compare those cached preparation-time target
binary fields. This is a material execution-integrity deviation, not a source
or frozen-identity change; no attempt was made to repair or rerun it.

The exact terminal classification is:

```text
PHASE_1C_ATTEMPT_007_EXECUTION_INVALID - STOP FOR REVIEW
REASON: FROZEN_EXECUTABLE_IDENTITY_MISMATCH
ATTEMPT_007_EXECUTED_ONCE
```

The launched server was shut down. Post-run checks found no llama/Qwen
process, supervisor, child, port-8080 listener, stale handoff, or Attempt 007
execution lock. Attempt 006 remains unchanged at preserved SHA
`c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07`.

Evidence, cleanup, validation, and the bounded publication are recorded in
the execution record. No wider Phase 1C conclusion or admissible calibration
disposition is supported by this invalid execution. No live rerun is
authorized.

## Phase 1C Attempt 007 executable-binding remediation

The preceding preparation-only status is historical and is superseded by the
live closeout immediately above: Attempt 007 crossed its live boundary exactly
once, produced the preserved raw evidence, and is permanently consumed.

The accepted remediation baseline was reverified at
1f036e1c6f0afb0db23e3031069d37511a401aae with main, origin/main, and a clean
worktree. Attempt-006 and Attempt-007 evidence remained byte-for-byte
preserved. The executable forensic record is
docs/phase-1/PHASE_1C_ATTEMPT_007_EXECUTABLE_BINDING_REMEDIATION.md.

The control defect was that preparation-bound supervisor and child executable
objects were recorded but never compared with the runtime handoff objects.
Runtime PID, parent, process-image, and path-representation checks were
internally consistent, so the changed target/debug objects could pass the
existing gate and reach the inference-capable calibration path. The
defensible root-cause statement is mutable target-directory replacement or
relinking after preparation; the exact writing command, compiler, and
timestamp are not durably recoverable.

The remediation adds a complete frozen supervisor/child object binding using
SHA-256, file size, Windows file identity, and final-path semantics where
needed. The supervisor checks both objects before child spawn. The child
rechecks both objects before post-start ownership and inference. Missing
preparation identity fails closed; path spelling alone is never a fallback.
The freeze_copy staging primitive refuses replacement of a staged destination,
and future preparation must bind the staged copies after all build/test/Clippy
work.

The preserved Attempt-007 mismatch fixture is
fixtures/phase1c/attempt-007-frozen-executable-mismatch.json. It classifies
the known prepared/runtime relationship as
FROZEN_EXECUTABLE_IDENTITY_MISMATCH without interpreting calibration output.
The historical certification 900 and its canonical hash remain immutable; the
corrected offline certification namespace is 900-v2.

The final-source Windows 900-v2 certification completed as
WORKFLOW_IDENTITY_CERTIFIED using bounded staged copies. The staged supervisor
was SHA-256 72f4a64c9b7d852cad95b8c8811bcb083eefa8281b29cf796ed94395c65ae3f9,
size 1259008, file ID volume=ba2f80f4;index=00110000002f2acc. The staged child
was SHA-256 0f1a21bc97b6edf7737ba270af35f3d55658559587b353a982ffb5dd3773ae1,
size 8525824, file ID volume=ba2f80f4;index=000b0000002f2ae4. Supervisor PID
19208 and child PID 20056 were observed, with native parent PID validation.
The generated identity canonical SHA-256 was
acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194.
Substitution of the mutable target/debug child was rejected before spawn as
FROZEN_EXECUTABLE_IDENTITY_MISMATCH and wrote no evidence file.

Attempt state is now explicit:

    PREPARED = true
    EXECUTED = true
    INTEGRITY_ACCEPTED = false
    CALIBRATION_ADMISSIBLE = false
    REUSABLE = false

The raw candidate FAIL and next_budget 512 are inadmissible and do not change
budget-selection state. The protocol permits NEXT_FRESH_ATTEMPT_ID = 008, but
no Attempt-008 identity was created or prepared.

Remediation accounting remains:

    model_server_startups=0
    port_8080_contacts=0
    tcp_readiness_contacts=0
    http_model_requests=0
    inference_requests=0
    attempt_007_executions_added=0
    attempt_008_executions=0

The final classification for this task is
PHASE_1C_FROZEN_EXECUTABLE_BINDING_REMEDIATION_ACCEPTED,
ATTEMPT_007_CALIBRATION_INADMISSIBLE,
ATTEMPT_007_PERMANENTLY_CONSUMED,
ATTEMPT_008_NOT_PREPARED.

## Phase 1C Attempt 008 preparation

The accepted Attempt-007 remediation baseline was reverified at
417673c06b141e07d54ad0bc101e1105e1a734de with `HEAD`, `main`, and
`origin/main` equal and a clean worktree. Attempt 006 evidence remains
unchanged at `c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07`.
Attempt 007 remains canonically frozen at
`fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8`, with
`EXECUTED=true`, `INTEGRITY_ACCEPTED=false`,
`CALIBRATION_ADMISSIBLE=false`, `REUSABLE=false`, and permanently consumed.

The frozen protocol permits a replacement attempt after an integrity-invalid
consumed attempt. The last admissible candidate before Attempt 007 was 1024;
the raw Attempt-007 `next_budget=512` remains preserved but excluded from
accepted calibration-state progression. The deterministic provenance fixture
is `fixtures/phase1c/attempt-008-budget-provenance.json`.

Attempt 008 is eligible and was prepared with candidate budget 1024:

    ATTEMPT_008_ELIGIBLE
    ATTEMPT_008_PREPARED
    ATTEMPT_008_NOT_EXECUTED
    FROZEN_EXECUTABLE_IDENTITY_BOUND

The preparation identity is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.json` with
canonical SHA-256
`917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5` and sidecar
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.sha256`.
It binds the accepted baseline, unchanged manifest and request hashes, the
complete 1024 runtime contract, accepted 900-v2 workflow certification
`acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194`, and the
fresh Attempt-008 frozen supervisor/child objects.

The final build source commit before freezing was
`ddc05be5951e3cc3da36a659f15303f9760d952a`. The frozen supervisor is
`f07a9302786f3abe5a01eb9fa9eda85f50de9fe9f3d34feba14ba0b2e09c2088`, size
1259008, file ID `volume=ba2f80f4;index=0007000000387abb`. The frozen child is
`0b859faae14dbcf5dad65612b646967fd79977173d67e5b2dabb570b861e5230`, size
8657920, file ID `volume=ba2f80f4;index=000c00000038d6f2`. Both are in the
bounded non-overwriting `target/phase1c-attempt-008-frozen/` stage; mutable
`target/debug` objects are not authorized execution objects.

Offline fingerprint, virgin preflight, dry run, preparation validation, exact
frozen-chain certification, and substitution tests passed. Correct frozen
objects were certified with native parent/child validation. Wrong supervisor,
wrong child, and mutable-path same-content substitution were rejected before
spawn as `FROZEN_EXECUTABLE_IDENTITY_MISMATCH`.

No Attempt-008 execution root, request ledger, retry record, lock, or live
result exists. The future model-server and frozen-supervisor commands are
recorded as `NOT_EXECUTED` in the preparation record
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_PREPARATION.md`.

Preparation accounting remains:

    model_server_startups=0
    port_8080_contacts=0
    tcp_readiness_contacts=0
    http_model_requests=0
    inference_requests=0
    attempt_007_executions_added=0
    attempt_008_executions=0

## Phase 1C Attempt 008 live execution closeout

The separately authorized Attempt 008 live boundary was crossed exactly once
from the published preparation baseline
`b7f756d2d9ba538736bf5ef9765a5a2112fb3d31`. The identity, virgin, native
prestart, server-configuration, frozen-object, and v2 certification gates
passed. The exact accepted llama command was verified on PID `4608`, and the
native process/TCP ownership checks found one expected server and no competing
Prefixity workflow.

The exact frozen supervisor and child executed once. Supervisor PID `120`
launched child PID `6312`; the recorded child parent PID was `120`, both
runtime executable objects matched the preparation SHA/size/file IDs, the
child exited `0`, and the supervisor completed. Readiness used one permitted
TCP listener connect. The ordered request ledger was exactly
`rbcal-001`, `rbcal-002`, `rbcal-003`, with three HTTP 200 inference requests,
zero retries, zero fallbacks, zero adaptive replicates, and no fourth request.

The raw candidate result is `FAIL`: `rbcal-001` passed, while `rbcal-002` and
`rbcal-003` reached `finish_reason=length` and failed structural validation.
The independently verified execution-integrity gate passed:

```text
ATTEMPT_008_INTEGRITY_ACCEPTED
ATTEMPT_008_CALIBRATION_ADMISSIBLE
```

The frozen protocol disposition is `next_budget=512`. That value is recorded
only as the result of Attempt 008; no 512 execution was authorized or started.

Raw evidence remains in the ignored root
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-008/`.
Its 23-file inventory was frozen before interpretation and matched the
deterministic evidence manifest and sidecar. The manifest SHA-256 is
`f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779`.
The tracked execution record is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_EXECUTION_RECORD.md`.

After evidence freeze, the exact server PID `4608` was terminated. Post-run
checks found no server, no port-8080 listener, no supervisor, no child, no
stale handoff, and no execution lock. Attempt 006 and Attempt 007 historical
evidence remain unchanged. No later budget or live attempt was started.

Execution accounting:

```text
model_server_startups=1
readiness_contacts=1
http_model_requests=3
inference_requests=3
automatic_retries=0
fallback_requests=0
adaptive_replicates=0
attempt_008_executions=1
```

Final classification:

```text
PHASE_1C_ATTEMPT_008_EXECUTION_ACCEPTED
ATTEMPT_008_EXECUTED_ONCE
ATTEMPT_008_INTEGRITY_ACCEPTED
ATTEMPT_008_CALIBRATION_ADMISSIBLE
CALIBRATION_DISPOSITION_NEXT_BUDGET_512_NOT_EXECUTED
```

## Phase 1C Attempt 009 preparation gate

Attempt 009 was eligible only through the independently revalidated,
integrity-accepted and calibration-admissible Attempt-008 result:

```text
SOURCE_ATTEMPT_008
SOURCE_BUDGET_1024
SOURCE_STATE_FAIL
NEXT_BUDGET_512
ATTEMPT_007_EXCLUDED_FROM_SELECTION
ATTEMPT_009_ELIGIBLE
```

Attempt-008 identity SHA-256 is
`917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5`; its
evidence-manifest SHA-256 is
`f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779`. The
offline predecessor audit verified the recorded FAIL, complete three-case
set, exactly three requests, zero retries/fallback/adaptive replicates, the
Attempt-008 supervisor handoff, and all 23 raw evidence file hashes.

The Attempt-009 preparation identity is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.json`,
canonical SHA-256
`0929aae1d371415d3efd92e4e7310490ad06fb69fd0398aa76aa9d857818f812`, with
sidecar
`docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.sha256`.
It binds the unchanged manifest and request hashes, candidate budget 512,
the Attempt-008 provenance fixture, accepted v2 workflow certification
`acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194`, and the
complete frozen executable objects.

The final source build commit before freezing was
`5918141941132fc444ce34a039574974bd363753`. The bounded stage is
`target/phase1c-attempt-009-frozen/`:

```text
supervisor SHA: aaba6a8202ccc88c4ea6c277b5c20058588cabdd98c5e5d101756e40e33aa2fb
supervisor size: 1259008
supervisor file ID: volume=ba2f80f4;index=000500000038291e
child SHA: c5c4913d1b4719b333d972ecfff11530fadbf24aaac99fde71f5a5bb8f4e1beb
child size: 8813056
child file ID: volume=ba2f80f4;index=000a000000382aae
```

Both frozen objects passed exact object binding; mutable `target/debug`
objects are forbidden. Offline fingerprint, preflight, dry-run, preparation
validation, Attempt-008 raw-evidence revalidation, v2 certification, and the
existing pre-spawn substitution rejection tests passed. No Attempt-009
evidence root or result artifact was created.

The future 512 model-server command and frozen supervisor invocation are
recorded as `NOT_EXECUTED` in
`docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_PREPARATION.md`.

Preparation accounting remains:

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_008_executions_added=0
attempt_009_executions=0
```

Final preparation classification:

```text
PHASE_1C_ATTEMPT_009_PREPARATION_ACCEPTED
ATTEMPT_009_PREPARED
ATTEMPT_009_NOT_EXECUTED
CANDIDATE_BUDGET_512
FROZEN_EXECUTABLE_IDENTITY_BOUND
```

## Phase 1C Attempt 009 live execution closeout

The separately authorized Attempt-009 live boundary was crossed exactly once
from the promoted preparation baseline
`d3895008932327ed625701ac86c3efb9fbfd34f0`. The virgin and exclusive
prestart gates passed. The exact operator-started llama server matched the
512 contract, and the exact frozen supervisor launched the exact frozen child
once with matching executable identities.

The child failed before readiness and inference with
`candidate order is not satisfied: prior candidate result is absent`. The
failure indicates a harness validation defect: the Attempt-009 candidate-order
precondition does not resolve the accepted Attempt-008 result in its default
path. No readiness contact, HTTP request, inference, request ledger, response,
or calibration result was produced. The run is consumed and must not be
rerun; source remediation is a separate task.

The bounded raw record is
`docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_EXECUTION_RECORD.md`.
The raw supervisor evidence remains at
`experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-009/`.
Its supervisor record SHA-256 is
`83d4fee4203672881f9ca8e26d7440c3fccae19f67315eb0352e1fc74bf8e077`; the
deterministic one-file evidence manifest SHA-256 is
`3f38b73f46800c1682f8e5dcc2f23ccc6788016fa48e00af86c6982810d3cc27`, with a
valid sidecar.

Runtime accounting for this task is:

```text
model_server_startups=1
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_008_executions_added=0
attempt_009_executions=1
automatic_retries=0
fallback_requests=0
adaptive_replicates=0
```

Cleanup completed: the exact server exited, port 8080 is clear, and no
supervisor, child, stale handoff, or execution lock remains. Final
classification:

```text
ATTEMPT_009_EXECUTED_ONCE
ATTEMPT_009_INTEGRITY_REJECTED
ATTEMPT_009_CALIBRATION_INADMISSIBLE
REASON: CHILD_FAILED_BEFORE_READINESS
```

## Phase 1C Attempt 009 candidate-order gate remediation

The consumed Attempt009 failure was forensically classified as a
preparation/runtime semantic disagreement. Preparation accepted the tracked
Attempt008-admissible transition, while the runtime candidate-order validator
looked for the unrelated generic prior-candidate path
`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024/candidate-result.json`.
The accepted Attempt008 result is represented by the tracked transition in
`fixtures/phase1c/attempt-009-budget-provenance.json`; ignored raw evidence is
not a clean-checkout dependency.

The remediation uses one validated loader for both Attempt009 preparation and
the runtime candidate-order gate. The new offline
`attempt-009-candidate-order` command proves the runtime dependency,
predecessor transition, and 512 candidate order without requiring a listener,
model server, virgin Attempt009 state, or ignored raw evidence. Focused tests
cover the valid transition, missing predecessor, Attempt007 exclusion,
Attempt008 precedence, preparation/runtime parity, clean-checkout behavior,
and the consumed Attempt009 regression.

State is preserved: Attempt008 remains the accepted `FAIL` at budget 1024 with
`next_budget=512`; Attempt009 remains consumed, integrity-rejected,
calibration-inadmissible, and non-reusable. **Attempt 009 is not evidence about
reasoning budget 512. Budget 512 remains untested.** The next fresh protocol
state is Attempt010 at budget 512, but Attempt010 is not prepared.

Remediation accounting remains:

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_009_executions_added=0
attempt_010_executions=0
```

Review follow-up: the shared loader's emitted Attempt008 evidence-manifest
hash was truncated (63 hex characters) and is corrected to the canonical
value; regressions now bind emitted hashes to the tracked Attempt009 identity
lineage, reject non-selected budgets, and reject corrupt or missing transition
fields. Documentation now records the original runtime lookup as the
prior-candidate `budget-1024` path and closes the preceding Attempt009 code
fence. Local offline validation before publication: `cargo fmt --check`,
`cargo clippy -D warnings`, `cargo test --workspace` (controlled-benchmark
library: 134 passed, 0 failed), `cargo build --bins`, and `git diff --check`
all passed on Rust 1.97.1.

## Phase 1C Attempt 010 budget-512 preparation

Attempt 010 was prepared at budget 512 from accepted baseline
`a0019bcd96ca8759801a879619f119032067a3f9`. The code commit
`cd7145319555d6f00c979e6e333fc66c921fa335` adds the Attempt-010 path. Its
live entry point and the offline `attempt-010-live-prerequisites` command
share one prerequisite function, which includes the pre-listener checks
extracted unchanged from the calibration runtime. After fmt, clippy, the
full workspace tests (controlled-benchmark library 140 passed, 0 failed), and
the bins build passed on Rust 1.97.1, the supervisor
(`3e43fddd…288e`, 1259008 bytes, `volume=ba2f80f4;index=00080000003b6e6d`)
and child (`e65a626e…61c9`, 9010176 bytes,
`volume=ba2f80f4;index=0008000000454e40`) were frozen into
`target/phase1c-attempt-010-frozen/` without overwrite.

The identity canonical SHA-256 is
`9292e9ecdd2e89f695dfb34bc782ade41b70412c427807c6c3a5653b50ec16f7`, computed
independently by the generator and by the frozen child. Fingerprint,
repository contract, preflight (`EXCLUSIVE_PRESTART`), dry run, and
preparation validation passed with the frozen child. The real frozen
supervisor launched the frozen child once for the prerequisite traversal,
which reached `READY_FOR_MODEL_READINESS_BOUNDARY` with
`RUNTIME_DEPENDENCIES_COMPLETE`, `PREDECESSOR_TRANSITION_VALID`, and
`CANDIDATE_ORDER_VALID`, and stopped before post-start ownership inspection,
TCP readiness, HTTP, and inference. Mutable child, mutable supervisor, and
same-content different-object substitutions were rejected before spawn as
`FROZEN_EXECUTABLE_IDENTITY_MISMATCH`. A clean-checkout integration test runs
the repository contract in CI.

State is preserved: Attempt 008 remains the last admissible result (1024,
`FAIL`, next 512); Attempt 009 remains consumed and inadmissible with zero
inference. **Budget 512 remains untested. Attempt 010 is prepared, not
executed.** The budget-256 generic predecessor lookup and a misplaced
Attempt-009 check inside the historical Attempt-008 identity validator are
recorded as deferred, non-blocking issues in the preparation record.

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_009_executions_added=0
attempt_010_executions=0
```

## Phase 1C Attempt 010 budget-512 execution

Attempt 010 was executed exactly once from promoted preparation `87cbacd`
against an operator-started server (PID 18048) whose command line,
executable identity, exclusivity, and sole port-8080 ownership were verified
read-only beforehand. The frozen supervisor (PID 11740) launched the frozen
child (PID 21240) once; the child passed the shared live prerequisites,
recorded `EXCLUSIVE_POSTSTART`, made one passed readiness check, and issued
exactly three requests with the frozen hashes.

```text
ATTEMPT_010_EXECUTED_ONCE
ATTEMPT_010_INTEGRITY_ACCEPTED
ATTEMPT_010_CALIBRATION_ADMISSIBLE
```

Admissible raw outcome at budget 512: rbcal-001 FAIL (`length`, 2048
tokens), rbcal-002 PASS (`stop`, 547), rbcal-003 FAIL (`stop`, 554, structural
schema). Candidate state `FAIL`, case set complete, selected budget null,
protocol `next_budget=256`. Evidence manifest
`5673e55b381d1f5171c38bc9f1721b3105adf829ece4ec029cbcca4db150c4b9` verifies all
23 raw files. The verified server PID was terminated after evidence freeze;
no server, listener, workflow process, handoff, or lock remains.

Budget 256 is not prepared or authorized. The deferred budget-256
candidate-order defect must be remediated and validated before any 256
attempt is prepared. Full record:
`docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_EXECUTION_RECORD.md`.

```text
model_server_startups_by_agent=0
readiness_contacts=1
http_model_requests=3
inference_requests=3
automatic_retries=0
fallback_requests=0
adaptive_replicates=0
attempt_009_executions_added=0
attempt_010_executions=1
```

## Phase 1C authoritative candidate-transition generalization

The recorded budget-256 candidate-order defect is remediated. Candidate
order for every non-initial budget now resolves through one tracked registry,
`fixtures/phase1c/calibration-candidate-transitions.json`, via
`resolve_authoritative_candidate_transition`; the 512 special case and the
generic ignored `budget-512/candidate-result.json` lookup are removed.

```text
ATTEMPT_008_TO_512_VALID      (1024 FAIL -> 512)
ATTEMPT_010_TO_256_VALID      (512 FAIL -> 256)
ATTEMPT_007_EXCLUDED
ATTEMPT_009_EXCLUDED
CANDIDATE_512_ORDER_VALID
CANDIDATE_256_ORDER_VALID
```

Each transition binds the source identity hash, evidence-manifest hash, and
execution-record hash and is cross-checked against those tracked files.
Missing, corrupt, inadmissible, wrong-source, wrong-budget, wrong-state,
wrong-successor, mismatched-hash, raw-path, duplicate, and ambiguous
transitions are rejected without reconstruction. Preparation and live
candidate order share the resolver through `calibration_prestart_checks`.
Clean-checkout integration tests establish 512 and 256 order from tracked
evidence only.

The consumed Attempt-010 identity now fails closed on its source binding, as
intended after the source change; its document check remains. The historical
Attempt-008 validator defect is not on the resolver path and stays deferred.
No evidence was modified. Attempt 011 at budget 256 is eligible but not
prepared. Record:
`docs/phase-1/PHASE_1C_AUTHORITATIVE_CANDIDATE_TRANSITIONS.md`.

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_010_executions_added=0
attempt_011_executions=0
```

## Phase 1C Attempt 011 budget-256 preparation

Attempt 011 was prepared at budget 256 from accepted baseline
`292dd1f0951f20fdcf6f49facbb446076d80a135`. Code commit
`4dd619e82a4d494f0ee8ca954440d46bb30f300e` adds the Attempt-011 path, whose
live entry point and offline `attempt-011-live-prerequisites` command share
one prerequisite function including `calibration_prestart_checks(256)`.
After fmt, clippy, the full workspace tests (controlled-benchmark library 150
passed, 0 failed), and the bins build passed, the supervisor (`9dbf6dff…b8b1`,
1259008 bytes, `volume=ba2f80f4;index=0006000000387bbc`) and child
(`10533899…15d5`, 9281536 bytes, `volume=ba2f80f4;index=0005000000387bbd`)
were frozen into `target/phase1c-attempt-011-frozen/` without overwrite.

The identity canonical SHA-256 is
`60b442a1b09a4ff817b1ca40c103853f6972baeb3d13c91f551cb58ff5bd03ed`, computed
independently by the generator and the frozen child. Candidate 256 resolves
only through the authoritative transition registry to Attempt 010 (512 FAIL
-> 256, manifest `5673e55b…c4b9`); 007 and 009 are excluded and 008 is not a
source. Fingerprint, repository contract, preflight (`EXCLUSIVE_PRESTART`),
dry run, and preparation validation passed; the real frozen supervisor ->
frozen child traversal reached `READY_FOR_MODEL_READINESS_BOUNDARY` with
runtime/contract parity; mutable, same-content, and Attempt-010 substitutions
were rejected before spawn. A clean-checkout integration test runs the
Attempt-011 contract in CI.

**Budget 256 remains untested. Attempt 011 is prepared, not executed.** The
historical Attempt-008 validator defect is not on the Attempt-011 path and
stays deferred. Record:
`docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_PREPARATION.md`.

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_010_executions_added=0
attempt_011_executions=0
```

## Phase 1C Attempt 011 budget-256 execution — terminal calibration result

Attempt 011 was executed exactly once from promoted preparation `427cfc9`
(source provenance `4dd619e`, identity `60b442a1…03ed`) against an
operator-started server (PID 6880) whose command line, executable identity,
exclusivity, and sole port-8080 ownership were verified read-only
beforehand. The frozen supervisor (PID 6220) launched the frozen child (PID
29544) once; the child passed the shared live prerequisites, recorded
`EXCLUSIVE_POSTSTART`, made one passed readiness check, and issued exactly
three requests with the frozen hashes.

```text
ATTEMPT_011_EXECUTED_ONCE
ATTEMPT_011_INTEGRITY_ACCEPTED
ATTEMPT_011_CALIBRATION_ADMISSIBLE
```

Admissible outcome at budget 256: rbcal-001 FAIL (`stop`, 359 tokens,
structural/schema), rbcal-002 FAIL (`length`, 2048 tokens, token
exhaustion), rbcal-003 FAIL (`stop`, 298 tokens, structural/schema).
Candidate state `FAIL`, case set complete, selected budget null, next budget
null. Budget 256 is the last registered candidate, so the frozen protocol's
terminal classification applies:

```text
REASONING-ON / 2048-TOKEN SCORED CONFIGURATION NOT FEASIBLE
```

The claim is bounded to `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, the three
registered calibration cases, this reasoning-on configuration, the 2048-token
output ceiling, and the frozen protocol. Evidence manifest
`5e8ccbc7be4fe5cb9b6d4041a3fc4c47943d5a889f0a4916ebab4e8fbebf73cd` verifies all
23 raw files. The verified server PID was terminated after evidence sealing.
The transition registry is unchanged (it has no terminal-state schema).

```text
LAST_ADMISSIBLE_ATTEMPT   = 011
LAST_ADMISSIBLE_BUDGET    = 256
LAST_ADMISSIBLE_STATE     = FAIL
CALIBRATION_TERMINAL      = true
SELECTED_REASONING_BUDGET = null
NEXT_CANDIDATE_BUDGET     = null
ATTEMPT_011_CONSUMED      = true

operator_model_server_startups=1
agent_model_server_startups=0
readiness_contacts=1
http_model_requests=3
inference_requests=3
retry_requests=0
fallback_requests=0
adaptive_replicates=0
warmup_requests=0
attempt_011_executions=1
```

Record: `docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_EXECUTION_RECORD.md`.

## Phase 1C scored-runtime V3 design decision

Design only; no code, inference, model startup, or evidence change. After the
terminal reasoning-budget calibration, the V3 decision
(`docs/phase-1/PHASE_1C_SCORED_RUNTIME_V3_DESIGN_DECISION.md`) is
`PHASE_1C_V3_DESIGN_RECOMMENDED`, framed as a **final bounded
feasibility/falsification gate for the existing Qwen3.5-0.8B instrument**,
not an expected fix. Offline re-reading of the calibration evidence shows two
independent failure classes: runaway repetitive final output after bounded
reasoning (every `length` failure) and structural-format failure on normal
termination (including code-fenced JSON).

V3 changes only the output ceiling (2048 -> 4096) and the request and
supervisor deadlines (1200000 -> 2400000 ms; 1320000 -> 2520000 ms). Unrestricted
reasoning is operationally `reasoning = on`, `reasoning_budget_flag = ABSENT`,
`max_tokens = 4096`, `context = 8192`, bound to a frozen llama.cpp executable
identity and a hashed GGUF model file. Preparation must prove
`projected_prompt_tokens + 4096 <= 8192` with the frozen tokenizer for every
statically determined pilot and full-cohort request. Before every live
request, the runner tokenizes the exact request and requires
`exact_prompt_tokens + 4096 <= 8192`; on failure it does not dispatch, records
`INCONCLUSIVE_CONTEXT_BOUND`, and never truncates, summarizes, drops history,
reduces `max_tokens`, increases context, or retries. In the feasibility gate
this closes the Qwen path; in a later V3 pilot it is an inconclusive
observation counted toward the >10% pause rule and never reclassifies the
gate. A control arm that becomes context-bound while INTERVENTION fits is
recorded only as descriptive mechanism evidence, not as preserved task
success or a scored win.

The gate reuses the three frozen `rbcal` cases and the unchanged calibration
predicate on one fresh server, with at most three requests and no retries. All
three passing authorizes only V3 pilot preparation. Any `length`, timeout,
inconclusive, structural failure, or failed input proof classifies
`CURRENT_QWEN_SCORED_PATH_CLOSED`: no second ceiling, no further Qwen
remediation, and the next permitted design review is for a different capable
model/runtime. A genuine zero-inference integrity failure may receive one
replacement gate identity. The runner must be driven by one compact spec with
generic validators; no new hand-cloned attempt-validator family.

V2 remains frozen and valid, V3 is a new lineage starting every arm fresh
from BASELINE, and no V1/V2/calibration observation enters the V3 comparison.
The stale `docs/INDEX.md` Stage 1 status was corrected. The next task is
offline V3 preparation; no inference is authorized.

## Phase 1C V3 feasibility-gate preparation

```text
PHASE_1C_V3_FEASIBILITY_PREPARATION_ACCEPTED
V3_FEASIBILITY_GATE_PREPARED
V3_FEASIBILITY_GATE_NOT_EXECUTED
CURRENT_QWEN_SCORED_PATH_OPEN_PENDING_GATE
NO_SCORED_INFERENCE_AUTHORIZED
```

Offline preparation only; record:
`docs/phase-1/PHASE_1C_V3_FEASIBILITY_GATE_PREPARATION.md`. No model server
was started, port 8080 was not contacted, and no readiness, token-count, or
inference request was made.

Work completed:

- Source provenance `b255606528388dda6bc098f7cb07a5e22d033b2b`: compact
  `V3GateSpec` runner; direct `-m` GGUF loading with `--offline` and no
  `LLAMA_ARG_*`; `/v1/chat/completions/input_tokens` as authoritative counter;
  reasoning on, budget flag absent, `max_tokens` 4096, context 8192.
- Design Amendment 2: supervisor deadline derived from the spec,
  `1*1000 + 3*60000 + 3*2400000 + 120000 = 7501000` ms; token counting no
  longer inherits the generation bound; an independent supervisor timeout is
  rejected; historical identities keep 1320000 ms.
- Design Amendment 3: `HISTORICAL_LLAMA_FILE_ID_RECORDING_DEFECT`. The helper
  is correct and cwd-independent; the V3 llama.exe identity is
  `volume=c4c93b54;index=00060000001ea970`. Attempts 009-011 unchanged.
- `V3_BOUND_SOURCES` has seven sources including
  `phase1c_executable_identity.rs`. The consumed Attempt-011 repository
  contract now fails closed on its source binding.
- Frozen supervisor `2cdac8ec...` and child `1b0342b2...` in
  `target/phase1c-v3-feasibility-frozen/`; gate identity V1 canonical SHA-256
  `65958c75ca0b4cd8c7223b1e5382222fa182e37df73bc3a349fc219c514d242a`.

Validation performed: fmt, clippy (`-D warnings`), workspace tests, bin build
and MSRV check pass at the provenance commit (one timing flake of the
pre-existing 500 ms handoff test on the first run; two further full runs
607/607). Frozen `v3-dry-run`, `v3-preflight`, and frozen supervisor -> child
`v3-live-prerequisites` (`READY_FOR_MODEL_READINESS_BOUNDARY`, applied
deadline 7501000 ms) all stopped before readiness with zero contacts. Mutable,
same-content-copied, and historical supervisor/child substitutions and
tampered llama.exe/GGUF identities were rejected before inference.

Remaining: the live gate needs separate operator authorization, a fresh
server started with the recorded command, and the recorded frozen supervisor
command. Post-start ownership was not exercised offline. `V3_FEASIBILITY_PASSED`
is not claimed; no pilot preparation, ceiling change, or reasoning budget is
authorized.
