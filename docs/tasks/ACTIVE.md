# Active Task — Phase 0 Foundation Slice (P0-L13)

Status: P0-L6C-R1 repair complete; P0-L6C Attempt 002 is preserved as
runtime-blocked / failed before inference, Attempt 003 is readiness-blocked
before inference, Attempt 004 is readiness-blocked before preflight, Attempt
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
`ceb145320517a84a8cad1df7695c5f1138e75a93bce2d53ae1ec4a18a37fe409`

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
