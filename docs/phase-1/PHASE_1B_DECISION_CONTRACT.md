# Phase 1B.0 Intervention Decision Contract

Status: implemented as an offline, conservative baseline. This document does
not authorize Phase 1C replay, live provider calls, prompt mutation or
automatic compression.

## Contract

`prefixity-core::decision::InterventionPlan` is the authoritative Phase 1B
contract. Its `contract_version` is currently `1`. Each plan records the
source trace identity, retained block IDs, deterministic recommendations and
fixed offline-safety notes. Every recommendation is marked
`hypothetical_only: true`.

The contract has exactly these classes:

- `KEEP` — retain a block at its recorded position.
- `DEFER` — hypothetical omission of explicitly optional material until it is
  requested; no order change is applied.
- `PRUNE` — hypothetical omission of an explicitly safe candidate; no source
  trace is changed.
- `RELOCATE_CANDIDATE` — hypothetical within-zone relocation candidate; the
  source order is never changed.
- `COMPRESS_CANDIDATE` — contract-only class reserved for later evidence; the
  Phase 1B.0 baseline never emits it.
- `DO_NOTHING` — retain the complete recorded context and order when no
  intervention is sufficiently justified.

When an intervention is proposed, the plan contains one auditable record per
block, including `KEEP` records for blocks not selected for intervention. If
no intervention is justified, it contains one trace-level `DO_NOTHING` record.

Each record includes deterministic reason codes, a human explanation, evidence
strength, relevant dependencies, expected structural effect, expected quality
risk, provider-state dependence, provider/economic evidence-presence flags,
separately represented structural/provider-cache/economic/quality/dependency
evidence, and the hypothetical-only marker.

## Phase 1B.0 baseline rules

The planner validates the trace and reuses existing analysis, prefixity scores,
semantic zones and the zone-constrained `StablePrefixPolicy` safety logic. It
only reads the trace.

- Explicit `required` blocks are retained.
- System/tool protocol content and current/user request content are protected
  from destructive intervention.
- Missing dependency references and dependency cycles make safety evidence
  uncertain; the planner fails open to retention or `DO_NOTHING`.
- A `PRUNE` candidate requires explicit `optional` and `stale` flags, a known
  tool-result source, no retained transitive dependent, and a non-chronological
  zone.
- A `DEFER` candidate requires explicit `optional` metadata, a known
  non-stale tool-result source, supporting low prefixity volatility evidence,
  no retained transitive dependent, and a non-chronological zone. The score is
  supporting evidence, never the safety proof.
- A `RELOCATE_CANDIDATE` may only describe an existing within-zone,
  non-chronological relocation with no relevant dependency edges and no
  protected target. It is never applied.
- Unknown source types, absent safety metadata, low token counts, repetition,
  non-gold status and Phase 1A structural-candidate counts do not establish
  removability.
- Provider usage is recorded as present/absent but is not used as a safety
  proof. The baseline has no economic evidence or current pricing input.
- Quality risk remains unknown for destructive or relocation candidates because
  no Phase 1C replay or task-quality check is performed.

The planner does not compress, replay, contact providers, mutate prompts or
write traces.
