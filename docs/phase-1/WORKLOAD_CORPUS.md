# Phase 1 Workload Corpus

## Purpose

Phase 1 must evaluate Prefixity on workloads that are not designed to make Prefixity look good.

Prefer established public corpora over inventing a new benchmark.

## Primary candidate: ContextBench

ContextBench is the preferred Phase 1A starting point because its research problem is closely aligned with Prefixity: how effectively agents gather and use context during software-engineering tasks.

Use it as an **external benchmark/evaluation source**, not as logic embedded in `prefixity-core`.

### Intended Prefixity use

1. build a thin importer/adapter;
2. preserve task and trajectory identifiers;
3. map observations/messages/tool results into Prefixity blocks;
4. preserve gold-context references as evaluation-only labels;
5. run Prefixity observation/recommendation offline;
6. compare decisions with known relevant/load-bearing context.

### Initial slice

Start with 20–50 tasks, including different trajectory lengths, successes/failures where available, repeated reads/tool outputs, and cases where `DO_NOTHING` may be correct.

Expand only after the importer and evaluation contract are stable.

## Secondary public trajectory corpora

A second corpus may be useful after the ContextBench path works, especially for heterogeneous agent frameworks or longer trajectories.

**Do not ingest or redistribute any secondary corpus until its current licence and redistribution terms are checked and recorded.**

## Corpus acceptance checklist

Record:

- source name and URL;
- version/tag/commit/dataset revision;
- licence and redistribution constraints;
- task and trajectory IDs;
- agent/framework and model where known;
- success/failure label where known;
- source revision/timestamp where available;
- whether raw content may be stored;
- whether derived excerpts may be committed;
- whether only hashes/metadata should be retained.

## Provenance requirements

Every normalized workload record must answer:

```text
Which corpus?
Which task?
Which trajectory?
Which source turn/tool event?
What transformation did the importer apply?
Was content retained, redacted, hashed or omitted?
Which labels are evaluation-only and hidden from Prefixity decisions?
```

## Evaluation leakage rule

Separate:

- **decision input** — what the real agent could have had at the turn;
- **evaluation labels** — gold context, success labels, later outcomes, human annotations;
- **analysis output** — Prefixity recommendation.

Evaluation labels must never influence the recommendation path.

## Unit of analysis

Prefer an ordered multi-turn trajectory, not an isolated prompt.

Preserve trajectory ID, turn index, chronology, block type, semantic zone, structural path if derivable, role, source reference, content hash, size/token information, dependency links where safely inferable, and required/optional/stale labels only when justified.

Do not manufacture `required=true` because a block merely looks important.

## Gold-context interpretation

A block absent from a gold set is **not automatically safe to remove**.

Distinguish:

- `gold_required`
- `protocol_required`
- `dependency_required`
- `unknown`

Only strong evidence should support `PRUNE`.

## Suggested metrics

- required/gold-context retention;
- safe-removal precision where labels support it;
- avoidable-context identification;
- structural churn;
- repeated reads/tool outputs;
- early divergence contribution;
- no-op quality.

## Local/private trajectories

Private trajectories are optional later. If added: explicit opt-in, local processing by default, hashes/metadata preferred, no private content in public fixtures, and documented retention/deletion.

## Phase 1A exit condition

At least one public corpus imports deterministically, provenance survives, labels are isolated from decision inputs, a representative subset analyses end-to-end, no mutation occurs, and licence/redistribution decisions are recorded.
