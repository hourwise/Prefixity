# Phase 1B.9 Held-Out Intervention Recall

## Assessment

**PASS — HELD-OUT POSITIVE INTERVENTION DEMONSTRATED**

The research-only policy selected all four admissible positive interventions in
the frozen 14-case held-out study. It selected no intervention for the ten
negative cases, produced no unsafe intervention, and preserved deterministic
results. This is a controlled structural result only; it does not establish
provider, model, natural-trace, economic, or Phase 1C readiness.

## Frozen study identity

- Base commit: `28189355daa630b0041e9a927bf388dd6ca4b320`
- Base commit message: `docs: add Phase 1B.8 benchmark review`
- Base CI: run `31336179075`, [GitHub Actions run](https://github.com/hourwise/Prefixity/actions/runs/31336179075)
- Preregistration: [`PHASE_1B9_PREREGISTRATION.md`](PHASE_1B9_PREREGISTRATION.md)
- Preregistration SHA-256: `e12846776660960093f9208b099ca171dc4b9c9583150b58de340e965409cd3b`
- Artifact: `prefixity-phase1b9-held-out-v1`
- Policy version: `controlled-evidence-policy-v1`
- Scope marker: `CONTROLLED_ONLY`

The preregistration was frozen before held-out outcomes were created or
scored. S01-S12 remain development and sanity fixtures; they are not included
in the held-out denominators.

## Blinding and evidence boundary

The planner-facing projection contains only opaque event, action, result,
context, reference, relation, order, actor, structural-zone, provenance, and
content-hash fields. The projection omits oracle outcomes, expected risk,
purpose, collateral state, answer-coded identifiers, baseline/variant identity,
age or staleness, and semantic labels. Evaluation labels and hidden execution
requirements are kept in a separate evaluation sidecar and are not passed to
the policy.

No production `RequestTrace` type or production planner behavior was changed.
The unchanged planner receives a neutral controlled `RequestTrace`; the
research policy runs only over the blinded projection.

## Held-out set and frozen hashes

There are 14 neutral held-out cases, including four positive cases and ten
negative cases. Coverage includes:

- exact duplicate with explicit same-state revision;
- duplicate without same-state evidence;
- explicit dependency and hidden load-bearing dependency;
- repeated load-bearing context;
- explicit supersession with protocol ordering;
- older/newer evidence without supersession;
- same-zone protocol-preserving relocation;
- cross-zone and ambiguous relocation candidates;
- no-op, insufficient-evidence, and distractor cases.

The frozen artifact hashes are:

| Artifact | SHA-256 |
| --- | --- |
| Held-out structural set | `10605a2e4b39f04dfbb9a26126a1a416dc036e185d719c32f4407115809b944b` |
| Evaluation key | `c473241757ee72b2df4726e76879c2b94fc4a54e5ca244bb4d6eb1b08bda9a9c` |
| Planner-facing evidence | `9c037de315e1e3090bb0c07786d4aeef180dc7cf84a9351cacb77bf9c42e28af` |
| Research policy | `2139e084d97b16f3ae4ad36d95f0c73b4b1f448fe68f197139aa744dfe0e4` |
| Deterministic report | `be13da638022e6123c21ee9b009f9eaeb58a752e8547508b9f3900fc9d04c0a5` |

The report JSON is [`PHASE_1B9_HELD_OUT_REPORT.json`](PHASE_1B9_HELD_OUT_REPORT.json).

## Frozen baseline

The unchanged frozen planner was run before the research policy. Its complete
held-out distribution was:

| Planner decision | Count |
| --- | ---: |
| `DO_NOTHING` | 14 |

The frozen planner report hash is
`137193fcd6785b4f6d8d4828aa48ba52e211c1f211e5a2446bcb2d448a7667a3`.

## Research-only policy

The policy is deterministic and fail-open. It has three narrowly scoped rules,
evaluated in fixed order:

1. Prune an exact duplicate only when the duplicate has the same content hash,
   explicit same-state revision, no consumer, and no protected dependency or
   protocol relation.
2. Defer an explicitly superseded event only when the newer event protocol-
   precedes the consumer action and the older event has no consumer or
   protected relation.
3. Select a same-zone relocation candidate only when a result/observation is
   explicitly referenced by its consumer action, protocol-precedes that action,
   has an intermediate event, and has no conflicting dependency or protocol
   relation.

All other inputs produce `DO_NOTHING`. The policy uses no numeric thresholds,
timestamps, labels, scenario names, or provider/model fields.

## Results

| Metric | Result |
| --- | ---: |
| Held-out cases | 14 |
| Positive cases available | 4 |
| Positive interventions selected | 4 |
| True positives | 4 |
| False positives | 0 |
| False negatives | 0 |
| True no-op decisions | 10 |
| Precision | 1.0 |
| Recall | 1.0 |
| Unsafe interventions | 0 |
| Baseline-pass → intervention-fail regressions | 0 |

Selected positive cases:

| Case | Selected intervention | Target | Intervention result |
| --- | --- | --- | --- |
| h001 | `PRUNE` | e002 | `PASS` |
| h004 | `PRUNE` | e003 | `PASS` |
| h007 | `DEFER` | e001 | `PASS` |
| h009 | `RELOCATE_CANDIDATE` | e002 | `PASS` |

Cases h002, h003, h005, h006, h008, h010, h011, h012, h013, and h014 all
returned `DO_NOTHING` and are recorded as true no-op decisions.

## Mutation and property coverage

The passing Phase 1B.9 tests cover:

- deterministic policy output and opaque-ID renaming;
- lexical ID changes without semantic interpretation;
- distractor insertion;
- removed same-state evidence;
- added explicit dependency evidence;
- protocol-boundary conflict;
- repeated and old-timestamp inputs not creating intervention evidence;
- changed evaluation-sidecar labels not entering policy inputs;
- serialized blinded evidence excluding prohibited fields and labels;
- byte-identical repeated report generation.

The focused Phase 1B.9 unit suite passed 4/4 tests. The unchanged controlled
benchmark integration suite passed 16/16 tests.

## Leakage and limitations

The planner-facing serialization audit found no oracle, expected-risk,
purpose, collateral, answer-coded, baseline/variant, age/staleness, or
provider/model labels. The evaluation key remains sidecar-only, and the
research policy has no parameter through which it can receive those labels.
The production planner and production trace types remain unchanged.

The evidence is self-authored, controlled-only, and small (four positive
cases). It demonstrates held-out structural intervention recall under the
frozen protocol; it does not demonstrate generalization to natural traces,
provider/model behavior, economics, or production safety.

## Phase 1C gate

The narrow Phase 1B.9 success condition is met: at least one true-positive
intervention was selected, unsafe regressions are zero, leakage checks pass,
the report is deterministic, and preregistration preceded scoring. Therefore
the answer for this study is **YES** for a separate Phase 1C design and
authorization gate. Phase 1C has not started and remains blocked until that
separate gate is explicitly completed.

Recommended next task: **Phase 1C design and authorization gate**.
