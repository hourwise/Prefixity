# Prefixity Knowledge Index

> Navigation map only. Use the linked documents for project explanation and
> evidence.

## Read first

| Path | Contains | Read when |
| --- | --- | --- |
| [`phase-0/CONTEXT_STABILITY.md`](phase-0/CONTEXT_STABILITY.md) | P0-L10 deterministic context stability, lifecycle, boundary, inversion and leading-region analysis. | Classifying context without automatically optimizing it. |
| [`phase-0/CONTEXT_LAYOUT_PLANNER.md`](phase-0/CONTEXT_LAYOUT_PLANNER.md) | P0-L11 bounded, constraint-first context-layout candidate planning. | Reviewing proposal-only safe ordering candidates. |
| [`phase-0/CANDIDATE_EVALUATION.md`](phase-0/CANDIDATE_EVALUATION.md) | P0-L12 candidate evidence ladder, capability gate and experiment-readiness boundary. | Evaluating what a layout candidate currently justifies. |
| [`phase-0/CANDIDATE_MATERIALIZATION.md`](phase-0/CANDIDATE_MATERIALIZATION.md) | P0-L13 inert candidate materialization, conservation checks, safety certificate, and experiment-pair boundary. | Materializing a candidate for controlled experimentation. |
| [`phase-0/LIVE_EXPERIMENT_HARNESS_PREPARATION.md`](phase-0/LIVE_EXPERIMENT_HARNESS_PREPARATION.md) | P0-L6A loopback-only transport, explicit opt-in, preflight/readiness, fixed sequence, and bounded evidence preparation. | Reviewing live-experiment preparation without authorizing a run. |
| [`phase-0/PAIRED_MUTATION_EXPERIMENT_PREPARATION.md`](phase-0/PAIRED_MUTATION_EXPERIMENT_PREPARATION.md) | P0-L6B paired vanilla/candidate mutation preparation, independent P0-L13 certification, five-case sequence, and no-direction outcome contract. | Reviewing the paired mutation hypothesis without authorizing a run. |
| [`phase-0/FRESH_ARM_PAIRED_MUTATION_EXPERIMENT.md`](phase-0/FRESH_ARM_PAIRED_MUTATION_EXPERIMENT.md) | P0-L6E two-independent-epoch fresh-server arm design, arm-local durability, semantic identity, and offline aggregation. | Preparing a deconfounded paired experiment without authorizing a live attempt. |
| [`SOURCE_OF_TRUTH.md`](SOURCE_OF_TRUTH.md) | Accepted product definition, implemented state, constraints and uncertainties. | Every substantial task. |
| [`tasks/ACTIVE.md`](tasks/ACTIVE.md) | Current task, validation status and next-task recommendation. | Every task. |
| [`../README.md`](../README.md) | User-facing scope, commands, repository layout and safety summary. | Orienting to the project or running the CLI. |
| [`RESEARCH.md`](RESEARCH.md) | Hypothesis, evidence, prior art, provider dependencies and open questions. | Research, validation or product-direction work. |

## Specifications and decisions

| Path | Contains | Read when |
| --- | --- | --- |
| [`PROJECT_CHARTER.md`](PROJECT_CHARTER.md) | Purpose, Phase 0 boundaries and source-of-truth principles. | Checking scope or non-goals. |
| [`phase-0/TRACE_FORMAT.md`](phase-0/TRACE_FORMAT.md) | Normative trace v2, usage schemas and profile format. | Changing or consuming trace data. |
| [`phase-0/ADR-001-LOCAL-FIRST-PRODUCT-BOUNDARY.md`](phase-0/ADR-001-LOCAL-FIRST-PRODUCT-BOUNDARY.md) | Accepted local-first product boundary and runtime-adapter scope. | Product or architecture boundary work. |
| [`phase-0/OBSERVATION_SCHEMAS.md`](phase-0/OBSERVATION_SCHEMAS.md) | Versioned neutral artifact, observation and capability contracts. | Changing or consuming the P0 observation vocabulary. |
| [`phase-0/CONFORMANCE_HARNESS.md`](phase-0/CONFORMANCE_HARNESS.md) | P0-L4 neutral cache-conformance experiment, mutation, runner and result foundation. | Designing controlled cache-behaviour experiments. |
| [`phase-0/LLAMA_CPP_CONFORMANCE_ADAPTER.md`](phase-0/LLAMA_CPP_CONFORMANCE_ADAPTER.md) | P0-L5 llama.cpp request projection, fake transport boundary and response observer. | Reviewing the llama.cpp adapter or its evidence boundary. |
| [`phase-0/PREFIX_DIFF.md`](phase-0/PREFIX_DIFF.md) | P0-L7 provider-neutral Prefix Diff, Envelope Diff and bounded evidence boundary. | Diagnosing structural request differences. |
| [`phase-0/CACHE_OBSERVATION_DIAGNOSTICS.md`](phase-0/CACHE_OBSERVATION_DIAGNOSTICS.md) | P0-L8 reference-based observation comparison, bounded cache assessment and three-layer evidence boundary. | Comparing cache observations without inferring causality. |
| [`phase-0/CACHE_CAPABILITY_REGISTRY.md`](phase-0/CACHE_CAPABILITY_REGISTRY.md) | P0-L9 deterministic capability registry, matrix, typed queries and research-gap report. | Querying capability knowledge without treating unknown as unsupported. |
| [`THREAT_MODEL.md`](THREAT_MODEL.md) | Offline-core data, input and terminal-safety constraints. | Handling trace content or untrusted input. |
| [`phase-0/PHASE_0B_LIVE_VALIDATION.md`](phase-0/PHASE_0B_LIVE_VALIDATION.md) | Controlled live protocol, guardrails and result classification. | Inspecting the live harness; never assume it authorizes a live run. |
| [`phase-0/PHASE_0B_FINDINGS.md`](phase-0/PHASE_0B_FINDINGS.md) | Individual DeepSeek observations and limitations. | Reviewing provider evidence. |
| [`phase-0/PHASE_0B_DEEPSEEK_CLOSEOUT.md`](phase-0/PHASE_0B_DEEPSEEK_CLOSEOUT.md) | DeepSeek closeout decision and stopping rule. | Reviewing the Phase 0B conclusion. |
| [`phase-1/PHASE_1_PLAN.md`](phase-1/PHASE_1_PLAN.md) | Phase 1A/1B/1C design gate and boundaries. | Any proposed Phase 1 work. |
| [`phase-1/PHASE_1B8_CONTROLLED_BENCHMARK_REVIEW.md`](phase-1/PHASE_1B8_CONTROLLED_BENCHMARK_REVIEW.md) and [`PHASE_1B9_HELD_OUT_INTERVENTION_RECALL.md`](phase-1/PHASE_1B9_HELD_OUT_INTERVENTION_RECALL.md) | Completed controlled Phase 1B evidence and limitations. | Reviewing the current Phase 1B result. |
| [`phase-1/PHASE_1C_DESIGN_AUTHORIZATION_GATE.md`](phase-1/PHASE_1C_DESIGN_AUTHORIZATION_GATE.md) and [`PHASE_1C_STAGE_0_CERTIFICATION.md`](phase-1/PHASE_1C_STAGE_0_CERTIFICATION.md) | Frozen Phase 1C design and certified offline replay boundary. | Reviewing Stage 0 or any later authorization. |
| [`phase-1/PHASE_1C_EXTERNAL_EVIDENCE_FRONT_HALF_GATE.md`](phase-1/PHASE_1C_EXTERNAL_EVIDENCE_FRONT_HALF_GATE.md), [`CONTEXTBENCH_FRONT_HALF_EXTERNAL_EVIDENCE.md`](phase-1/CONTEXTBENCH_FRONT_HALF_EXTERNAL_EVIDENCE.md), and [`CONTEXTBENCH_EXTERNAL_TRAJECTORY_ADMISSION.md`](phase-1/CONTEXTBENCH_EXTERNAL_TRAJECTORY_ADMISSION.md) | Current external-evidence admission result and Stage 1 blocker. | Reviewing ContextBench/Tracebench status. |
| [`phase-1/PHASE_1C_H001_TIMEOUT_V2_PREPARATION.md`](phase-1/PHASE_1C_H001_TIMEOUT_V2_PREPARATION.md) | Timeout-only V2 h001 preparation, supervisor boundary, hashes, and separate-live-authorization stop. | Reviewing the V2 preparation record. |
| [`phase-1/PHASE_1C_H001_V2_BASELINE_LIVE_EXECUTION.md`](phase-1/PHASE_1C_H001_V2_BASELINE_LIVE_EXECUTION.md) | Authorized V2 BASELINE child-failure record and pre-inference stop. | Reviewing the V2 execution outcome. |
| [`phase-1/PHASE_1C_H001_V2_BASELINE_ATTEMPT_002_PREPARATION.md`](phase-1/PHASE_1C_H001_V2_BASELINE_ATTEMPT_002_PREPARATION.md) | CLI remediation, immutable attempt-001 lineage, and separate attempt-002 live boundary. | Reviewing attempt-002 preparation. |
| [`phase-1/PHASE_1C_REASONING_BUDGET_CALIBRATION_DESIGN.md`](phase-1/PHASE_1C_REASONING_BUDGET_CALIBRATION_DESIGN.md), [`PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.json), [`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_002_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_002_IDENTITY_V1.json), [`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_003_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_003_IDENTITY_V1.json), and [`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_004_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_004_IDENTITY_V1.json) | Independent non-scored reasoning-on feasibility calibration, immutable attempt lineage, Windows-native exclusivity remediation, launch-specific expected-workflow identity, and Attempt 004 offline preparation. | Attempt 003 is invalid before readiness/inference because the registered supervisor was misclassified; Attempt 004 is prepared only and still requires separate live authorization. |
| [`phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_005_PREPARATION.md`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_005_PREPARATION.md), [`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_005_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_005_IDENTITY_V1.json) | Attempt 004 supervisor launch-identity handoff remediation and Attempt 005 offline preparation. | Attempt 004 was invalid before readiness/inference; Attempt 005 is prepared only and requires separate live authorization. |
| [`phase-1/PHASE_1C_ATTEMPT_005_HASH_FORENSICS_ATTEMPT_006_PREPARATION.md`](phase-1/PHASE_1C_ATTEMPT_005_HASH_FORENSICS_ATTEMPT_006_PREPARATION.md), [`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_006_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_006_IDENTITY_V1.json) | Attempt 005 pre-live hash reconciliation, canonical source fingerprinting, and Attempt 006 offline preparation. | Attempt 005 was not started; Attempt 006 is prepared only and requires separate live authorization. |
| [`phase-1/PHASE_1C_CI_PORTABILITY_REMEDIATION.md`](phase-1/PHASE_1C_CI_PORTABILITY_REMEDIATION.md) | Clean-checkout CI portability fixes, canonical source hashing, and the explicit preserved-evidence certification boundary before Attempt 007. | Reviewing CI remediation without authorizing runtime work. |
| [`phase-1/PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION.md`](phase-1/PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION.md), [`PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION_RECORD.md`](phase-1/PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION_RECORD.md), and [`PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION_V1.json`](phase-1/PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION_V1.json) | Offline real-supervisor/real-child executable-identity certification, Attempt-006 path-mismatch forensics, stable identity rule, and zero-network result. | Reviewing the workflow-identity gate before any Attempt 007 authorization. |
| [`phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_PREPARATION.md`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_PREPARATION.md) and [`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.json) | Attempt 007 preparation gate for the reasoning-budget calibration namespace: exact 1024 configuration, current executable identity, non-contact preflight, dry-run request evidence, and separate live boundary. | Attempt 007 preparation baseline; the frozen identity remains unchanged. |
| [`phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_EXECUTION_RECORD.md`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_EXECUTION_RECORD.md) | Bounded Attempt 007 live execution ledger, raw evidence inventory, cleanup, calibration output, and executable-identity integrity review. | The authorized run was consumed once; runtime target binaries differed from the preparation-bound objects, so execution is invalid and stopped for review. |
| [`phase-1/PHASE_1C_ATTEMPT_007_EXECUTABLE_BINDING_REMEDIATION.md`](phase-1/PHASE_1C_ATTEMPT_007_EXECUTABLE_BINDING_REMEDIATION.md) | Offline provenance, validation-gap classification, frozen supervisor/child object binding, staging lifecycle, regression fixture, and 900-v2 certification boundary. | Reviewing the remediation after Attempt 007 was consumed with invalid executable identity. |
| [`phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_PREPARATION.md`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_PREPARATION.md) and [`PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.json) | Attempt 008 offline eligibility, budget provenance, complete 1024 contract, v2 dependency, frozen execution-object identities, substitution rejection, and future launch commands. | Reviewing the prepared replacement attempt; its live result is recorded separately below. |
| [`phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_EXECUTION_RECORD.md`](phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_EXECUTION_RECORD.md) | Single consumed Attempt 008 live execution, request ledger, runtime identity integrity, bounded evidence hashes, cleanup, and admissible calibration disposition. | Reviewing the published Attempt 008 result; no rerun or 512 follow-up is authorized here. |
| [`phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_PREPARATION.md`](phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_PREPARATION.md), [`PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.json), and [`PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_EXECUTION_RECORD.md`](phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_EXECUTION_RECORD.md) | Attempt 009 preparation and its separately authorized one-time live boundary. The child failed before readiness because the candidate-order validator could not find the prior candidate result; bounded failure evidence and integrity rejection are recorded. | Reviewing the consumed Attempt 009 run; no rerun is permitted and no calibration result is admissible. |
| [`phase-1/PHASE_1C_ATTEMPT_009_CANDIDATE_ORDER_REMEDIATION.md`](phase-1/PHASE_1C_ATTEMPT_009_CANDIDATE_ORDER_REMEDIATION.md) | Offline forensics and remediation of the preparation/runtime candidate-order disagreement, using the tracked Attempt008-to-Attempt009 transition as the shared authoritative dependency. | Reviewing the accepted remediation; Attempt009 remains inadmissible, budget 512 remains untested, and Attempt010 is not prepared. |
| [`phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_PREPARATION.md`](phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_PREPARATION.md) and [`PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json`](phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json) | Attempt 010 budget-512 preparation: eligibility, Attempt008 transition, frozen contract and executables, frozen-supervisor live-prerequisite traversal, substitution rejection, and future commands. | Reviewing the prepared Attempt 010; it is not executed and budget 512 remains untested. |
| [`phase-1/WORKLOAD_CORPUS.md`](phase-1/WORKLOAD_CORPUS.md) | Corpus, licence, provenance and evaluation-leakage requirements. | Planning Phase 1A ingestion. |
| [`phase-1/QUALITY_GATE.md`](phase-1/QUALITY_GATE.md) and [`phase-1/SUCCESS_CRITERIA.md`](phase-1/SUCCESS_CRITERIA.md) | Quality gates, safety failures and phase acceptance criteria. | Designing or evaluating interventions. |
| [`phase-1/PHASE_1A_CORPUS_CLOSEOUT.md`](phase-1/PHASE_1A_CORPUS_CLOSEOUT.md) | Phase 1A corpus/import/observer closeout, historical Tracebench rejection and limitations. | Reviewing the completed Phase 1A corpus gate. |
| [`phase-1/PHASE_1B_DECISION_CONTRACT.md`](phase-1/PHASE_1B_DECISION_CONTRACT.md) | Phase 1B.0 intervention-plan contract and conservative offline baseline. | Reviewing Phase 1B decisions and invariants. |
| [`phase-1/PHASE_1B1_CHARACTERIZATION.md`](phase-1/PHASE_1B1_CHARACTERIZATION.md) | Frozen Phase 1B.0 planner characterization over the accepted Phase 1A traces. | Reviewing the Phase 1B.1 result. |
| [`phase-1/PHASE_1B1_CHARACTERIZATION_SCHEMA.md`](phase-1/PHASE_1B1_CHARACTERIZATION_SCHEMA.md) | Frozen reporting schema for Phase 1B.1 characterization evidence. | Reproducing or auditing Phase 1B.1 reporting. |
| [`phase-1/PHASE_1B2_EVIDENCE_GAP_STUDY.md`](phase-1/PHASE_1B2_EVIDENCE_GAP_STUDY.md) | Evidence-model gap, provenance recommendation and raw-schema uncertainty. | Reviewing the Phase 1B.2 design gate. |
| [`phase-1/PRIOR_ART_DECISIONS.md`](phase-1/PRIOR_ART_DECISIONS.md) | Reuse, integration and differentiation decisions. | Considering external systems or architecture. |

## Implementation locations

| Path | Contains | Read when |
| --- | --- | --- |
| [`../crates/prefixity-core/src/lib.rs`](../crates/prefixity-core/src/lib.rs) | Core module map and offline boundary. | Starting core-code inspection. |
| [`../crates/prefixity-core/src/model.rs`](../crates/prefixity-core/src/model.rs), [`validation.rs`](../crates/prefixity-core/src/validation.rs), [`limits.rs`](../crates/prefixity-core/src/limits.rs) | Trace/profile model, validation and bounds. | Changing input or schema behavior. |
| [`structure.rs`](../crates/prefixity-core/src/structure.rs), [`usage.rs`](../crates/prefixity-core/src/usage.rs), [`prefixity_score.rs`](../crates/prefixity-core/src/prefixity_score.rs) | Fingerprints, provider normalization and heuristic scoring. | Reviewing identity, usage or scoring. |
| [`analysis.rs`](../crates/prefixity-core/src/analysis.rs), [`compare.rs`](../crates/prefixity-core/src/compare.rs), [`cost.rs`](../crates/prefixity-core/src/cost.rs), [`policy.rs`](../crates/prefixity-core/src/policy.rs) | Analysis, comparison, economics and offline policy simulation. | Changing behavior or interpreting results. |
| [`../crates/prefixity-cli/src`](../crates/prefixity-cli/src) | Offline CLI, bounded file loading and output, including Phase 1B planning. | Changing commands or output. |
| [`../crates/prefixity-live/src`](../crates/prefixity-live/src) | Disposable Phase 0B providers, scenarios, guardrails and artifacts. | Reviewing controlled live validation only. |
| [`../crates/prefixity-controlled-benchmark/src`](../crates/prefixity-controlled-benchmark/src) | Isolated offline, research-only controlled benchmark/evaluator and Phase 1C Stage 0 machinery. | Reviewing controlled evidence or offline certification; not production runtime. |

## Tests, fixtures and validation material

The [external artifact admission contract](phase-1/EXTERNAL_ARTIFACT_ADMISSION_CONTRACT.md)
defines the research-only provenance, permission, leakage, retention, and
admission checks for future supplied manifests. The [research-state consistency
guard](RESEARCH_STATE_CONSISTENCY.md) checks a small bounded set of current
repository facts without treating historical evidence documents as current
state.

| Path | Contains | Read when |
| --- | --- | --- |
| [`../crates/prefixity-core/tests`](../crates/prefixity-core/tests) | Fixture, policy, determinism, normalization and safety integration tests. | Verifying core behavior. |
| [`../crates/prefixity-core/src/observation.rs`](../crates/prefixity-core/src/observation.rs) and [`../crates/prefixity-core/tests/observation_schemas.rs`](../crates/prefixity-core/tests/observation_schemas.rs) | Versioned neutral observation/capability types and focused validation tests. | Changing or consuming P0-L2/L3 contracts. |
| [`../crates/prefixity-live/tests`](../crates/prefixity-live/tests) | Fully offline mock-transport pipeline tests. | Verifying live-harness behavior without network access. |
| [`../crates/prefixity-cli/src/output.rs`](../crates/prefixity-cli/src/output.rs) | CLI JSON/output determinism tests. | Verifying rendered output. |
| [`../fixtures/traces/README.md`](../fixtures/traces/README.md) and [`../fixtures/traces`](../fixtures/traces) | Synthetic scenarios and sanitized provider-derived fixtures. | Reproducing documented examples. |
| [`../fixtures/observations`](../fixtures/observations) and [`../fixtures/capabilities`](../fixtures/capabilities) | Representative neutral artifact/observation fixtures and local/cloud capability examples. | Reproducing P0-L2/L3 schema examples. |
| [`../fixtures/conformance/coding-agent-cache-conformance-v1.json`](../fixtures/conformance/coding-agent-cache-conformance-v1.json) | Small synthetic P0-L4 coding-agent-style mutation experiment. | Reproducing conformance-harness tests. |
| [`../fixtures/llama-cpp`](../fixtures/llama-cpp) and [`../fixtures/capabilities/llama-cpp-documented-v1.json`](../fixtures/capabilities/llama-cpp-documented-v1.json) | Synthetic llama-server protocol responses and documented-only capability shape. | Reproducing P0-L5 adapter tests. |
| [`../provider-profiles/README.md`](../provider-profiles/README.md) and [`../provider-profiles`](../provider-profiles) | Synthetic cost-profile data. | Running cost or simulation examples. |
| [`../experiments/runs`](../experiments/runs) | Local ignored live artifacts, not tracked benchmarks. | Auditing recorded live runs if present. |
| [`phase-0/SUCCESS_CRITERIA.md`](phase-0/SUCCESS_CRITERIA.md) and [`phase-0/EXPERIMENTS.md`](phase-0/EXPERIMENTS.md) | Offline acceptance mapping and proposed experiment groups. | Distinguishing harness checks from future experiments. |

There is currently no `benches/` directory or tracked end-to-end quality report.
The Phase 1B decision layer and controlled evidence path are complete through
the 1B.9 held-out study. Phase 1C Stage 0 is certified offline, while Stage 1
is blocked by the external trajectory admission dependency. The Phase 1A
corpus/import evidence and the later Phase 1B/1C results are documented in the
linked closeouts and gates above.

For history, use `git log` and [`research/PRIOR_ART.md`](research/PRIOR_ART.md)
when the task requires provenance or prior-art context.
