# Phase 1B.2 Evidence Modeling Gap Study

Status: complete; assessment `PIVOT`.

This is a research/design gate. It does not implement an importer revision,
change the Phase 1B.0 planner, broaden the corpus, expose evaluation labels to
planner inputs, replay trajectories, or make provider calls.

## Inspection method and boundary

The inspected corpus is the accepted `NJU-LINK/CodeTraceBench` revision
`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`, `verified` split, under
`fixtures/phase-1a/codetracebench-mini-swe-v1`.

The inspection read JSON structure and metadata only: field names, object
shapes, IDs, counts, roles, source classifications, positions, provenance
locators, hashes and label metadata. It did not print, reconstruct or
interpret prompt text, reasoning, assistant content or tool output. All 719
normalized traces and 24,416 normalized blocks were counted. A deterministic
detail sample was fixed before interpretation: the lexicographically first
selected trajectory in each solved x short/medium/long cell (six
trajectories total). The sample IDs and sanitized field counts are recorded in
[`phase1b2-structural-audit.json`](../../fixtures/phase-1a/codetracebench-mini-swe-v1/results/phase1b2-structural-audit.json).

No raw `.traj.json` or `.tar.zst` artifact is present under the accepted local
fixture root, and the sandbox could not inspect sibling user directories. The
existing importer source was inspected only to document its upstream field
contract and deterministic mapping; this study does not treat unverified raw
fields as present.

## Structural findings

The accepted derived evidence contains:

- 24 trajectory IDs, task names and task slugs, all round-tripping to the
  pinned selection;
- 719 unique request IDs and 24 session/trajectory IDs;
- 24,416 blocks with source-event index, generated message ID, structural path,
  role, source classification, semantic zone, content hash and byte/token
  metadata;
- 11,489 `assistant`, 12,208 `user` and 719 `system` roles;
- 11,489 `conversation`, 10,998 `tool_result`, 1,210 `user_request` and 719
  `system_policy` source classifications;
- only `system` and `messages` semantic zones;
- zero dependency edges, zero provider-usage records, zero timestamps, zero
  lifetimes and zero retained content;
- zero true `optional`, `required` or `stale` flags. Their presence in every
  normalized block is the importer’s default boolean shape, not upstream
  evidence that the corresponding fact is false.

The separate evaluation file contains 24 trajectory records, 23 labelled
stage records and 60 labelled step records. It is explicitly marked as
evaluation-only and has no source-event index or normalized-block identifier
for an exact join.

## Evidence classification matrix

`Planner-safe` means safe to preserve as structural evidence for the existing
offline boundary. It does not mean that the fact proves removability or task
quality. `Import revision` describes a future design only; no revision is
implemented here.

| Desired Prefixity evidence | Upstream/accepted source | Classification | Deterministic rule | Planner-safe? | Evaluation-only? | Import revision? |
| --- | --- | --- | --- | --- | --- | --- |
| Trajectory identity | Manifest `traj_id`; trace/session metadata | `CAPTURED_EXPLICIT` | Copy the pinned trajectory identifier and verify directory/trace agreement | Yes, provenance only | No | Preserve |
| Task identity | Manifest `task_name`, `task_slug` | `CAPTURED_EXPLICIT` | Copy exact manifest fields | Yes, provenance only | No | Preserve |
| Stage identity | Label records’ `stage_id`; no normalized stage field | `EVALUATION_ONLY` | Preserve only in the external label channel | No | Yes | Preserve in evaluation sidecar only |
| Step identity | Label records’ `step_id`; no normalized step field | `EVALUATION_ONLY` | Preserve only in the external label channel | No | Yes | Preserve if an explicit upstream mapping is verified |
| Message/event identity | `message-####` / `source_event_id` | `DERIVED_STRUCTURAL` | Format the source message-array index; scope with trajectory/source-file identity | Provenance only; not an upstream ID | No | Also preserve an upstream ID if explicitly present |
| Request/turn identity | `request_id`, `session_id`, `turn_index` | `DERIVED_STRUCTURAL` | Prefix context before each assistant response and enumerate assistant turns | Yes for request chronology | No | Preserve the rule and source locator |
| Message role | Upstream message `role`, retained in blocks/events | `CAPTURED_EXPLICIT` | Copy the explicit role | Yes for protocol protection | No | Preserve |
| Adapter protocol kind | Role plus source-format marker classification | `DERIVED_STRUCTURAL` | `system` -> system policy; `assistant` -> conversation; marked `user` -> tool result; other `user` -> user request | Limited; useful for audit, not removability | No | Preserve with a versioned rule ID |
| Chronological order | Upstream message-array order and source index | `CAPTURED_EXPLICIT` | Preserve array order; normalized `position` is its deterministic projection | Yes for chronology | No | Preserve |
| Current-request boundary | Prefix ending immediately before an assistant response | `DERIVED_STRUCTURAL` | Use the response index and preceding message prefix | Yes as a request boundary, not semantic optionality | No | Preserve source response index/rule |
| Assistant reasoning vs visible assistant message | No channel/type field in accepted evidence | `ABSENT` | No safe split from role alone | No | No | Add only if explicit upstream field is verified |
| Tool invocation identity | No action/tool-call ID in accepted traces/events | `ABSENT` | No identifier available | No | No | Preserve only explicit upstream IDs |
| Tool observation/result identity | `tool_result` source classification only | `DERIVED_STRUCTURAL` | Apply the importer’s explicit source-format marker rule | Limited; no identity or link | No | Preserve classification and rule, not a guessed ID |
| Tool invocation/result linkage | No call/result/reference IDs | `ABSENT` | Do not pair by adjacency or role sequence | No | No | Add only from explicit references |
| Action -> observation relation | No action/observation relation field | `ABSENT` | No safe relation available | No | No | Add only from explicit references |
| Stage -> contained steps | Evaluation `stage_id` with nested label step IDs | `EVALUATION_ONLY` | Keep the evaluation nesting outside planner input | No | Yes | Preserve in evaluation sidecar |
| Semantic zone | Normalized `system`/`messages` from protocol mapping | `DERIVED_STRUCTURAL` | Derive only from role/source-kind mapping, never content semantics | Yes for system/messages chronology; no tools zone exists | No | Preserve mapping rule and origin |
| Protocol dependency | Role/zone protection rules, but no graph edge | `ABSENT` | Do not turn protocol criticality into a dependency edge | No dependency claim | No | Keep protection distinct from dependency graph |
| Explicit reference dependency | `dependencies: []` in every accepted block | `ABSENT` | Empty default is unknown evidence, not proof of no dependency | No | No | Preserve only explicit IDs |
| Semantic/load-bearing dependency | No explicit reference or relation | `INFERRED_UNSAFE` | Reject topical similarity, order, repetition or later use as a dependency | No | No | Do not add |
| Required | Normalized `required: false` default; no source field | `ABSENT` | Do not interpret the default boolean as an upstream negative fact | No | No | Use explicit tri-state evidence only |
| Optional | Normalized `optional: false` default; no source field | `ABSENT` | Do not infer from tool-result type, age, score, repetition or non-gold status | No | No | Use explicit tri-state evidence only |
| Stale | Normalized `stale: false` default; no invalidation field | `ABSENT` | Do not infer from age or absence of later use | No | No | Use explicit invalidation/supersession only |
| Supersession/invalidation | No accepted field or transition record | `ABSENT` | No stale transition can be established | No | No | Preserve explicit transition IDs if verified |
| Provider usage/cache state | `usage` absent in all 719 traces | `ABSENT` | Provider/model identity is not usage evidence | No | No | Preserve explicit raw usage only when present |
| Deterministic token estimate | Surrogate token-count metadata | `DERIVED_STRUCTURAL` | `ceil(canonical_event_chars / 4)`; never call it provider usage | Limited accounting only | No | Preserve method and units |
| Exact provider token usage | No raw usage or provider tokenizer result | `ABSENT` | Do not promote surrogate counts | No | No | Add only explicit provider capture |
| Source provenance | Source paths, source-file/archive hashes, event indices | `CAPTURED_EXPLICIT` in accepted evidence | Verify hashes and preserve locators | Yes for audit | No | Preserve and type the origin |
| Evaluation-step join | Trajectory IDs overlap; no block/event/step key mapping | `ABSENT` | Do not map by position or step count | No | Labels remain external | Add only an explicit verified mapping |

## Deterministic derivations versus captured facts

The following are safe deterministic projections, not upstream facts:

1. `message-####` IDs and normalized block positions from the source message
   array index.
2. Request/session identifiers and turn indices from trajectory identity and
   assistant-response prefixes.
3. Source kinds and semantic zones from explicit role plus the documented
   source-format marker rule.
4. Structural paths such as `messages[n]`.
5. Content hashes, byte counts and surrogate token estimates from canonical
   serialized message objects.

These derivations are useful for chronology, audit and structural comparison.
They do not establish optionality, requiredness, staleness, semantic
dependency, tool-call linkage, quality preservation or exact provider usage.

## Evaluation-only evidence and join result

Solved/unsolved outcome, stage IDs, step IDs, incorrect labels and unuseful
labels remain in `evaluation/labels.json` only. The label file has 24
trajectory IDs, 23 labelled stages and 60 labelled steps, while normalized
traces and source-event records contain no `stage_id` or `step_id` fields.

The current accepted artifacts therefore do not support an exact join of
`normalized block/request -> upstream step ID -> evaluation label`. A
positional mapping from message index, assistant turn index or step count
would be an unsafe inference and is rejected. A future exact join requires an
explicit source mapping captured at import time, or a separately verified
upstream mapping table. Labels must remain outside planner inputs either way.

## Unsafe inference rejected

This study rejects the following proposed shortcuts:

- optional because a tool result is old, repeated, low-scoring, non-gold or
  not referenced later;
- required because a trajectory solved, because context is gold, or because a
  block is large or early;
- stale because a block is old or no longer appears in a later prefix;
- dependency because blocks are adjacent, topically similar, chronologically
  related, or because a result appears after an action without an explicit ID;
- action/result linkage from message position alone;
- semantic zones from natural-language content;
- evaluation labels as removability or safety evidence;
- provider usage from provider/model identity; and
- exact provider token usage from surrogate character counts.

## Proposed provenance model

The next evidence adapter should make origin auditable per imported or derived
field without overloading the existing booleans:

```text
EvidenceOrigin = source_explicit | derived_structural | unknown

EvidenceProvenance {
    origin: EvidenceOrigin,
    source_locator: {
        trajectory_id,
        source_file_sha256,
        source_event_index,
        source_event_id,
        upstream_field_path?
    },
    derivation_rule?: stable_rule_id,
    derivation_inputs?: [source_locator],
    evaluation_only: bool
}
```

Safety-sensitive values should use an explicit evidence state such as
`value + provenance` or `unknown`, rather than a default `false` that hides
whether the source was silent. `source_explicit` is reserved for a source
field that directly states the fact. `derived_structural` is reserved for a
documented deterministic transformation. `unknown` carries no planner-safe
claim. Evaluation records remain in their existing external channel and must
never be copied into planner decision inputs.

## Conditional importer-revision design

No importer revision is implemented or authorized by this study. If raw
artifact access and upstream schema verification are separately resolved, the
minimum future design should be:

1. Keep the current hash-only content boundary and preserve the pinned
   trajectory/task/source-file provenance.
2. Capture explicit upstream message, action, observation, parent, stage and
   step identifiers only when the exact source fields exist; record their
   field paths and source hashes.
3. Add a typed provenance sidecar or later trace-schema extension carrying
   `source_explicit`, `derived_structural` and `unknown`, with stable rule IDs
   for derivations.
4. Separate raw message role from a versioned `protocol_kind` so a derived
   shell-observation classification is not confused with an upstream `user`
   role. This is a later planner-contract integration question, not a planner
   change in this task.
5. Preserve only explicit tool-call/result references. Keep chronology as
   chronology; never turn it into a semantic dependency.
6. Keep `required`, `optional`, `stale` and dependency values unknown unless
   explicit source evidence is verified. Do not manufacture positive planner
   gates from this corpus.
7. Maintain a separate evaluation sidecar. Add an exact block/step join only
   when the source provides a stable mapping; otherwise record no join.
8. Add deterministic import, provenance round-trip, missing-ID, label
   isolation, privacy, no-content-retention and negative inference tests.

Backward compatibility should keep existing trace-v2 readers working. A
typed sidecar is preferable to silently changing the meaning of existing
boolean fields, but it would not by itself create optional/stale/dependency
evidence or justify a planner rerun.

## Privacy and licence findings

The accepted CodeTraceBench revision’s metadata declares MIT and its primary
README states that the trajectory archives are released under MIT. The
README-linked root `LICENSE` file is absent at the exact revision, so the
missing-license-text limitation remains recorded and no text is recreated or
inferred. No raw archive, prompt, reasoning or tool output is present in the
tracked accepted evidence.

Any future adapter must retain the hash-only boundary, avoid copying command
or observation text, keep evaluation labels separate, and commit only compact
structural metadata and hashes. Source and archive hashes remain provenance
identifiers, not permission to redistribute raw artifacts.

## Decision-gate answers

1. **Does the underlying accepted artifact contain materially more useful
   structural evidence than Phase 1A preserved?** Not established. The
   available accepted evidence confirms roles, order, identity and provenance
   already preserved by Phase 1A; raw trajectory artifacts needed to verify
   additional explicit action/step/reference fields are not locally
   available.
2. **Can the available evidence be preserved or derived without semantic
   guessing?** Yes for identity, role, order, request prefixes, source-kind
   mapping and provenance. No for removability, stale state, semantic
   dependencies, exact tool relations, quality or provider usage.
3. **Is there enough planner-safe evidence to justify an importer revision and
   rerun?** Not for a meaningful positive Phase 1B characterization. A
   conditional provenance/identity design is justified, but implementation and
   rerun require raw-schema verification first.
4. **Which positive planner gates could this evidence legitimately exercise?**
   Protocol/system retention, current-request protection, chronology and
   conservative `DO_NOTHING`; no safe removal or relocation candidate is
   established.
5. **Which gates remain untestable?** Explicit optional/stale prune/defer,
   dependency-aware destructive decisions, non-message safe relocation,
   compression, provider/economic evidence and quality preservation.
6. **Can evaluation step IDs support an exact later post-hoc join?** Not with
   the accepted artifacts as currently preserved. The required mapping is
   absent; positional reconstruction is rejected.
7. **Should CodeTraceBench remain the Phase 1B corpus?** Retain it as a
   bounded structural/provenance corpus, but do not treat it as sufficient for
   positive planner coverage. Reassess only after exact raw-schema and
   identifier verification; do not broaden the corpus in this task.

## Phase 1B.2 assessment

`PIVOT`. The current evidence path is deterministic and privacy-conscious,
but it cannot establish enough planner-safe positive evidence to justify an
importer revision followed by a Phase 1B rerun. The next gate must resolve
exact raw-artifact availability and upstream identifier/schema facts before
any importer work is authorized.

Recommended next task, not started: a narrowly scoped raw-artifact access and
upstream-schema verification gate for this exact CodeTraceBench revision,
including explicit step/action/tool-reference fields and licence evidence.
If those facts remain unavailable or absent, review a separately authorized
corpus/evaluation-strategy pivot. Do not begin that task as part of this
closeout.
