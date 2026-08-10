# ContextBench External Trajectory Admission

## 1. Decision

**NO-GO — NO PERMISSION-CLEARED EXTERNAL TRAJECTORY ARTIFACT FOUND**

This gate searched for an already-existing, independently authored coding-agent
trajectory artifact that is stably joinable to ContextBench, contains enough
chronological tool/context evidence for the preregistered study, is pinned to an
immutable revision, and is permission-cleared for bounded local research.

The search found a technically promising artifact: the public
`Contextbench/Tracebench` dataset contains real SWE-bench trajectories and has a
large exact join against the selected ContextBench IDs. It does not, however,
declare a dataset license, reuse terms, or permission for the raw trajectory
and source material. Public availability is recorded as access evidence, not
as an inferred redistribution or local-research grant. The candidate therefore
does not satisfy the permission gate.

No trajectory was generated, no provider/model call was made, no credential was
accessed, no repository or container was executed, and no raw trajectory or
benchmark row was added to Prefixity.

## 2. Scope and authoritative checkpoint

- Repository: `hourwise/Prefixity`
- Starting checkpoint: `6ef6fe6e4b271d0dc62fe5481a7c408e2db3c306`
- Starting commit: `docs: record ContextBench front-half admission no-go`
- CI #37: successful — [run 31379728346](https://github.com/hourwise/Prefixity/actions/runs/31379728346)
- Production planner: unchanged
- Research policy: `controlled-evidence-policy-v1`, not promoted
- Stage 1: **BLOCKED**

The protected `docs/tasks/ACTIVE.md` was not modified. Its required SHA-256
remained:

`D329C117BF346D65B2587B07EF9B13AA394E5796B580C623E71B1593853F17E2`

This document records an acquisition/admission gate only. It does not authorize
an adapter, schema smoke, replay, provider call, prompt mutation, raw-data
copying, or production-planner change.

## 3. ContextBench identity and pins

The corrected ContextBench source remains:

- Repository: [`EuniAI/ContextBench`](https://github.com/EuniAI/ContextBench)
- Repository revision:
  `1436c28a8eb95496da4ea69ad458b9f8a8eb7d61`
- Paper: [arXiv 2602.05892](https://arxiv.org/abs/2602.05892)
- Dataset: [`Contextbench/ContextBench`](https://huggingface.co/datasets/Contextbench/ContextBench)
- Dataset revision:
  `c2855792b006af41c67202d33883fb9d46362853`

The repository and its documentation describe 1,136 issue-resolution tasks from
66 repositories in eight languages, human-annotated gold contexts, and
trajectory-level context recall, precision, and efficiency. The pinned
ContextBench repository contains the evaluator and task/gold material, but no
bundled external agent-run collection suitable for this gate. Its runner
documentation describes how to execute agents and collect trajectories; that is
a generation path, not an already-existing external artifact.

The unrelated same-name repository `cioutn/context-bench` is not used as a
source, license basis, provenance basis, or trajectory source.

## 4. Admission rules applied

The review used only the following stable joins:

1. exact ContextBench `original_inst_id` to exact trajectory `task_name`;
2. exact upstream benchmark instance ID where the artifact documents one; or
3. an exact repository/task identifier with an independently pinned revision.

Fuzzy names, semantic similarity, manual patch matching, repository adjacency,
and final-patch similarity were not accepted as joins.

A trajectory was considered structurally sufficient only if the artifact or its
first-party description provides chronological agent actions and observations,
including tool calls/results, file views or searches, and edits or equivalent
action chronology. A patch, answer, aggregate score, or stage summary alone was
not sufficient.

Gold independence was classified as follows:

- `BLIND_TO_GOLD`: no ContextBench gold context is supplied to the trajectory
  input, subject to confirmation before any future raw extraction;
- `GOLD_CONDITIONED`: the trajectory or generation process uses ContextBench
  gold labels or equivalent annotations; and
- `UNKNOWN`: the public provenance is insufficient to decide.

Only `BLIND_TO_GOLD` is eligible. That classification never overrides a missing
license or unclear third-party provenance.

## 5. Preferred candidate: Contextbench/Tracebench

### 5.1 Immutable source and format

- Dataset: [`Contextbench/Tracebench`](https://huggingface.co/datasets/Contextbench/Tracebench)
- HF dataset commit: `7da2e4f45b330be8b6e8f1cff835247723cb3341`
- API observation: `gated: false`, `private: false`, last modified
  `2026-04-22T08:18:34Z`
- Manifest objects observed at that commit:
  - `bench_manifest.full.jsonl`: SHA-256
    `8f688bb147fb1840572887af10d137653b4b49523c5bbd3dc3ee9c333bbe082e`
  - `bench_manifest.full.parquet`: SHA-256
    `1330b1b7c35e4d563ff73dbe18fb046300cfb2e2efc70c1ca02092f1d33d13c5`
  - `bench_manifest.verified.jsonl`: SHA-256
    `5b33978effbeaa966d701160429729eb9631d524305abc870f9aa0d421ef8963`
  - `bench_manifest.verified.parquet`: SHA-256
    `2fbedb26641ad9843642c869674473defb3c606542a9c9215ebfba38755ebe14`
- Declared full split: 3,316 trajectories, comprising 2,670 TerminalBench
  and 646 SWE-bench trajectories.
- Declared verified split: 1,000 trajectories, comprising 489 SWE-bench and
  511 TerminalBench trajectories.
- Declared artifact availability: 3,291 of 3,316 full-split trajectories have
  `.tar.zst` artifacts; 25 OpenHands entries lack raw artifacts.
- Declared manifest fields include `traj_id`, `agent`, `model`, `task_name`,
  `task_slug`, step/stage summaries, `source_relpath`, and `artifact_path`.
- The public file tree contains `swe_raw` directories for mini-SWE-agent,
  OpenHands, and SWE-agent runs. A representative public file path is
  [`swe_raw/openhands__poly/microsoft__vscode-153857/gpt-5-1769074698.5290082.json`](https://huggingface.co/datasets/Contextbench/Tracebench/blob/main/swe_raw/openhands__poly/microsoft__vscode-153857/gpt-5-1769074698.5290082.json).

The manifest and public trajectory-file preview establish that this is a real
chronological trajectory collection rather than a final-patch-only result. No
archive was downloaded or extracted for this gate. The representative preview
shows agent execution and repository-context material; no such raw content was
copied into Prefixity.

### 5.2 Bounded exact-join result

The prior ContextBench preflight had a bounded selected set of 500 ContextBench
IDs: 174 `Verified`, 54 `Pro`, 116 `Poly`, and 156 `Multi`. The 646 SWE-bench
manifest rows were read through the public dataset-viewer metadata endpoint in
bounded pages and compared only by exact `original_inst_id == task_name`.

| ContextBench source | Exact task IDs joined | Trajectory rows joined | Rows with artifact path |
| --- | ---: | ---: | ---: |
| `Verified` | 162 | 176 | 176 |
| `Pro` | 46 | 104 | 104 |
| `Poly` | 85 | 208 | 208 |
| `Multi` | 83 | 108 | 108 |
| **Total** | **376** | **596** | **596** |

This is a lower-bound study result over the selected 500-task slice, not a
claim that all 1,136 ContextBench tasks have been joined. It already exceeds the
desired 50-task and minimum 25-task thresholds technically. The exact join is
strong enough to reopen the technical question, but not the permission
question.

### 5.3 Independence and provenance

At the metadata level, Tracebench identifies its SWE material as SWE-bench and
does not expose ContextBench `gold_context`, `gold_context` spans, or other
ContextBench gold-label fields in the trajectory manifest schema. The bounded
classification is therefore `BLIND_TO_GOLD` with respect to ContextBench gold,
pending any future raw-artifact inspection. No gold label was used as Prefixity
planner or evidence input during this review.

The trajectory collection is nevertheless derived from several upstream
benchmark families and their source repositories. The relevant project-level
licenses do not settle the rights to the combined task, source, prompt,
patch/test, tool-output, and trajectory material:

- [SWE-bench](https://github.com/SWE-bench/SWE-bench) declares MIT for its
  project. That does not make every third-party repository, issue, source
  excerpt, patch, or trajectory output MIT-licensed.
- [SWE-PolyBench](https://github.com/amazon-science/SWE-PolyBench) declares MIT
  for its project, while its tasks reference separate repositories and source
  material.
- [Multi-SWE-bench](https://github.com/multi-swe-bench/multi-swe-bench) declares
  a project license, but its underlying tasks and repositories remain separate
  provenance subjects.
- SWE-bench Pro includes public, held-out, and commercial task material in its
  documented benchmark design; a public subset or a downstream trace path is
  not a blanket redistribution grant.

The Tracebench HF API metadata has no `license` tag, no `LICENSE` or terms file
in the repository file listing, and no reuse permission in the dataset README.
The dataset is publicly accessible and ungated, but the permission status of
raw trajectories and embedded third-party material is **UNCLEAR**.

### 5.4 Classification

`PERMISSION_UNCLEAR`

Sub-findings:

- trajectory sufficiency: technically sufficient by declared artifact format
  and bounded manifest/file-preview evidence;
- exact joinability: technically sufficient for 376 selected task IDs and 596
  trajectory rows;
- gold independence: `BLIND_TO_GOLD` at metadata level;
- immutable pin: sufficient at the HF dataset commit and manifest-object level;
- local research permission: **not established**;
- copy/vendor/redistribute permission: **not established and not inferred**.

Tracebench is the preferred candidate for the single follow-up strategy, but it
is not an admitted Prefixity input.

## 6. Other candidates searched

| Candidate | What was found | Classification | Blocking fact |
| --- | --- | --- | --- |
| Pinned `EuniAI/ContextBench` repository | Evaluator, parsers, gold/task material, and trajectory format contracts; no bundled run collection | `TRAJECTORY_INSUFFICIENT` | Would require generating or separately obtaining trajectories |
| Pinned `Contextbench/ContextBench` HF dataset | 1,136 task/gold rows and source metadata; no trajectory field | `GOLD_CONDITIONED` / `TRAJECTORY_INSUFFICIENT` | Gold material is not an external agent trajectory |
| `Contextbench/Tracebench` HF dataset | 3,316 real trajectories; 646 SWE rows; 596 exact joined rows in the selected slice | `PERMISSION_UNCLEAR` | No license, terms, or raw-material reuse grant |
| [`SWE-bench/experiments`](https://github.com/SWE-bench/experiments) | Official repository documents per-instance `trajs/` and execution logs and uses exact SWE-bench task IDs | `ADMISSIBLE_METADATA_ONLY` | Actual logs/traces are in public S3 and require an AWS account; no repository license or immutable S3 artifact hash was established |
| ContextBench website and leaderboard | Aggregate model/agent results and benchmark descriptions | `REFERENCE_ONLY` | No consumable trajectories |
| Existing Prefixity CodeTraceBench slice | Existing 24-trajectory observational source remains pinned separately | `JOIN_INSUFFICIENT` | Prior ContextBench preflight found zero joined candidates; no widening was authorized |
| Generic Agentless, SWE-agent, mini-SWE-agent, and OpenHands repositories/examples | Parser/framework code, examples, or result summaries | `JOIN_INSUFFICIENT` | No independently pinned exact ContextBench trajectory artifact found in the inspected public surfaces |
| Unrelated `cioutn/context-bench` | Same-name project only | `REFERENCE_ONLY` | Explicitly excluded from identity, license, and provenance review |

The official SWE-bench experiments repository is useful evidence that upstream
trajectory records exist, but its S3 indirection and missing rights/hash
evidence prevent it from satisfying this gate without a separate authorization
and source-owner clarification.

## 7. Permission and raw-data boundary

The following distinctions remain mandatory:

1. ContextBench framework/code license: Apache-2.0 at the pinned repository.
2. ContextBench distributed task/gold dataset: public, but its current dataset
   metadata does not provide a blanket raw-data license.
3. Tracebench artifact and manifest: public/ungated at the observed HF commit,
   but no artifact/data license or reuse terms were declared.
4. Upstream benchmark tasks and repositories: separately licensed and
   provenance-specific; project licenses do not override their terms.
5. Local research consumption: not admitted for raw Tracebench material until
   the owner or a clear license establishes permission.
6. Copying, vendoring, or redistribution in Prefixity: not authorized and not
   performed.

No raw ContextBench rows, gold contexts, patches, source bodies, cloned
third-party repositories, Tracebench archives, or trajectory files are staged
or tracked by this change. Only bounded public metadata and source links are
recorded.

## 8. Effect on the front-half experiment

The existence of Tracebench changes the technical diagnosis from “no external
trajectory artifact was visible” to “a technically joinable artifact exists but
is not permission-cleared.” It does not authorize the ContextBench adapter or
change the previous study result. No adapter, synthetic trajectory, front-half
score, ContextBench gold comparison, provider call, or replay was performed.

The front-half experiment therefore remains **not admitted**. Stage 1 schema
smoke remains blocked, and Stage 2 replay remains blocked. The production
planner and `controlled-evidence-policy-v1` remain unchanged.

## 9. Stage 1 readiness and stopping decision

Stage 1 is **BLOCKED**. It may not begin unless all of the following are later
separately authorized and evidenced:

- a permission-cleared, immutable Tracebench slice or equivalent artifact;
- confirmation that the selected raw trajectories are blind to ContextBench
  gold labels;
- a source-specific review of task, repository, prompt, patch, test, and
  trajectory rights;
- a bounded no-vendoring acquisition plan outside the Prefixity repository;
- explicit authorization for any adapter or schema-smoke implementation.

Exactly one recommended next task:

> **Obtain written reuse clarification or an explicitly licensed, immutable
> Tracebench trajectory slice from the source owner, then repeat the admission
> review.**

That task is recommended, not started here.

## 10. Validation and completion boundary

- Checkpoint and remote `main` were verified at
  `6ef6fe6e4b271d0dc62fe5481a7c408e2db3c306`.
- CI #37 was verified successful for that checkpoint.
- The ContextBench repository, paper, dataset, Tracebench commit, manifest
  objects, and upstream source links were checked for identity consistency.
- The bounded exact-join computation used only public manifest metadata and
  exact IDs; no fuzzy or patch-based join was used.
- No provider/model inference call, credential access, agent execution, Docker
  execution, replay, prompt mutation, adapter, or score was performed.
- `docs/tasks/ACTIVE.md` was preserved byte-for-byte and remains unstaged.
- This gate is complete at the research/admission boundary and must stop before
  commit, push, or any follow-on experiment.
