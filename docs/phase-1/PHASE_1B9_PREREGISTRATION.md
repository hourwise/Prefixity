# Phase 1B.9 - Blinded Held-Out Planner-Evidence Bridge and Intervention-Recall Study

Status: preregistered before held-out fixture construction and outcome
evaluation.

This document freezes the admissible evidence, bridge, policy, measurements,
and decision gates for the Phase 1B.9 offline intervention-selection study.
Changing the policy, evidence interpretation, held-out inclusion criteria, or
success gates after held-out oracle results are inspected invalidates the
study and requires a new preregistration.

The study is offline only. It makes no provider, model, network, replay,
economic, pricing, or Phase 1C call.

## 1. Scientific question and scope

Question:

> Can one minimal deterministic policy select a useful intervention on unseen
> controlled cases from legitimate pre-intervention structural evidence,
> without observing evaluation answers or causing baseline-pass to
> intervention-fail regressions?

The study measures intervention-selection validity, not natural-workload
generalization, provider economics, token savings, or live task quality.
The controlled benchmark's closed-world property is explicitly marked
`CONTROLLED_ONLY`; it must not be promoted into the production planner or
natural-workload interpretation.

S01-S12 are development/sanity cases only. Their oracle outcomes are not
held-out evaluation evidence and are not used to claim policy generalization.

## 2. Planner-facing evidence contract

The blinded planner representation may contain only these admissible facts:

- opaque deterministic scenario, event, action, result, and context IDs;
- event type and actor role;
- validated sequence/order position;
- action-to-result origin identity;
- explicit event references;
- explicit authored `depends_on`, `references`, `supersedes`,
  `protocol_precedes`, and `same_state_revision` relations;
- structural zone derived from event type;
- source class/provenance classification and bounded source revision;
- immutable content hash/equality when present;
- a constant `CONTROLLED_ONLY` closed-world scope marker;
- deterministic relation topology and event counts.

Opaque IDs are positional/structural aliases generated from the validated
event order. Renaming answer-bearing source identifiers must not change a
policy decision.

The bridge preserves explicit relation meaning. A dependency is not converted
into a universal `required` label; supersession is not inferred from time;
repetition is not sufficient for removal; and adjacency is not a dependency.

The existing production `RequestTrace` and Phase 1B planner are not changed.
The frozen baseline receives a neutral `RequestTrace` projection with safety
booleans absent/false, matching the existing conservative projection
contract. The experimental policy receives the separate blinded research
representation, not evaluation objects.

## 3. Explicitly inadmissible evidence

The planner-facing experiment must not contain or branch on:

- oracle `PASS`/`FAIL`, `INVALID_BASELINE`, or `INCONCLUSIVE`;
- expected intervention class, target path, expected risk, or expected effect;
- intervention manifest or exact transformation;
- gold final state, baseline/intervention state comparison, or collateral
  difference;
- fixture purpose, hidden task semantics, or evaluation notes;
- answer-coded scenario, event, action, result, context, or fixture names;
- labels containing `safe`, `unsafe`, `load-bearing`, `irrelevant`,
  `removable`, `protocol-breaking`, or equivalent outcome-bearing language;
- baseline/variant/control outcome identity;
- any field generated from an oracle result;
- timestamps or age interpreted as stale, optional, or removable;
- absence of a dependency as a safety proof except where the explicit
  closed-world duplicate rule below applies.

Evaluation-only answer keys remain in a separate internal object/module and
are accessed only after policy decisions and the frozen baseline have been
recorded.

## 4. Frozen evidence bridge and policy extension

Policy version: `controlled-evidence-policy-v1`.

Rule ordering is fixed:

### Rule 1 - controlled exact-duplicate prune

Select a later message/context event for `PRUNE` only when all conditions hold:

1. the event has an exact immutable content hash equal to an earlier event;
2. an explicit `same_state_revision` relation connects the two events;
3. the target has no explicit consumer reference;
4. no authored dependency or protocol relation requires the target;
5. the relation topology is complete under the controlled closed-world
   contract; and
6. the target is not an action, result producer, current request, or
   protocol-critical event.

This is a `CONTROLLED_ONLY` research inference about the exact closed-world
case. It is not a production `removable=true` label and is not valid for
natural workloads merely because a block is unreferenced.

### Rule 2 - explicit supersession deferral

Select an older context event for `DEFER` only when an explicit
`supersedes(newer, older)` relation exists, the newer event explicitly
precedes the consuming action, and no reference/dependency/protocol relation
requires the older event before that action. Time, lexical order, adjacency,
or content similarity cannot establish supersession.

### Rule 3 - explicit same-zone protocol-preserving relocation

Select a source result/observation for `RELOCATE_CANDIDATE` only when:

1. an explicit `protocol_precedes(source, action)` relation exists;
2. the action explicitly references the source;
3. source and action occupy the same non-message structural zone;
4. at least one unrelated same-zone event lies between source and action; and
5. no authored relation contradicts moving the source immediately before the
   action.

The hypothetical transformation moves only the source immediately before its
consumer. It never crosses a declared protocol/dependency boundary and is
never applied to a live or production trace.

If none of the rules matches, the policy emits `DO_NOTHING`.

There are no tunable numeric thresholds. Rule predicates are exact. The
policy is deterministic, fail-open, and produces at most one selected target
per held-out case. It never emits a destructive decision from repetition,
age, missing edges, scenario identity, or answer-coded identifiers alone.

## 5. Held-out inclusion criteria

The held-out set must:

- contain 12-18 new cases with neutral IDs;
- use new structural arrangements, not renamed copies of S01-S12;
- keep planner-facing structural data separate from evaluation-only answer
  definitions;
- have a complete bounded controlled world and independent deterministic
  oracle;
- include positive, unsafe, ambiguous, and no-op cases;
- include removal, deferral/supersession, relocation, and controls;
- vary event counts, intermediate ordering, distractors, chain depth,
  repetition position, and relation topology without artificial complexity;
- be frozen and hashed before any policy outcome is inspected;
- preserve answer keys outside planner-facing serialization.

Required coverage is:

- exact duplicate with explicit same-state evidence and safe removal;
- unreferenced but hidden load-bearing context;
- explicit dependency/load-bearing result;
- safe and load-bearing repeated context variants;
- absent dependency evidence where no safety inference is allowed;
- explicit supersession safe deferral;
- older/newer context without supersession;
- safe same-zone relocation with explicit protocol order;
- protocol/dependency-boundary relocation failure;
- ambiguous relocation;
- already-efficient, insufficient-evidence, and tempting-unsafe no-op
  controls.

## 6. Frozen baseline and measurements

Before policy evaluation, run the unchanged existing Phase 1B planner on the
neutral blinded projection. Record its full intervention distribution and
deterministic report hash. The production planner is not modified.

For the experimental policy, record per case:

- whether the case has a safe positive intervention available;
- selected class and opaque target, if any;
- true positive, false positive, false negative, or true no-op status;
- independent oracle result for any selected intervention;
- baseline completion and baseline validity;
- baseline-pass to intervention-fail regression;
- unsafe intervention rate;
- class-specific and aggregate precision/recall;
- repeated-run byte/hash equality.

Definitions:

- positive case: the isolated evaluation key declares one bounded intervention
  that preserves the independent task predicate and collateral invariants;
- selected positive: the policy selects the declared target/class for that
  case;
- true positive: selected positive whose independent oracle result is `PASS`;
- false positive: any selected intervention on a non-positive case, or a
  selected positive whose oracle result is not `PASS`;
- false negative: a positive case for which the policy does not select the
  declared positive target/class;
- true no-op: no selection on a case for which no safe positive intervention
  is available;
- precision: true positives / all selected interventions, undefined when no
  interventions are selected;
- recall: true positives / positive cases available;
- unsafe intervention rate: selected interventions causing an oracle `FAIL`
  after a passing baseline / all selected interventions.

No positive result is inferred from structural counts alone. No economic
measurement is part of this study.

## 7. Success and failure gates

The study may be assessed as a held-out positive-intervention success only if
all conditions hold:

- at least one held-out true positive is selected;
- zero baseline-pass to intervention-fail regressions occur;
- zero planner/evaluation leakage is detected;
- the policy and evidence bridge were frozen before held-out scoring;
- decisions are deterministic and reproducible;
- no production planner behavior changes;
- no held-out answer was used for post-hoc tuning.

If no positive intervention is selected, recall remains unresolved/failed;
that is not a safety success. If any unsafe false positive occurs, record it
individually and do not tune and rerun against the same held-out set.

## 8. Mutation/property gates

The implementation must prove that:

- opaque ID renaming leaves decisions unchanged;
- scenario identity changes leave decisions unchanged;
- event-ID lexical ordering changes do not change validated execution or
  decisions;
- unrelated distractors do not change load-bearing relation interpretation;
- removing dependency evidence fails open;
- adding an explicit dependency blocks unsafe removal;
- protocol-boundary changes are governed by relations, not adjacency;
- repeated hashes without same-state/removability evidence do not prune;
- old timestamps do not create stale/defer decisions;
- changing evaluation sidecar labels does not change planner decisions;
- planner-facing serialization contains no prohibited fields or strings;
- repeated study runs produce byte-identical reports.

## 9. Phase 1C gate

Phase 1C remains blocked unless the study demonstrates at least one
reproducible, blinded, held-out, causally validated positive intervention
selected from admissible pre-intervention evidence. Even if that gate passes,
Phase 1C requires a separate design and authorization decision; this study
does not start replay or provider calls.

