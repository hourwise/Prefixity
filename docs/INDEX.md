# Prefixity Knowledge Index

> Navigation map only. Use the linked documents for project explanation and
> evidence.

## Read first

| Path | Contains | Read when |
| --- | --- | --- |
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
