# Phase 1B.8 - Controlled Benchmark Review and Quality-Gate Interpretation

Status: review complete; no planner, fixture, corpus, provider, or replay
implementation started.

Assessment: `PASS — BOUNDED FOLLOW-UP REQUIRED`.

The corrected Phase 1B.7 evidence establishes a reproducible, provider-neutral
quality-measurement mechanism for twelve bounded self-authored interventions.
It does not establish that the frozen Prefixity planner can select positive
interventions, that the authored rules generalize, or that Phase 1C replay is
ready.

## 1. Corrected checkpoint and reproduction

The reviewed base is:

```text
repository: hourwise/Prefixity
branch:     main
commit:     2ae2e05aeb7016343a79a9702709d209c24682a1
message:    fix: preserve controlled benchmark execution order
```

The corrected Phase 1B.7 CI run completed successfully:

- workflow: `CI`
- run: [#31](https://github.com/hourwise/Prefixity/actions/runs/31335825798)
- head: `2ae2e05aeb7016343a79a9702709d209c24682a1`
- conclusion: `success`

The focused controlled-benchmark reproduction confirms:

```text
aggregate hash: 6de4017421e66a81e4bc6d5662b731d879f5d7c2ed9a343d7d8f758ac90ca37d
PASS:           7
FAIL:           5
INVALID_BASELINE: 0
INCONCLUSIVE:     0
planner:        DO_NOTHING 24 / 24
```

The corrected world executes the validated event vector in `sequence_index`
order and tracks result availability at the point where an event is reached.
The S06 regression tests demonstrate both the actual action order and that a
future result is not treated as available merely because it is structurally
present in the trace.

The existing `docs/tasks/ACTIVE.md` modification is unrelated protected work.
It was inspected but not changed, staged, or included in this review.

## 2. What Phase 1B.7 establishes

The controlled benchmark establishes four bounded facts:

1. A validated, provider-neutral event model can express explicit action,
   result, reference, dependency, supersession, protocol-order, state, and
   paired-intervention relationships for self-authored cases.
2. A small scripted world can execute those cases deterministically when
   authoritative trace order and current result availability are enforced.
3. An independent deterministic oracle can distinguish the authored safe and
   unsafe paired outcomes, including baseline invalidity and inconclusive
   execution paths.
4. Evaluation data can remain outside the object passed to the current frozen
   planner computation; repeated projections and planner runs remain
   deterministic.

These facts are evidence about the controlled artifact and its bounded
measurement path. They are not evidence that the same structural signals are
available, reliable, or safe on natural workloads.

The layers remain distinct:

| Layer | What it contains | What it may establish |
| --- | --- | --- |
| Case-level causal fact | Baseline/intervention execution and the scripted state predicate | Whether this exact authored intervention changed this exact bounded task |
| Planner-visible structural evidence | Events, identities, order, hashes, authored relations and projected `RequestTrace` fields | What a deterministic planner could inspect, subject to the projection contract |
| Evaluation-only answer key | Intervention manifest, expected effect/risk, oracle result, final-state comparison and collateral keys | How the pair was designed and how it scored |
| Possible future planner policy | A pre-registered rule using admissible evidence | A hypothesis to test on unseen cases, not a fact supplied by this benchmark |

## 3. S01-S12 interpretation

The table records structural evidence available before intervention, the
intervention class, the case-level result, and the limit of the conclusion.
The descriptions intentionally do not promote the evaluation answer into
planner evidence.

| Case | Structural evidence available before intervention | Intervention / oracle | Causal interpretation and scope | Not justified |
| --- | --- | --- | --- | --- |
| S01 `irrelevant_context_removal` | A message is unrelated to the update action; no authored reference or state transition uses it. | Remove; `PASS` | In this scripted world, removing that exact unrelated event preserved the task state and paired baseline. | Unrelated-looking context is not universally removable; absence of an edge is not a general quality proof. |
| S02 `load_bearing_removal_failure` | Checkout explicitly references the inventory result and has an authored dependency on it. | Remove; `FAIL` | The paired removal breaks the exact checkout prerequisite. This is bounded evidence for the authored dependency. | Every referenced item is not automatically universally required across tasks or providers. |
| S03 `explicit_supersession_deferral` | Old and new policy messages have an explicit supersession relation; the new policy precedes the apply action. | Defer/relocate old policy; `PASS` | Moving the explicitly superseded old policy behind the action boundary preserved the current-policy task in this case. | Age, adjacency, or text similarity alone does not establish supersession or safe deferral. |
| S04 `action_result_needed_later` | The later update action explicitly references the generated identifier result from create and has a dependency edge. | Remove; `FAIL` | Removing the exact producer result makes the later scripted update fail. | All prior action results are not universally load-bearing, and the result identity does not establish a general pruning rule by itself. |
| S05 `action_result_not_needed` | Audit produces a result that has no authored consumer and no scripted state effect; the update path is independent. | Remove; `PASS` | This exact unreferenced audit result was removable without changing the bounded state. | Unreferenced in one trace does not prove irrelevant in natural workloads or under hidden consumers. |
| S06 `dependency_chain_preservation` | Create, authorize, and commit form an explicit action/result/dependency/protocol chain. | Remove; `FAIL` | Removing the authorization result breaks the exact commit chain; corrected execution also proves the producer/action order. | An explicit dependency in a synthetic world is not a universal `KEEP` rule for every similar-looking block. |
| S07 `safe_context_relocation` | A reference is required before the action, while a same-state context item occupies the same non-message context region; protocol order is authored. | Relocate; `PASS` | The exact within-zone move preserved the scripted state and protocol boundary. | A relocation that is safe here is not safe across zones, chronology, protocol boundaries, or unseen state. |
| S08 `protocol_breaking_relocation` | Handshake must precede the dependent action through explicit protocol and dependency relations. | Relocate; `FAIL` | Moving the handshake after its consumer breaks the scripted protocol. | All reordering is unsafe, or all protocol-looking order can be inferred from adjacency alone. |
| S09 `repeated_context_removal` | Two messages have the same authored immutable content hash/state relationship; the action references the first, not the second. | Remove; `PASS` | The exact repeated second message was removable in this bounded task while the first remained. | Structural repetition alone is not a general removability signal. |
| S10 `repeated_but_load_bearing` | Similar-looking repeated context includes an explicit later reference to the second item. | Remove; `FAIL` | The apparently repeated second item is load-bearing in the authored task. | Similarity, repetition, or token reduction can never substitute for dependency/quality evidence. |
| S11 `already_efficient_noop` | Minimal action/result trace with no proposed structural change. | No change; `PASS` | The no-op control remains a valid successful outcome. | A no-op result says nothing about intervention recall or savings. |
| S12 `ambiguous_evidence` | Repeated-looking observations have no authored dependency or removability evidence. | No change; `PASS` | Retaining ambiguity is the correct bounded control under the fail-open contract. | Ambiguity is not evidence for either removal or preservation in general. |

The causal meaning of each row depends on the independent scripted predicate
and paired transformation. The table does not convert the case name,
intervention manifest, expected risk, or oracle outcome into a planner label.

## 4. Interpretation of the 7 PASS / 5 FAIL split

The mixed result is meaningful in a limited but important sense. The oracle
does not approve every authored intervention:

- removal includes safe S01 and S05, but rejects load-bearing S02, S04, S06,
  and S10;
- explicit supersession/defer succeeds in S03;
- relocation succeeds only when the authored protocol boundary is preserved
  (S07) and fails when it is broken (S08);
- exact repetition is removable in S09 but not when the repeated item is
  explicitly load-bearing in S10;
- S11 and S12 demonstrate that no-op/control outcomes are valid.

This reduces the specific risk that the oracle is merely an
"approve-everything" mechanism. It does not make the result statistically
balanced or independent: the twelve cases were intentionally authored to
contain these distinctions, the world is small, the task predicates are
handwritten, and the oracle checks the same bounded state model that shaped
the fixtures. There is no natural-workload estimate of precision, recall,
generalization, or economic value.

The correct conclusion is therefore: the benchmark can measure bounded
causal distinctions; it has not validated a general intervention policy.

## 5. Interpretation of 24/24 `DO_NOTHING`

The zero-intervention result is a central Phase 1B finding, not a benchmark
failure and not evidence of planner success.

The corrected projection intentionally sets every projected block's
`optional`, `required`, and `stale` fields to `false`. It also supplies no
quality/replay evidence, no economic profile, no provider usage, and no
semantic removability label. Authored `DependsOn` relations are projected as
structural dependency references, but they do not manufacture optionality or
prove that a non-dependent block is safe to remove. The frozen planner's
destructive gates require explicit optionality plus tool-result/source/zone
and dependency conditions; repetition, non-gold status, age, event identity,
and the controlled oracle are explicitly insufficient. Relocation remains
subject to the existing zone, chronology, protocol, dependency, and source
constraints and is hypothetical only.

Consequently, the planner has no admissible positive safety evidence in this
projection and fails open to retention or trace-level `DO_NOTHING`. This is
consistent with the Phase 1B contract and the planner's conservative design.

Zero intervention recall is now the principal unresolved Phase 1B problem if
the research question is whether Prefixity can select useful interventions.
The evidence gap is not simply that the thresholds are too high: the current
controlled-to-production projection deliberately refuses to create the
semantic safety fields that the frozen planner requires. The benchmark proves
causal distinctions independently, but the current planner experiment cannot
test whether those distinctions are recoverable as planner evidence.

The right response is not to lower thresholds, add answer labels, or tune
against S01-S12. It is to define a blinded, pre-registered evidence bridge
and test it on unseen controlled cases before claiming intervention recall.

## 6. Planner/evaluation leakage review

### Confirmed separation

The following evaluation-only fields remain outside `PlannerEvidence`'s
production `RequestTrace` projection and outside the argument to
`plan_interventions`:

- oracle `PASS`/`FAIL`;
- expected structural effect and expected quality-risk category;
- intervention target manifest and exact transformation;
- baseline/intervention final-state comparison;
- collateral state-key results;
- hidden case purpose and evaluation notes.

The serialized projection test explicitly checks that
`evaluation_sidecar`, `oracle_result`, `expected_quality_risk_category`, and
`intervention_manifest` do not enter planner evidence. The oracle is called
separately from the planner, and the planner runs are produced from the
projection without consulting evaluation records.

### Subtler leakage risk

The current architecture is safe for the frozen planner's actual computation,
which receives only `evidence.request_trace` and does not branch on these
identifiers. It is not yet a sufficiently blinded interface for future
planner-policy experimentation:

- scenario IDs such as `S02_load_bearing_removal_failure` and
  `S07_safe_context_relocation` encode the intended semantic outcome;
- event IDs and fixture names contain terms such as `inventory-result`,
  `authorize-result`, `safe`, `protocol-breaking`, and `load-bearing`;
- `variant_role` distinguishes baseline, variant, and control;
- scenario and benchmark IDs remain in `PlannerEvidence` and in
  `RequestTrace` session/metadata fields.

These are harmless research identifiers for audit and report joining, and
the current frozen planner ignores them. They are nevertheless answer-coded
features available to a future rule author or learned policy and could
invalidate an intervention experiment if left exposed. Tool names and
explicit action/result identities are different: they can be legitimate
structural evidence, but their admissibility must be declared before an
experiment and tested without using the evaluation answer.

No leakage correction is made in Phase 1B.8. The issue is recorded for the
single bounded follow-up below.

## 7. Overfitting and benchmark contamination risk

S01-S12 are deliberately useful tests and deliberately unsafe as a direct
development set. A rule designed to recognize the exact names, event shapes,
or authored relation patterns can reproduce the seven positive cases while
failing on unseen tasks. Reusing these same cases to choose the rule and then
reporting their oracle results would conflate development and evaluation.

Before any planner-policy experiment, the minimum defensible controls are:

- pre-register the admissible evidence, rule, thresholds, and expected
  failure behavior before reading held-out outcomes;
- blind answer-coded scenario, fixture, and variant identifiers;
- keep S01-S12 as development/sanity cases or freeze them as a development
  set, not as the sole evaluation set;
- create a small held-out set of structurally constrained unseen cases,
  including positive, negative, and no-op controls;
- use mutation/property tests for ordering, dependency closure, missing
  references, supersession, repetition, collateral changes, and determinism;
- keep the oracle and quality labels outside planner input and report every
  baseline-pass to intervention-fail regression individually.

A large benchmark, external corpus import, or learned planner is not needed
to resolve this immediate validity question.

## 8. Phase 1C readiness assessment

Phase 1C is not ready.

The Phase 1 plan and quality gate already define the required shape of replay:
predeclared quality gates and thresholds, reproducible recommendations,
measurable required/gold-context preservation, abort/rollback behavior,
bounded and authorized provider calls, and end-to-end accounting for input,
fresh input, cache reads, output, turns, tool calls, rereads, recovery,
latency, and cost.

Phase 1B.7 supplies a bounded deterministic oracle and structural Tier 0/2
signals, but it does not supply:

- a positive Prefixity-selected intervention from the current planner;
- a blinded or held-out planner-policy result;
- provider/model replay authorization or settings;
- predeclared replay thresholds and evaluator/scorer configuration;
- live abort/rollback execution evidence;
- end-to-end efficiency measurements.

Beginning Phase 1C now would mostly replay the no-op behavior selected by the
frozen planner. That would test the harness's ability to preserve a baseline,
not the central hypothesis that Prefixity can select a useful intervention
while preserving task quality. A no-op control is valuable in Phase 1C, but it
cannot by itself answer that intervention-selection question.

## 9. Candidate next paths

| Path | Assessment |
| --- | --- |
| A - Go directly to Phase 1C | Not justified. The current planner selects no positive intervention, and replay gates/thresholds/provider controls are not yet instantiated. |
| B - Change the planner using S01-S12 | Not defensible as an evaluation. It risks overfitting to authored names, shapes, and answer semantics and would reuse the development cases as proof. |
| C - Build a held-out controlled evaluation set before changing planner policy | Strongly justified. It preserves a clean test of recall and false-positive behavior, provided identifiers are blinded and rules are frozen first. |
| D - Design a narrow evidence bridge/policy extension, then freeze it before unseen evaluation | Necessary as part of the bounded follow-up. The bridge must expose only source/experiment-authorized evidence and must not manufacture optionality or removability from oracle labels. |
| E - Return to another natural/public corpus | Not the smallest next step. The accepted natural corpus still lacks the exact action/result/dependency/load-bearing joins needed for this causal question. |
| F - Stop/Pivot | Not required. The controlled oracle and architecture provide enough bounded evidence to justify one more validity-preserving Phase 1B step. |

Paths C and D are therefore combined into one narrowly scoped follow-up below;
no part of that follow-up is started here.

## 10. Central-hypothesis assessment

| Proposition | Assessment | Evidence |
| --- | --- | --- |
| 1. Prefixity can represent real/natural workloads safely. | `PARTIALLY SUPPORTED` | Phase 1A deterministically represented 719 accepted request traces with provenance and hash-only boundaries; no safe intervention on natural workloads has been established. |
| 2. Prefixity can preserve provenance without inventing safety semantics. | `SUPPORTED` | Phase 1A provenance/evidence boundaries and Phase 1B.7 loader/projection tests preserve explicit versus derived evidence and reject synthesized safety labels. |
| 3. Prefixity can express controlled structural dependency/supersession/order evidence. | `SUPPORTED` | The validated controlled schema and corrected scripted world express and enforce the authored S01-S12 relationships, within this bounded provider-neutral model. |
| 4. A deterministic quality oracle can distinguish safe from unsafe bounded interventions. | `SUPPORTED` | The corrected twelve-case run is deterministic and returns seven PASS, five FAIL, with explicit invalid-baseline and inconclusive test paths. |
| 5. Planner/evaluation information can be isolated. | `PARTIALLY SUPPORTED` | Answer fields are excluded from the current planner computation and leakage tests pass, but answer-coded identifiers remain exposed to future planner experiments and require blinding. |
| 6. The current frozen planner can identify positive interventions from available controlled structural evidence. | `NOT YET SUPPORTED` | All 24 baseline/variant projections produce `DO_NOTHING`; the projection intentionally lacks the explicit safety metadata required for positive gates. |
| 7. The current evidence supports general intervention policy. | `NOT YET SUPPORTED` | The cases are self-authored, bounded, and answer-shaped; no held-out, natural, statistical, or end-to-end policy evidence exists. |
| 8. Prefixity is ready for live/provider replay. | `NOT YET SUPPORTED` | Phase 1C requires reproducible positive selection, frozen quality thresholds, preservation measurement, rollback controls, bounded authorization, and end-to-end accounting not yet present. |

## 11. Overall Phase 1B.8 decision

### `PASS — BOUNDED FOLLOW-UP REQUIRED`

The Phase 1B program remains scientifically viable. Phase 1B.7 met its
narrow purpose: it established that an isolated controlled artifact can
measure bounded quality consequences without leaking the oracle into the
current planner computation. It did not meet the stronger purpose of showing
useful intervention selection by Prefixity.

Advancing directly to Phase 1C would test a no-op planner and would not
resolve the central Phase 1 hypothesis. Stopping would discard a useful
measurement path prematurely. One blinded, pre-registered, held-out evidence
and recall step is justified before replay.

## 12. Remaining limitations

- All twelve cases are self-authored and synthetic.
- The scripted world is not a general agent or application simulator.
- The oracle measures bounded task predicates and state equality, not broad
  semantic quality or human utility.
- The seven/five distribution is not an unbiased estimate of intervention
  precision or recall.
- The current planner projection intentionally omits optional/stale/required/
  removable safety semantics rather than learning them from the oracle.
- Answer-coded identifiers remain a future-experiment contamination risk.
- No end-to-end token, cache, latency, cost, reread, recovery, or tool-call
  benefit has been established for an intervention.
- No live provider call or Phase 1C replay has been authorized or performed.

## 13. Exactly one recommended next task

> **Phase 1B.9 - Blind held-out planner-evidence bridge and intervention-recall study.**
>
> Freeze and pre-register one minimal, source/experiment-authorized evidence
> projection that does not manufacture safety labels or expose answer-coded
> identifiers; create a small structurally constrained held-out controlled set
> with independent oracle labels; then measure the unchanged planner and any
> separately approved narrow policy extension for positive recall,
> false-positive interventions, baseline-pass to intervention-fail
> regressions, and determinism. Keep S01-S12 for development/sanity use and
> keep the oracle entirely outside planner input.

This task is recommended but not started by Phase 1B.8.
