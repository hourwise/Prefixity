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
| [`THREAT_MODEL.md`](THREAT_MODEL.md) | Offline-core data, input and terminal-safety constraints. | Handling trace content or untrusted input. |
| [`phase-0/PHASE_0B_LIVE_VALIDATION.md`](phase-0/PHASE_0B_LIVE_VALIDATION.md) | Controlled live protocol, guardrails and result classification. | Inspecting the live harness; never assume it authorizes a live run. |
| [`phase-0/PHASE_0B_FINDINGS.md`](phase-0/PHASE_0B_FINDINGS.md) | Individual DeepSeek observations and limitations. | Reviewing provider evidence. |
| [`phase-0/PHASE_0B_DEEPSEEK_CLOSEOUT.md`](phase-0/PHASE_0B_DEEPSEEK_CLOSEOUT.md) | DeepSeek closeout decision and stopping rule. | Reviewing the Phase 0B conclusion. |
| [`phase-1/PHASE_1_PLAN.md`](phase-1/PHASE_1_PLAN.md) | Phase 1A/1B/1C design gate and boundaries. | Any proposed Phase 1 work. |
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

## Tests, fixtures and validation material

| Path | Contains | Read when |
| --- | --- | --- |
| [`../crates/prefixity-core/tests`](../crates/prefixity-core/tests) | Fixture, policy, determinism, normalization and safety integration tests. | Verifying core behavior. |
| [`../crates/prefixity-live/tests`](../crates/prefixity-live/tests) | Fully offline mock-transport pipeline tests. | Verifying live-harness behavior without network access. |
| [`../crates/prefixity-cli/src/output.rs`](../crates/prefixity-cli/src/output.rs) | CLI JSON/output determinism tests. | Verifying rendered output. |
| [`../fixtures/traces/README.md`](../fixtures/traces/README.md) and [`../fixtures/traces`](../fixtures/traces) | Synthetic scenarios and sanitized provider-derived fixtures. | Reproducing documented examples. |
| [`../provider-profiles/README.md`](../provider-profiles/README.md) and [`../provider-profiles`](../provider-profiles) | Synthetic cost-profile data. | Running cost or simulation examples. |
| [`../experiments/runs`](../experiments/runs) | Local ignored live artifacts, not tracked benchmarks. | Auditing recorded live runs if present. |
| [`phase-0/SUCCESS_CRITERIA.md`](phase-0/SUCCESS_CRITERIA.md) and [`phase-0/EXPERIMENTS.md`](phase-0/EXPERIMENTS.md) | Offline acceptance mapping and proposed experiment groups. | Distinguishing harness checks from future experiments. |

There is currently no `benches/` directory or tracked end-to-end quality report.
The Phase 1B.0 decision layer exists. Phase 1B.1 characterized the frozen
planner and pivoted because the accepted derivative representation did not
exercise positive intervention gates. Phase 1B.2 records the resulting
evidence-model gap and uncertainty about the exact upstream raw schema. The
Phase 1A corpus/import evidence is documented in the closeout above.

For history, use `git log` and [`research/PRIOR_ART.md`](research/PRIOR_ART.md)
when the task requires provenance or prior-art context.
