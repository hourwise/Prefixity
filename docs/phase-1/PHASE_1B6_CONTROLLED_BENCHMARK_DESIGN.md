# Phase 1B.6 - Controlled Intervention Benchmark Design and Seed Audit

Status: design complete; no benchmark, adapter, runner, production schema,
planner rule, provider call, or Phase 1C work was started.

Assessment: `PIVOT`.

The controlled evaluation direction is justified, but AppWorld is not a
defensible first implementation dependency. Its protected encrypted task,
app, and API bundles require a separate data/licence audit and its execution
surface is larger than the smallest experiment needed here. Prefixity should
therefore implement a fully self-authored, provider-neutral controlled
artifact. AppWorld remains a design and task/environment reference only.

The Phase 1B.5 two-track recommendation is unchanged:

1. Keep the pinned, hash-only CodeTraceBench slice as the natural-workload
   observational track. Its 24 accepted trajectories / 719 traces and
   719/719 frozen `DO_NOTHING` result are not altered or reinterpreted.
2. Add a separate controlled paired-intervention track for causal quality
   evidence. It is not a replacement for CodeTraceBench and its evaluation
   labels are not planner input.

The proposed machine-readable schema is
[`controlled-benchmark-v1.schema.json`](schemas/controlled-benchmark-v1.schema.json),
and the twelve-case seed audit is
[`PHASE_1B6_SEED_MANIFEST.json`](PHASE_1B6_SEED_MANIFEST.json). They are design
artifacts, not executable fixtures.

## 1. Design boundary and hypothesis

The experiment is deliberately narrower than a general context classifier:

> Under one fixed task, environment revision, initial state, and seed, does a
> precisely described intervention preserve an independently measured task
> outcome and its no-collateral-change invariants?

A passing ablation establishes evidence for that bounded case and pair. It
does not establish a universal `optional`, `stale`, `required`, or
`removable` property. A failing ablation establishes a load-bearing result
for that case, not a universal `required=true` label. Timestamp age is not
staleness, adjacency is not a tool relationship, and repeated content is not
automatically removable.

No production `RequestTrace` or existing CodeTraceBench representation is
changed. The controlled envelope is an offline research artifact. A future
loader must explicitly project only its `planner_input` member into any
Prefixity analysis and keep the evaluation sidecar unavailable to the
planner.

## 2. Exact public-source audit

The audit was performed against primary repository, release, README, and
licence sources on 2026-08-09. No large dataset was downloaded, no protected
bundle was opened, and no provider/model call was made.

### AppWorld - leading reference, not an implementation dependency

The audited repository identity is `StonyBrookNLP/appworld`. The selected
release is `v0.1.3.post1`, commit
`66ad8099e12188ece0d3fe45e661dbc01880813b`:

- [exact AppWorld tag tree](https://github.com/StonyBrookNLP/appworld/tree/v0.1.3.post1)
- [README at the exact tag](https://github.com/StonyBrookNLP/appworld/blob/v0.1.3.post1/README.md)
- [repository licence path at the exact tag](https://github.com/StonyBrookNLP/appworld/blob/v0.1.3.post1/LICENSE)

The public/plain-text repository portion is Apache-2.0. Protected task/app/API
material is distributed in encrypted bundles under Apache-2.0 with an
additional encrypted-redistribution requirement. The root repository licence
must not be treated as blanket permission to copy or redistribute those
bundles. Any future Prefixity use must pin and audit the exact material used
before implementation and should not copy protected raw data into the
repository.

AppWorld is selected as role C: design/task/environment inspiration only. It
is not a runtime or development dependency and is not an external fixture
generator for this seed. This preserves the useful state-based task,
action/result, and collateral-damage ideas without importing protected data or
making the first benchmark depend on AppWorld's encrypted distribution and
larger execution environment.

### tau2-bench - outcome and replay reference

The audited release is `sierra-research/tau2-bench` `v1.0.1`, commit
`fc0055dc4e0a316c3f83133267fbd6faaa770992`:

- [exact tau2-bench tag tree](https://github.com/sierra-research/tau2-bench/tree/v1.0.1)
- [v1.0.1 release](https://github.com/sierra-research/tau2-bench/releases/tag/v1.0.1)
- [v1.0.1 licence](https://github.com/sierra-research/tau2-bench/blob/v1.0.1/LICENSE)

The release licence is MIT. The retained ideas are explicit action/tool/result
semantics, replay against a task outcome, and the fact that a reference
trajectory is not automatically the only correct sequence. tau2-bench is not
used as a dependency or raw trajectory source. The release is pinned because
its own release notes identify grading changes that make unpinned score
comparisons unsafe.

### ToolSandbox - schema reference only

The public project is [apple/ToolSandbox](https://github.com/apple/ToolSandbox),
with its [state/tool README](https://raw.githubusercontent.com/apple/ToolSandbox/main/README.md)
and [Apple Software licence](https://raw.githubusercontent.com/apple/ToolSandbox/main/LICENSE).
The reviewed public history did not provide an immutable release or verified
commit pin suitable for a Prefixity dependency, so no ToolSandbox revision is
admitted as an implementation source. Its licence is not treated as MIT or
Apache-2.0. Future use would require a fresh exact-revision and licence audit.

The retained ideas are explicit roles and tool results, per-turn/world-state
snapshots, directional milestone relationships, and separate evaluation
milestones. No ToolSandbox code, data, or generated conversation is copied.

## 3. Evidence taxonomy and provenance

The existing taxonomy remains normative:

| Class | Controlled-track meaning |
| --- | --- |
| `CAPTURED_EXPLICIT` | The authored scenario or pinned public metadata directly states an event, ID, action/result link, state revision, order constraint, or provenance field. |
| `DERIVED_STRUCTURAL` | A deterministic hash, canonical ID, ordering projection, state-diff identity, or exact join is computed without assigning a new safety meaning. |
| `EVALUATION_ONLY` | A baseline/variant outcome, gold state, oracle result, collateral diff, hidden case purpose, or expected intervention result. It is never planner input. |
| `INFERRED_UNSAFE` | A semantic claim guessed from age, adjacency, repetition, content similarity, or a convenient trajectory. It is rejected as planner evidence. |
| `ABSENT` | The source or experiment does not establish the claim. The loader must preserve absence rather than fill a default semantic label. |

Every source-bearing structural record carries a bounded provenance entry:
`source_kind`, taxonomy `classification`, optional source locator, source
revision, and a SHA-256 content hash. Self-authored seed records use
`source_kind=self_authored`; public design references are recorded as
metadata, not copied content. A locator identifies a bounded source location;
it is not a raw prompt or archive path.

Raw content is optional. The proposed seed can use deterministic fixture
values, IDs, hashes, state predicates, and generated structural metadata. If a
future implementation needs a small content value for replay, it must be
self-authored, bounded, and covered by the manifest hash. No third-party raw
trajectory, AppWorld bundle, prompt, reasoning trace, credential, screenshot,
or archive belongs in this repository.

## 4. Canonical controlled schema

The schema is an envelope with four separable parts:

1. `scenario` identifies the task version, environment revision, initial state,
   fixed seed, and provenance.
2. `trace` identifies the baseline/variant/control role and baseline trace ID.
3. `trace.planner_input` contains only the event, relation, and structural
   provenance records that could exist at decision time.
4. `evaluation_sidecar` contains the intervention manifest reference and
   quality IDs/results. It is an explicit non-planner channel.

The schema can represent, without requiring raw content:

| Requirement | Representation |
| --- | --- |
| benchmark/task and scenario version | `benchmark_id`, `scenario.scenario_id`, `scenario_version`, `task_revision` |
| event identity and order | `event_id`, `sequence_index`, `order.logical_tick` |
| actor and event kind | `actor_role`, `event_type` |
| action/tool identity | `action.action_id`, `action.tool_name`, optional argument hash |
| result/observation identity | `result.result_id`, `originating_action_id`, optional observation hash/status |
| parent/reference identity | bounded `parent_event_ids` and `reference_event_ids` |
| world state | `world_state_revision`, `same_state_revision` relations |
| context identity | bounded `context_block_id` and optional content hash |
| source provenance | bounded provenance arrays on scenario, planner input, events, and relations |
| timestamp/order metadata | `order.logical_tick`; source timestamp only with `timestamp_origin=source_explicit` |
| baseline/ablation identity | trace role and `baseline_trace_id` |
| intervention target/class | evaluation-only manifest, never an event safety boolean |
| quality evaluation | sidecar IDs and later oracle records, never planner input |

The event schema does not contain direct `optional`, `required`, `stale`, or
`removable` fields. It also does not contain a generic tool-call/result
relationship inferred from neighboring positions. A result relationship must
be explicit through `originating_action_id` and, where relevant, a
`produces` relation.

The proposed schema is bounded: IDs are short and unique within the case,
events and relations have finite limits, text is capped, and hashes use
lowercase SHA-256. These are design limits, not a change to the production
trace limits.

## 5. Safety-relation semantics

Relations are scenario-local and versioned as
`controlled-benchmark-relations-v1`. They are not universal labels:

- `produces(A, R)` means result or observation `R` originated from action `A`.
  It is an identity relation, not proof that a later action needs `R`.
- `references(X, Y)` means the authored event `X` explicitly names or reads
  `Y` as an input. It is narrower than general semantic dependence.
- `depends_on(X, Y)` means the benchmark-authored execution of `X` is not
  valid from the same initial state without `Y` or the state transition it
  represents. It requires an explicit task/environment contract or paired
  failure; adjacency alone cannot create it.
- `supersedes(new, old)` means the scenario explicitly defines `new` as the
  replacement version for the named use of `old`. A later timestamp never
  creates this relation.
- `invalidates(X, Y)` means event/state transition `X` explicitly makes `Y`
  unusable for a named bounded purpose. It is not a synonym for old, unseen,
  or repeated.
- `protocol_precedes(A, B)` means the scenario contract requires `A` before
  `B`; it is the relation used to test safe versus protocol-breaking
  relocation.
- `same_state_revision(A, B)` records that two records belong to the same
  explicitly identified state revision. It does not imply redundancy.

The benchmark may author these narrower relations. It may also mark a target
in an intervention manifest for an experiment. Neither action creates a
general planner boolean. `required`, `optional`, `stale`, and `removable`
remain absent as direct fields. A later bounded conclusion can be recorded in
the evaluation sidecar as “this pair passed/failed under oracle v1”; the
conclusion must not be promoted to a universal rule or silently fed to the
planner.

## 6. Baseline, variant, and control pairing

Each proposed case has one canonical baseline and at least one named variant
or no-change control. Pairing requires:

- identical task revision, environment revision, initial state, and fixed
  seed;
- stable baseline and variant trace IDs;
- one explicit intervention manifest with target event/context IDs;
- exactly described transformation, not hidden mutation;
- a manifest hash and later aggregate manifest hash;
- baseline validation before any variant result is interpreted.

The manifest records whether the target and transformation are
`planner_visible` structural facts or `evaluation_only` experiment metadata.
The target event ID, expected quality-risk category, gold outcome, and oracle
result are evaluation-only even when the structural event itself is visible.
The planner receives neither the answer key nor a future variant outcome.

An ablation that changes more than the named target, starting state, task
revision, or environment revision is invalid. An invalid baseline or a
non-deterministic replay is `INVALID_BASELINE` or `INCONCLUSIVE`, not a
positive or negative safety label.

## 7. Deterministic quality oracle

The first oracle is a scripted, provider-neutral world evaluator, versioned as
`prefixity-scripted-oracle-v1`. It does not use an LLM judge. For every
baseline/variant pair it checks:

1. the baseline reaches the task completion assertion;
2. the final environment/database state satisfies exact predicates;
3. every required action/result identity and named relation is satisfied;
4. prohibited collateral state changes are absent; and
5. the variant's canonical final-state and predicate result is compared with
   the paired baseline under the same starting conditions.

The result vocabulary is `PASS`, `FAIL`, `INVALID_BASELINE`, and
`INCONCLUSIVE`. A case can include a deterministic integer component count,
but no subjective or model-generated score is needed for this seed. A variant
passes the intervention-quality gate only when its task predicates pass and
the collateral-change invariant passes. A successful variant is still scoped
to its scenario and transformation.

The oracle output, gold state, expected case purpose, final-state hash,
variant success/failure, and collateral diff live in the evaluation sidecar.
The planner can later consume only an independently approved structural
evidence projection; it cannot consume the oracle output while choosing the
intervention that the oracle evaluates.

## 8. Decision vocabulary coverage

The existing six decision strings remain unchanged. The seed maps them as
follows:

| Decision | Proposed evidence and cases | What remains unavailable |
| --- | --- | --- |
| `KEEP` | S02, S04, S06, S10: explicit action/result, dependency, protocol, or later reference plus a failed removal/reorder pair. | No universal required label; the failure is case-scoped. |
| `DEFER` | S03: explicit supersession and a passing relocation behind the bounded use boundary. | No timestamp-age or generic stale rule. |
| `PRUNE` | S01, S05, S09: bounded paired removals with equal final predicates and no collateral change. | No general “unreferenced means safe” rule outside the tested case. |
| `RELOCATE_CANDIDATE` | S07: safe relocation within explicit order/state constraints. | No permission to move across a protocol or dependency edge. |
| `COMPRESS_CANDIDATE` | No seed case. Compression semantics, representation, and quality oracle are not justified yet. | Any compression result and planner rule. |
| `DO_NOTHING` | S08, S11, S12 and any invalid/ambiguous pair. | No positive intervention is forced just to improve coverage. |

The table describes future experiment paths, not current planner behavior. No
planner rule is changed or tuned by this design.

## 9. Proposed seed audit

The smallest proposed seed is twelve self-authored scenarios, one for each
required case type. The complete machine-readable table is in
[`PHASE_1B6_SEED_MANIFEST.json`](PHASE_1B6_SEED_MANIFEST.json). The coverage
summary is:

| Scenario | Controlled question | Path |
| --- | --- | --- |
| S01 | genuinely irrelevant context can be removed | `PRUNE` |
| S02 | load-bearing inventory result removal fails | `KEEP` / `DO_NOTHING` |
| S03 | explicitly superseded context can be deferred | `DEFER` |
| S04 | generated action result is needed later | `KEEP` / `DO_NOTHING` |
| S05 | unreferenced action result is not needed later | `PRUNE` |
| S06 | explicit dependency chain must be preserved | `KEEP` / `DO_NOTHING` |
| S07 | relocation inside a safe pre-action zone works | `RELOCATE_CANDIDATE` |
| S08 | relocation across a protocol boundary fails | `DO_NOTHING` / `KEEP` |
| S09 | exact repeated immutable context can be removed in this case | `PRUNE` |
| S10 | superficially repeated context can remain load-bearing | `KEEP` / `DO_NOTHING` |
| S11 | already-efficient trace has no intervention | `DO_NOTHING` |
| S12 | ambiguous evidence is retained and not labelled | `DO_NOTHING` |

Every case has a baseline, an explicitly described variant or control, a
deterministic oracle condition, a negative/control condition, evaluation-only
fields, and no external data dependency. No executable fixture is created in
this task.

## 10. Planner-visible versus evaluation-only boundary

### Planner-visible

The permitted projection contains only what would be available before an
intervention is selected:

- bounded event, context, action, result, and trace IDs;
- event type, actor role, sequence/order, and explicit source timestamps;
- action/tool name and argument/observation hashes where present;
- explicit `originating_action_id`, reference, dependency, supersession,
  invalidation, protocol, or state-revision relations with provenance;
- scenario/environment/task revision and initial-state identity;
- source provenance, bounded locators, and deterministic content hashes.

It does not contain an evaluation result or a semantic boolean filled from
the result.

### Evaluation-only

The sidecar and future oracle records contain:

- baseline/variant pairing and target intervention fields;
- expected structural effect and expected quality-risk category;
- gold task state, completion answer, and hidden case purpose;
- baseline/variant success or failure;
- final-state equality, collateral diffs, and oracle result;
- any “removable,” “required,” “optional,” or “stale” conclusion scoped to a
  particular experiment.

The eventual loader should expose two typed outputs, such as
`PlannerEvidence` and `EvaluationRecord`, and refuse to serialize the latter
into planner input. That loader is future implementation work, not part of
Phase 1B.6.

## 11. Provider neutrality, determinism, and privacy

The initial artifact must run offline with scripted actions and a local
deterministic world. It must not require OpenAI, Anthropic, DeepSeek, another
provider, paid API access, model-generated trajectories, or live network
calls. Provider usage is therefore intentionally absent from this controlled
schema; provider usage remains the responsibility of the separate natural
CodeTraceBench track and its verified adapter evidence.

Future generation/import must use:

- schema `prefixity.controlled-benchmark` version `1`;
- scenario version, stable IDs, and a fixed non-secret seed;
- `prefixity-scripted-world-v1` and `prefixity-scripted-oracle-v1` revisions;
- exact environment and task revisions;
- canonical JSON serialization: UTF-8, LF line endings, sorted object keys,
  no insignificant whitespace, and deterministic array order;
- lowercase SHA-256 for content, source, per-trace, and manifest records;
- a stable intervention-manifest hash and an aggregate SHA-256 over the
  canonical manifest list.

The aggregate hash is not claimed yet because no fixtures have been
implemented. A future run must publish the schema/scenario/oracle revisions,
source manifest, manifest hash, counts, and aggregate hash without publishing
raw protected or private content.

The repository boundary is hash-only for external evidence. The future seed
should be self-authored and deterministic. AppWorld is not a runtime/dev
dependency or fixture generator; no protected AppWorld bundle, third-party
raw trajectory, model reasoning, credential, or archive may be committed.

## 12. Phase 1B.6 decision gate

1. **Exact AppWorld revision/release:** `v0.1.3.post1`, commit
   `66ad8099e12188ece0d3fe45e661dbc01880813b`, repository
   `StonyBrookNLP/appworld`.
2. **Licence/data boundary:** the public/plain-text portion is Apache-2.0;
   protected task/app/API material is distributed in encrypted bundles under
   Apache-2.0 with an additional encrypted-redistribution requirement. The
   exact material used must be pinned and audited before implementation, and
   protected raw data must not be copied into Prefixity.
3. **AppWorld role:** design/task/environment reference only, not a runtime
   dependency or external fixture generator.
4. **ToolSandbox ideas retained:** explicit event roles, tool/action/result
   identity, state snapshots, directional milestone/state relationships, and
   separate evaluation milestones. No unpinned source is relied upon.
5. **tau2-bench ideas retained:** explicit tool/action/result semantics,
   replay-oriented task outcomes, and the principle that a reference
   trajectory is not automatically the only correct trajectory.
6. **Canonical schema:** the versioned controlled-benchmark envelope in
   `schemas/controlled-benchmark-v1.schema.json`, separating
   `planner_input` from `evaluation_sidecar`.
7. **Action/result relationships:** an action has a stable action ID/tool name;
   a result has a stable result ID and required `originating_action_id`, with
   `produces` and explicit `references` where applicable.
8. **Dependencies:** scenario-local, source-explicit or benchmark-authored
   `depends_on` relations, validated by the paired world/oracle; never event
   adjacency.
9. **Supersession/invalidation:** explicit versioned `supersedes` and named
   use-scoped `invalidates` relations; neither is inferred from age or order.
10. **Direct safety booleans:** `optional`, `required`, `stale`, and
    `removable` do not exist as direct benchmark fields. Paired outcomes can
    establish only bounded experimental conclusions in the evaluation sidecar.
11. **Pairing:** same task/environment revision, initial state, seed, and
    canonical baseline; stable trace IDs; explicit manifest; no hidden
    mutation; baseline validity required.
12. **Planner-visible:** bounded structural events, IDs, order, explicit
    source/provenance, hashes, and scenario/state/relation evidence available
    before intervention selection.
13. **Evaluation-only:** target intervention, gold state, expected risk,
    success/failure, oracle result, collateral diff, and any case-scoped
    conclusion.
14. **Quality oracle:** deterministic local final-state predicates, completion
    assertions, action/result checks, invariant checks, and prohibited
    collateral-change checks; no LLM judge.
15. **Paid model calls:** yes, the initial benchmark can run without them;
    the design requires scripted offline actions and makes no provider call.
16. **Seed size:** twelve canonical scenarios.
17. **Decision paths:** `PRUNE` S01/S05/S09; `KEEP` S02/S04/S06/S10;
    `DEFER` S03; `RELOCATE_CANDIDATE` S07; `DO_NOTHING` S08/S11/S12.
18. **Untestable paths:** `COMPRESS_CANDIDATE` remains untestable; a general
    stale/optional/required/removable rule and provider-specific usage effect
    also remain outside this seed.
19. **Provider neutrality:** preserved; correctness comes from the scripted
    world and oracle, not a model or provider.
20. **Privacy/hash-only philosophy:** preserved; only self-authored bounded
    data and hashes are proposed, and protected/uncertain third-party raw
    material stays outside the repository.
21. **Seed size sufficiency:** yes. Twelve cases cover the required positive,
    negative, no-op, and ambiguity classes while remaining auditable by hand.
22. **Implementation now justified:** not with AppWorld as a dependency. The
    controlled benchmark is justified only as the next self-authored
    implementation task described below; no implementation starts in B1.6.

## 13. Assessment, limitations, and next task

### Assessment: `PIVOT`

The public-environment direction is unnecessarily complex for the first
controlled artifact and has a material encrypted-data/licence boundary. A
self-authored deterministic benchmark is smaller, auditable, provider
neutral, and compatible with the hash-only repository boundary. This is not a
claim that AppWorld is poor research; it is a scope and provenance decision.

Important limitations remain:

- the twelve scenarios are a design seed, not measured benchmark evidence;
- no controlled trajectory, oracle result, intervention outcome, or savings
  result exists yet;
- `COMPRESS_CANDIDATE` is intentionally untested;
- scenario-local relations cannot be generalized to arbitrary traces;
- no provider usage, latency, token, or natural-workload claim follows from
  this artifact;
- AppWorld protected material has not been used and must be re-audited if the
  project later revisits it;
- the existing Phase 1B planner has not been changed or tuned.

The exact recommended next task is:

> **Phase 1B.7 - Implement the self-authored controlled benchmark artifact and
> deterministic scripted oracle.**
>
> Implement only the version-1 schema/loader, twelve bounded fixtures, paired
> baseline/variant manifest, offline scripted world, and deterministic oracle;
> keep `PlannerEvidence` separate from `EvaluationRecord`, run the existing
> frozen planner unchanged as a read-only integration check, and do not add an
> AppWorld adapter or begin Phase 1C.

No Phase 1B.7 implementation was started here.
