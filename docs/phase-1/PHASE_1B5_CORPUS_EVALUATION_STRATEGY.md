# Phase 1B.5 - Corpus and Evaluation Strategy Review

Status: complete. Assessment: `PASS WITH RECORDED LIMITATIONS`.

Reviewed 2026-08-09 using repository documentation and public primary sources.
This is a research/design record only. It does not add an importer, corpus
adapter, planner rule, benchmark implementation, provider call, or raw third-
party data.

## Decision summary

The recommended next evaluation architecture is a hybrid (`E`) with two
separate validation tracks:

1. **Natural-workload observational validation.** Keep the frozen,
   hash-only CodeTraceBench slice as the primary source for natural agent
   structure, provider usage, source-explicit timestamps, provenance, and
   partial evaluation locators. Its intervention result remains 719/719
   `DO_NOTHING`; this is a valid safety-preserving result, not a reason to
   tune the planner or repeat the same characterization.
2. **Controlled intervention/quality validation.** Build a small
   Prefixity-controlled artifact from public, reproducible task/environment
   ideas, with paired and ablated traces, explicit action/result identity,
   explicit state or reference relationships, and an independent task-quality
   oracle. The controlled artifact is necessary to establish load-bearing
   context experimentally. It must be provider-neutral and must keep
   evaluation labels outside planner inputs.

The best public seed for the controlled track is **AppWorld** for its
state-based task evaluation, public task/world design, provider-neutral
simulated applications, and public/plain-text portion under Apache-2.0.
Protected task/app/API material is distributed in encrypted bundles under
Apache-2.0 with an additional encrypted-redistribution requirement. Any
future Prefixity use must pin and audit the exact material used before
implementation and should not copy protected raw data into the repository.
**ToolSandbox** is the strongest schema and ablation reference for explicit
message/tool/state/milestone structure, but its Apple-specific licence and
weaker maintenance make it a poor raw corpus dependency.

The exact next task is:

> **Phase 1B.6 - Controlled intervention benchmark design and seed audit.**
> Pin the selected public environment/task reference, define a small
> provider-neutral trace and evaluation schema, specify paired/ablated cases
> and leakage/privacy rules, and approve the seed set before implementing the
> benchmark or adapter.

This recommendation is deliberately not a claim that any current public
benchmark already supplies Prefixity's optional, stale, dependency,
removability, invalidation, or action-result safety evidence. Those meanings
must be source-explicit or established by a controlled experiment; timestamp
age, adjacency, and evaluation labels are not substitutes.

## 1. Evidence requirements derived from Phase 1B.1-Phase 1B.4

| Need | Phase 1B.4 fact | Requirement for the next validation track |
| --- | --- | --- |
| Natural workload | CodeTraceBench contains useful multi-turn structural observations and provider/source metadata. | Keep its pinned 24 trajectories / 719 request traces as observational evidence. |
| Provider evidence | All 719 accepted traces retain explicit raw usage; response metadata is preserved. | Do not replace this corpus merely to obtain intervention labels. |
| Time evidence | 1,498/1,498 source events retain explicit numeric timestamps. | Preserve source-explicit time; never turn age into staleness. |
| Provenance | Hash-only source identity and bounded locators are auditable. | Preserve hash-only privacy and pin every external input. |
| Evaluation joins | 32 steps have exact bounded source-event joins; 28 remain unresolved. | Add exact event identity where the source provides it; leave unresolved joins absent. |
| Action/result identity | Not established in CodeTraceBench. | Capture an explicit action/tool request and the corresponding result/observation identity. |
| Dependency | Not established in CodeTraceBench. | Use source-declared references/state transitions or a controlled causal ablation; do not use adjacency. |
| Load-bearing context | Not established in CodeTraceBench. | Measure whether removing a bounded context item changes the independently scored task outcome. |
| Optional/removable/stale | Not established in CodeTraceBench. | Treat these as absent unless the source or experiment genuinely establishes them. |
| Task quality | No causal task-success or savings/latency join is available for the planner result. | Keep quality labels in the evaluation sidecar and outside planner inputs. |
| Reproducibility | Frozen re-import and planner characterization are deterministic. | Prefer replayable local environments and deterministic import; record environment/version pins. |

The controlled track should therefore answer a different question from the
natural track: not "does this trace look repetitive?" but "does a documented
context intervention preserve or break an independently measured task
outcome, and can the source relationship be audited?"

## 2. Evidence classification rule

The existing Prefixity taxonomy is applied conservatively:

| Class | Use in this review |
| --- | --- |
| `CAPTURED_EXPLICIT` | The public artifact directly records the message, action, result, observation, state, reference, or source-declared evaluation field. |
| `DERIVED_STRUCTURAL` | A deterministic identity, ordering, path, hash, or unique join is computed without assigning a new semantic meaning. |
| `EVALUATION_ONLY` | A gold task, reward, milestone, state-diff, test, or ablation outcome is used to assess quality; it is not planner input. |
| `INFERRED_UNSAFE` | A semantic claim such as optional, stale, dependent, required, or removable is guessed from adjacency, age, repetition, content markers, or a convenient reference trajectory. It is rejected. |
| `ABSENT` | The source does not establish the evidence, or the reviewed artifact does not expose it in a stable form. |

In particular, an action identifier is not by itself a dependency edge, a
reference trajectory is not necessarily the only correct trajectory, and a
successful evaluation label is not evidence that a context block is safe to
remove.

## 3. Sources and review boundary

The accepted natural source is the exact CodeTraceBench revision
[`aa213b84ffb6690fc37ca15766d6ca174ec36d4d`](https://github.com/NJU-LINK/CodeTraceBench/tree/aa213b84ffb6690fc37ca15766d6ca174ec36d4d),
`verified` split, with the existing 24-trajectory selection. It remains the
only frozen corpus in the current implementation.

Candidate repositories were reviewed at their public default `main` pages on
2026-08-09. These are reconnaissance references, not import pins. Any next
implementation must record an immutable commit/tag, the task/data release,
licence text, and a small source manifest before ingestion. No candidate raw
dataset was downloaded and no provider/model call was made.

The primary sources reviewed were:

- [sierra-research/tau2-bench](https://github.com/sierra-research/tau2-bench),
  including its [task schema and evaluation documentation](https://github.com/sierra-research/tau2-bench/blob/main/docs/evaluation.md)
  and [MIT licence](https://raw.githubusercontent.com/sierra-research/tau2-bench/main/LICENSE);
- [apple/ToolSandbox](https://github.com/apple/ToolSandbox), its
  [trajectory/state/tool documentation](https://raw.githubusercontent.com/apple/ToolSandbox/main/README.md),
  and its [Apple Software licence](https://github.com/apple/ToolSandbox/blob/main/LICENSE);
- [StonyBrookNLP/appworld](https://github.com/StonyBrookNLP/appworld), its
  [public/plain-text Apache-2.0 licence](https://github.com/StonyBrookNLP/appworld/blob/main/LICENSE),
  encrypted-bundle/data-boundary documentation, and the [AppWorld ACL paper](https://aclanthology.org/2024.acl-long.850.pdf);
- [ServiceNow/BrowserGym](https://github.com/ServiceNow/BrowserGym) and its
  [Apache-2.0 licence](https://raw.githubusercontent.com/ServiceNow/BrowserGym/main/LICENSE);
- [web-arena-x/webarena](https://github.com/web-arena-x/webarena), including
  its [Apache-2.0 licence and public trajectory statement](https://github.com/web-arena-x/webarena);
- [ethz-spylab/agentdojo](https://github.com/ethz-spylab/agentdojo), including
  its [MIT licence](https://github.com/ethz-spylab/agentdojo/blob/main/LICENSE);
- [SWE-bench](https://github.com/SWE-bench/SWE-bench), including its
  [MIT licence and containerized evaluation description](https://github.com/SWE-bench/SWE-bench).

## 4. Serious candidate review

### 4.1 tau2-bench / current tau-bench line

**Source and availability.** The current repository describes a simulation
framework across mock, airline, retail, telecom, and banking-knowledge
domains, with turn-based tool use and a voice mode. The public repository has
task data, source, tests, and a MIT licence. Its README records a July 2026
v1.0.1 grading update and warns that affected scores are not comparable with
pre-v1.0.1 results; a future import must therefore pin a release/commit, not
use an unqualified branch.

**Artifact size and diversity.** The reviewed documentation exposes the task
and simulation layout, but does not establish one stable offline total of
completed trajectories suitable for a Prefixity corpus. The framework spans
multiple domains and provider adapters; generated trajectories depend on the
selected agent/user models. That is useful diversity, but generation requires
provider configuration and is outside this task.

**Evidence classification.**

- `CAPTURED_EXPLICIT`: simulated tool calls, arguments, tool results, turns,
  task identifiers, policies, and evaluation criteria are part of the model;
  action records have explicit action identity in task examples.
- `DERIVED_STRUCTURAL`: replay order and deterministic event numbering can be
  used for bounded joins if the selected output artifact records them.
- `EVALUATION_ONLY`: database end-state comparison, environment assertions,
  communication checks, natural-language assertions, action checks, and reward
  basis are scoring mechanisms. The project explicitly documents that a
  reference `evaluation_criteria.actions` sequence is generally one way to
  reach a target state, not automatically the only required trajectory.
- `INFERRED_UNSAFE`: treating reference actions as required, treating adjacent
  calls as dependent, or treating a reward success as removability.
- `ABSENT`: source-established optional, stale, removable, or context-block
  labels; a stable, provider-neutral, already-generated public trajectory
  corpus with a frozen count.

**Replay and engineering.** The simulated environment and task evaluator are
good foundations for replay and action/result quality joins. Deterministic
import of an existing local output should be feasible after pinning the
schema; deterministic trajectory generation is a separate problem. Effort is
medium to high because the current release spans domains, model adapters, and
changing evaluation rules. It can exercise task success and action/result
paths, but it does not by itself exercise safe context removal or staleness.

**Privacy and leakage.** Domain state is simulated, but model-generated
transcripts and knowledge/voice outputs can still contain provider content.
Do not commit runs, credentials, prompts, or archives. Use task metadata and
hash-only manifests until a licence and privacy review approves any artifact.

**Disposition.** Strong controlled-environment candidate and useful reference
for outcome semantics; not selected as Prefixity's first raw corpus.

### 4.2 Apple ToolSandbox

**Source and availability.** ToolSandbox is a public stateful,
conversational, interactive benchmark. Its README describes execution state,
dialog history, world-state snapshots, explicit tool results, a user role, and
scenario milestones. It also describes generated `conversation.json`
artifacts under `data/`. The reviewed repository page showed a small project
history and the README does not establish a fixed public trajectory count.

**Evidence classification.**

- `CAPTURED_EXPLICIT`: roles, message visibility, tool requests, captured
  stdout/stderr results, turn snapshots, and world-state changes.
- `DERIVED_STRUCTURAL`: bounded turn ordering and identity projections;
  directional milestone-DAG order may be used only where the scenario source
  explicitly supplies it.
- `EVALUATION_ONLY`: intermediate/final milestone satisfaction and similarity
  checks.
- `INFERRED_UNSAFE`: converting a state dependency category or a milestone
  failure into a universal removable/stale label for arbitrary context.
- `ABSENT`: a broad, stable corpus-wide annotation for optional/stale context
  and a stable public completed-trajectory count.

**Replay and engineering.** This is the closest existing candidate to the
desired explicit trace shape and a good design reference for paired state
ablations. Importing an existing trajectory JSON is plausibly deterministic;
generating trajectories requires model/API configuration. Effort is low to
medium for a small scenario subset, but a full dependency would add an Apple-
specific licence and older environment constraints.

**Licence, privacy, and leakage.** The repository uses an Apple Software
licence rather than MIT/Apache. It permits use, modification, and some
redistribution subject to its notice, disclaimer, and acknowledgement terms;
this is not a reason to copy raw trajectories without a separate legal/data
review. Example worlds are synthetic, but generated dialogs can contain raw
model output and API-linked content. Keep only hashes/structural manifests.

**Disposition.** Strong schema/ablation reference; rejected as the primary
external corpus because of licence friction, maintenance risk, and the lack
of a frozen public corpus count.

### 4.3 AppWorld

**Source and availability.** AppWorld is a public project whose
public/plain-text portion is Apache-2.0, with a controllable simulated world
of day-to-day applications. Protected task/app/API material is distributed in
encrypted bundles under Apache-2.0 with an additional encrypted-
redistribution requirement. The ACL paper describes 750 tasks, 9 apps, 457
APIs, and approximately 100 simulated people. The repository documents
task/world artifacts, train/dev/test splits, API calls and responses, database
state, ground-truth evaluation programs, and local or containerized
replay/evaluation. The number of paper tasks is a benchmark fact, not a claim
that 750 completed agent trajectories are publicly available.

**Evidence classification.**

- `CAPTURED_EXPLICIT`: task IDs, initial world state, API names/arguments,
  API responses, execution events, application state, and source-declared
  task/evaluation fields where present.
- `DERIVED_STRUCTURAL`: API call order, state diffs, and unique event joins
  computed by a deterministic replay/import layer.
- `EVALUATION_ONLY`: ground-truth programs, state-based unit tests, task
  success, and collateral-damage checks. These labels must remain outside
  planner inputs.
- `INFERRED_UNSAFE`: assuming every API call is required, or assuming an
  application state field is safe to remove because a particular solution did
  not read it.
- `ABSENT`: a ready-made Prefixity context-removability label set and a
  stable public corpus of natural multi-turn model trajectories.

**Replay and engineering.** AppWorld is the strongest public source for a
controlled quality oracle and provider-neutral state transitions. It is high
engineering effort for Prefixity because the agent may operate through code
execution/API calls rather than a simple message/tool/result stream; a narrow
adapter would have to preserve event identity without importing raw code or
data. Replay is feasible only with pinned package, task release, runtime, and
database artifacts.

**Licence, privacy, and leakage.** The public/plain-text portion is
Apache-2.0. Protected task/app/API material is distributed in encrypted
bundles under Apache-2.0 with an additional encrypted-redistribution
requirement. Any future Prefixity use must pin and audit the exact material
used before implementation. The repository also documents separate ground
truth availability by split and a request not to post extracted code/data.
Protected raw-data copying and test-set ingestion into Prefixity are therefore
inappropriate. The safe route is an external pinned dependency or a
self-authored small task subset with no protected third-party bundle content.

**Untested decision path and disposition.** AppWorld can support task-success,
state-dependency, and collateral-damage experiments, which are materially
closer to Prefixity's missing safety evidence than CodeTraceBench. It cannot
be treated as evidence that a context block is removable until paired
ablation cases are authored and evaluated. Select as the primary seed/design
reference for the controlled track, subject to the Phase 1B.6 licence and
data-boundary audit.

### 4.4 BrowserGym and WebArena

**Source and availability.** BrowserGym is a public Apache-2.0 framework for
browser tasks and exposes reset/step observations, actions, rewards, and task
evaluation. Its ecosystem covers several benchmark families and points to
trajectory artifacts released separately. WebArena is a public Apache-2.0,
self-hostable web environment; its README reports roughly 170 released human
annotator trajectories and execution trajectories.

**Evidence classification.**

- `CAPTURED_EXPLICIT`: browser actions, observations, environment steps, and
  task identifiers; WebArena also exposes execution/human trajectory
  artifacts where available.
- `DERIVED_STRUCTURAL`: step order and page/state transition structure.
- `EVALUATION_ONLY`: task success/reward and benchmark-specific checks.
- `INFERRED_UNSAFE`: treating a page observation, DOM node, or adjacent browser
  step as optional, stale, or dependent without a controlled causal test.
- `ABSENT`: stable context-block removability labels, provider usage metadata,
  and a compact privacy-compatible corpus that is already aligned to
  Prefixity's context model.

**Replay, privacy, and engineering.** BrowserGym can provide quality and
action/observation evaluation, but reproducibility depends on browsers,
Docker, websites, snapshots, and benchmark-specific services. Web content,
screenshots, account-like state, and prompt/task text increase privacy and
leakage review cost. Deterministic import of a frozen trajectory is possible;
deterministic environment replay is materially harder. Engineering effort is
high and provider neutrality is possible only above the environment wrapper.

**Disposition.** Keep as a future web-specific option, not the next Prefixity
source. It exercises action/observation quality but does not isolate context
removability and introduces disproportionate infrastructure and leakage risk
for the immediate gate.

### 4.5 AgentDojo

AgentDojo is a public MIT benchmark for dynamic environments, tool use, and
prompt-injection attack/defence evaluation. It is relevant to safety and
stateful tool interaction, but the reviewed project is primarily a security
benchmark rather than a context-management/removability corpus. The README
does not establish a stable offline completed-trajectory count for Prefixity.

`CAPTURED_EXPLICIT` environment/task/tool interactions and
`EVALUATION_ONLY` security/task outcomes are useful secondary evidence.
Dependency, optional, stale, and removable context labels remain `ABSENT`;
deriving them from attack success would be `INFERRED_UNSAFE`. It is provider
neutral at the environment interface but normally requires running model
evaluations. Disposition: reject for the next track; retain as a later
security-specific validation candidate.

### 4.6 SWE-bench

SWE-bench is a public MIT task/evaluation artifact, including a 500-task
Verified subset and containerized patch evaluation. It supplies strong
task-quality and replay-oriented testing, but it is not a multi-turn agent
trajectory corpus. The patch, tests, and issue are not an explicit
action/result trace, a context dependency graph, or a removable-context
annotation.

Its task tests are `EVALUATION_ONLY`; any call/step structure generated by a
future agent run would need to be captured separately. It is provider neutral
at the task evaluator but high effort and code-content-heavy for this project.
Disposition: reject as the next corpus; consider only if Phase 1B later needs a
software-task quality oracle independent of tool-use context.

## 5. Evidence-gap comparison matrix

The cells below classify evidence actually exposed by the reviewed artifact,
not what could be invented by an adapter. `E` means `CAPTURED_EXPLICIT`, `D`
means `DERIVED_STRUCTURAL`, `V` means `EVALUATION_ONLY`, `U` means
`INFERRED_UNSAFE`, and `A` means `ABSENT`. A slash means the artifact exposes
the first class for one bounded aspect but not the stronger Prefixity
semantic claim.

| Phase 1B.3/Phase 1B.4 gap | CodeTraceBench | tau2-bench | ToolSandbox | AppWorld | BrowserGym/WebArena | Controlled paired/ablated artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Natural multi-turn context | E | E | E | E/D | E | E (target) |
| Action/tool identity | A | E | E | E | E | E (target) |
| Result/observation identity | A | E | E | E | E | E (target) |
| Explicit parent/reference edge | A | A/D | E (scenario milestone/state graph) | A/D (state transitions) | A/D | E (target, only if authored/source-declared) |
| Environment/state dependency | A | E | E | E | D | E (target) |
| Load-bearing context block | A | A | V/E (scenario milestone effect) | V/E (state-test effect) | A | V/E (paired ablation) |
| Optional/removable/stale meaning | A | A | A | A | A | V/E only after controlled design |
| Exact quality join | V partial | V | V | V | V | V (target) |
| Successful and failed outcomes | A/V partial | V | V | V | V | V (target) |
| Replayable local environment | A | E/D | D | D/E | D | E (target) |
| Provider usage/metadata | E | A/adapter-dependent | A/adapter-dependent | A/adapter-dependent | A/adapter-dependent | E (target, synthetic/provider-neutral) |
| Privacy-compatible hash reduction | E | D | D with licence review | D with encrypted-data boundary | D with web-content review | E (target) |
| Currently untested planner path | No | Outcome/action only | State/milestone only | State/collateral-damage | Browser quality only | Yes, contingent on evidence contract |

The matrix does not authorize a planner change. In the proposed controlled
artifact, paired-ablation outcomes are evaluation evidence. If the planner is
ever allowed to consume a source-declared dependency or reference, that fact
must be represented separately from the evaluation label and must pass a new
decision-contract gate.

## 6. Strategy classes A-E

### A. Add a second existing public corpus

Useful for triangulation, especially tau2-bench or AppWorld, but no reviewed
corpus supplies all of the missing Prefixity semantics. Adding one now would
increase adapter and privacy surface without guaranteeing a tested
intervention path. Keep as a future option after the controlled schema exists.

### B. Replace CodeTraceBench for intervention-quality evaluation

Do not replace it. CodeTraceBench is the only accepted natural-workload
observation source with the current provider/timestamp/provenance evidence and
deterministic 24/719 corpus identity. A replacement would trade away verified
observational facts while still requiring controlled ablations.

### C. Construct a small controlled benchmark

Necessary for Prefixity's causal intervention hypothesis. The benchmark can
be small and synthetic, with explicit tool/result IDs, authored dependency or
state references, paired removal variants, task outcomes, and an independent
quality oracle. It should use public environments only as pinned inspiration
or an external runner, not as raw data copied into the repository.

### D. Paired/ablated trajectories

This is the evaluation method inside the controlled track, not a substitute
for source provenance. For each baseline trace, create a minimally changed
variant that removes or relocates one bounded context item, replay both under
the same task/world seed, and record task quality as `EVALUATION_ONLY`. The
variant must be linked to the baseline by a manifest ID, while the planner
must not receive the gold outcome label.

### E. Hybrid natural plus controlled validation

This is the recommendation. It preserves CodeTraceBench's observation role and
adds only the small, purpose-built evidence needed to assess intervention
quality. It also prevents a controlled synthetic benchmark from being
mistaken for a representative natural workload.

## 7. Recommended validation architecture

```text
CodeTraceBench (pinned, hash-only)
        |
        +--> Track 1: natural workload observation
              provider usage, timestamps, provenance, structural audits
              frozen planner remains conservative

Public environment/task reference (pinned, no raw bundle copied)
        |
        +--> Phase 1B.6 controlled artifact design
              explicit events + paired/ablated variants + independent oracle
        |
        +--> Track 2: intervention/quality validation
              planner inputs: only permitted source/structural evidence
              evaluation sidecar: success, failure, ablation and quality joins
```

The tracks share bounded IDs and manifests, not raw content. The controlled
track must include at least: task seed, baseline/variant ID, event ID,
action/result or observation reference, state/reference edge when genuinely
available, source locator, environment version, provider-neutral executor,
quality outcome, and an import/manifest hash. Raw prompts, model reasoning,
tool payloads, screenshots, archives, and third-party bundles remain outside
the repository unless separately approved and sanitized.

## 8. Licence, privacy, and provenance decision

AppWorld's public/plain-text portion is Apache-2.0. Protected task/app/API
material is distributed in encrypted bundles under Apache-2.0 with an
additional encrypted-redistribution requirement. Any future Prefixity use
must pin and audit the exact material used before implementation, and should not
copy protected raw data into the repository. tau2-bench is MIT and has clear
task/evaluation source, but its current grading changes require release
pinning and its generated trajectories require model/provider setup.
ToolSandbox's Apple Software licence requires notice/acknowledgement handling
and warrants a separate legal review. BrowserGym and WebArena are Apache-2.0,
but web content, screenshots, environment images, and website drift create
more privacy and reproducibility exposure. AgentDojo and SWE-bench are MIT,
but their primary evaluation questions do not match the missing Prefixity
intervention evidence.

The required provenance boundary for the next task is:

- record source URL, immutable revision/tag, licence text URL, task/data
  release, and retrieval date in a manifest;
- keep external raw data outside Git and never commit archives or trajectory
  content;
- hash source artifacts and structural records after deterministic canonical
  serialization;
- distinguish source-explicit fields from deterministic projections and
  evaluation-only labels;
- prohibit provider calls in importer/tests and use scripted or fixture-based
  executors for determinism;
- audit test-set leakage before any quality result is treated as evidence.

## 9. Phase 1B.5 assessment

**Assessment: `PASS WITH RECORDED LIMITATIONS`.**

A defensible next strategy is identified: retain CodeTraceBench for natural
observational validation and add a small controlled paired/ablation track for
intervention quality. The limitation is material: no existing public corpus
reviewed here simultaneously supplies a compact, privacy-compatible,
provider-neutral, frozen trajectory set with explicit Prefixity context
removability semantics. The controlled artifact remains to be designed and
audited. This assessment is based on evidence coverage, not on a desire to
produce positive planner interventions.

The next task must not begin until this record is accepted. It should not
change the frozen planner or retroactively reinterpret Phase 1B.4's 719/719
`DO_NOTHING` result.
