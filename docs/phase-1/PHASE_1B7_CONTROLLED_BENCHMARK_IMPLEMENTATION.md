# Phase 1B.7 - Controlled Benchmark Implementation

Status: implementation complete; validation complete; commit/push pending
final review.

Assessment: `PASS WITH RECORDED LIMITATIONS`.

The approved Phase 1B.6 design is implemented as an isolated workspace crate.
It provides a deterministic loader, twelve self-authored cases, explicit pair
validation, a provider-neutral scripted world, a deterministic quality oracle,
and a one-way planner-visible projection. It does not change `RequestTrace`,
CodeTraceBench, the frozen planner eligibility rules, Phase 1C, or any live
provider path.

## 1. Architecture and changed files

The new crate is `prefixity-controlled-benchmark`:

- `src/model.rs` contains the version-1 controlled envelope, provenance,
  events, relations, pairing manifest, and separate `PlannerEvidence` /
  `EvaluationRecord` types.
- `src/loader.rs` performs bounded JSON loading, strict structure/version/hash/
  ordering/relation validation, manifest hashing, and baseline/variant pair
  validation.
- `src/fixtures.rs` materializes the twelve self-authored deterministic seed
  cases in code. No third-party prompt, trajectory, archive, or protected
  bundle is stored.
- `src/world.rs` implements only the scripted state/action/result operations
  required by S01-S12.
- `src/oracle.rs` implements `prefixity-scripted-oracle-v1` and the exact
  `PASS`, `FAIL`, `INVALID_BASELINE`, `INCONCLUSIVE` result vocabulary.
- `src/planner.rs` projects structural evidence into the existing
  `prefixity-core::model::RequestTrace` without adding evaluation fields, then
  invokes the unchanged `prefixity-core::decision::plan_interventions`.
- `src/hashing.rs` provides canonical JSON and lowercase SHA-256 identities.
- `tests/controlled_benchmark.rs` covers loader, pairing, evidence isolation,
  oracle, planner, determinism, and mutation boundaries.
- `Cargo.toml` adds the isolated crate to the existing workspace.
- `Cargo.lock` records only the new isolated workspace package.
- This findings document records the implementation evidence.

`docs/tasks/ACTIVE.md` was not modified, staged, or included. Its pre-existing
unrelated FATES-SLICE-002 worktree modification remains protected.

## 2. Schema and loader result

The implementation accepts the approved `prefixity.controlled-benchmark`
schema version `1`. The loader rejects:

- unsupported schema IDs or versions;
- malformed or overlong IDs, text, hashes, lists, events, relations, or input
  files;
- non-contiguous event sequence indexes;
- duplicate event/action/result/relation IDs;
- invalid action/result identity or `produces` origin relationships;
- unknown reference, result, or relation endpoints;
- missing `controlled-benchmark-relations-v1` scope/version;
- source timestamps without `source_explicit` origin;
- evaluation-only provenance in the planner-visible section; and
- pair mutations outside the declared intervention.

Unknown or absent evidence is preserved. The loader does not synthesize
`optional`, `required`, `stale`, or `removable` fields. It does not turn age,
adjacency, repetition, or an oracle outcome into a safety relation.

The controlled schema remains a design envelope with separate
`trace.planner_input` and `evaluation_sidecar` members. The production
`RequestTrace` model was not changed.

## 3. Fixture and pair integrity

The seed contains twelve baselines and twelve paired intervention traces:
ten variants and two no-change controls. Each pair preserves task revision,
environment revision, initial state, fixed seed, baseline trace ID, variant or
control trace ID, target IDs, exact transformation, and manifest identity.

| Scenario | Pair role | Oracle result | Controlled distinction |
| --- | --- | --- | --- |
| S01 `irrelevant_context_removal` | variant / remove | `PASS` | Unrelated context removal preserves the state. |
| S02 `load_bearing_removal_failure` | variant / remove | `FAIL` | Checkout fails without its inventory result. |
| S03 `explicit_supersession_deferral` | variant / defer | `PASS` | Superseded policy v1 moves behind the use boundary. |
| S04 `action_result_needed_later` | variant / remove | `FAIL` | Later update fails without its generated identifier. |
| S05 `action_result_not_needed` | variant / remove | `PASS` | Unreferenced audit result has no task effect. |
| S06 `dependency_chain_preservation` | variant / remove | `FAIL` | Commit fails without authorization result. |
| S07 `safe_context_relocation` | variant / relocate | `PASS` | Reference moves within the safe pre-action zone. |
| S08 `protocol_breaking_relocation` | variant / relocate | `FAIL` | Handshake moved after its dependent action. |
| S09 `repeated_context_removal` | variant / remove | `PASS` | Exact repeated immutable context is removable in this case. |
| S10 `repeated_but_load_bearing` | variant / remove | `FAIL` | Similar repeated context remains explicitly load-bearing. |
| S11 `already_efficient_noop` | control / no-change | `PASS` | Already-efficient trace remains unchanged. |
| S12 `ambiguous_evidence` | control / no-change | `PASS` | Ambiguity remains a no-intervention control. |

Measured aggregate oracle results:

```text
PASS              7
FAIL              5
INVALID_BASELINE  0
INCONCLUSIVE      0
```

The expected positive and negative distinctions were reproduced by the
independent scripted oracle. These are bounded scenario/intervention results,
not universal planner labels.

The aggregate report hash is:

```text
e257b14803c9a80c69e1c38c549fe2a41cf6edc6d8604cd95243ea4517245572
```

Stable per-scenario manifest hashes are:

| Scenario | Manifest SHA-256 |
| --- | --- |
| S01 | `ec01a792d4e18344c8d3ef23b1ec13054545fbea7b804550857bb0dc23efbbfc` |
| S02 | `1fe86974831dda0af6c285fa4889e47ca339d161054629e7ab25776ddc9638fd` |
| S03 | `3058b560b2144354dc4f861cd7e73915693ea83fa05e778e6dfc507cc86e1ace` |
| S04 | `cfcb33f6466a979e8fdcbfbba652e4f4b7fa78c7015ec7c05e55c681924fc9ae` |
| S05 | `0007249a8fa684844a11f7000fdd80b637c51c7d024df0306624f525954b3016` |
| S06 | `5d8c4510fde79c596d465f3d17e9bbb87bcf13b6f35d4c713e1f5b4e1eec6d38` |
| S07 | `7d4205e73c504c7d0cfdd114b8f824241dda659f3b0eec1fbe08892a2572793c` |
| S08 | `342a99c9c69bb1c0187f845ad9c447f37cb2430599a9c793d4a333ff6ddfa8b0` |
| S09 | `f3d3a5c8ffa29b92448433b5594f57760ee0870b5da1bb0a2e5a1ececfdb7b7c` |
| S10 | `14eba4144d888aba35440c5603fdd10b509ed26d91d624b608932d5e3e384de2` |
| S11 | `c456038ed222606c15704e5791aa634988124307507e0be77598162c8627bc9e` |
| S12 | `fd3806cfc663d76364133ba7aafecb239a0481f9d93c542859b6ed595bbfc3a6` |

## 4. Planner evidence and evaluation isolation

`PlannerEvidence` contains only the scenario identity, trace structural
identity, planner events/relations/provenance, and the existing
`RequestTrace` projection. It has no field for the evaluation sidecar,
intervention result, expected risk, gold state, hidden purpose, collateral
diff, or oracle result.

`EvaluationRecord` is a separate type produced only by the oracle. The planner
projection deliberately sets production-trace safety booleans to their
conservative absent values and carries no provider usage or evaluation label.
Explicit dependency relations are projected only as structural dependency
references; no outcome is used to create them.

The leakage tests verify that serialized `PlannerEvidence` contains none of
`evaluation_sidecar`, `oracle_result`, `expected_quality_risk_category`, or
`intervention_manifest`. Planner execution accepts only `PlannerEvidence`,
not a combined evaluation object.

## 5. Scripted world and oracle semantics

The world has a deterministic initial state containing only scenario, initial
state, and task revision identities. It implements the bounded actions needed
by the seed: profile update, inventory/check-out, policy application, record
creation/update, authorization/commit, reference execution, handshake
execution, repeated-context execution, minimal task, and ambiguous-task
completion.

World execution uses explicit event IDs, action IDs, result origin IDs,
reference order, dependency order, and protocol order. A dependency requires
the dependent action after its prerequisite; `protocol_precedes` requires the
opposite order. No wall clock, random source, network, provider, or model is
used.

The oracle first requires the baseline to complete and match its exact
scenario predicate. A failed baseline is `INVALID_BASELINE`; an unresolved
baseline is `INCONCLUSIVE`. A variant is `PASS` only when it completes, has
the expected state, equals the paired baseline state, and has no collateral
state-key difference. A task failure is `FAIL`; an unresolved execution is
`INCONCLUSIVE`.

## 6. Frozen planner integration

The existing Phase 1B planner was invoked unchanged over 24 projections:
the 12 baselines plus the 12 variant/control projections.

```text
DO_NOTHING  24 / 24
```

No planner eligibility rule, threshold, reason code, safety rule, or
production trace type was changed. The planner did not receive expected
intervention paths or oracle labels, and it was not forced to emit the
conceptual `KEEP`, `PRUNE`, `DEFER`, or relocation paths from the design.

## 7. Determinism and privacy

Repeated `run_benchmark()` calls produced identical fixture projections,
manifest hashes, oracle records, planner runs, canonical report bytes, and
aggregate hash. Source fixture objects remained unchanged after world,
oracle, and planner execution.

The implementation is entirely offline and provider-neutral. It contains
only self-authored labels, bounded identifiers, structural hashes, and
deterministic state values. No AppWorld, tau2-bench, or ToolSandbox dependency
was added. No third-party raw data, protected bundle, prompt, trajectory,
credential, or archive was committed.

## 8. Checks and limitations

Focused coverage includes:

- strict schema/version/hash/order and bound rejection;
- all twelve fixture round trips through the loader;
- stable manifest and aggregate hashes;
- malformed pair and environment mismatch rejection;
- undeclared collateral fixture mutation rejection;
- explicit action/result and authored relation round trips;
- timestamp age and repetition not becoming safety labels;
- evaluation sidecar exclusion from planner evidence;
- deterministic conservative planner output;
- deterministic `PASS`, `FAIL`, `INVALID_BASELINE`, and `INCONCLUSIVE` paths;
- collateral mutation preventing `PASS`;
- fixture immutability; and
- deterministic offline world execution.

The full workspace suite and lint/build checks remain the final gate before
commit. The controlled artifact is intentionally not a general agent
simulator, does not evaluate compression, and does not establish general
optional/stale/required/removable semantics. The 12 cases are synthetic and
bounded; they demonstrate an isolated causal-quality measurement path, not
natural-workload representativeness, provider economics, latency, or Phase 1C
replay readiness.

## 9. Phase 1B.7 assessment and recommended next task

Assessment: `PASS WITH RECORDED LIMITATIONS`.

The implementation demonstrates the required isolated path: self-authored
paired interventions, explicit structural relations, independent deterministic
quality evaluation, strict planner/evaluation separation, and reproducible
aggregate identities. The limitations are deliberate: compression remains
untested, the world is narrow and synthetic, and no causal result generalizes
beyond its exact scenario and intervention pair.

The recommended next task is:

> **Phase 1B.8 - Controlled benchmark review and quality-gate interpretation.**
>
> Review the implementation and measured oracle/planner separation, confirm
> that the bounded PASS/FAIL cases are not being promoted into universal
> planner evidence, and decide whether any strictly scoped follow-up is
> justified. Do not begin Phase 1C or add external benchmark data without a
> new approval gate.
