# Phase 1C Claim-2 workload V3 design review

Status: `CLAIM_2_WORKLOAD_V3_DESIGN_READY` — documentation-only design.
Starting accepted `main`: `ac58674fa326d584c54f108fea7b0a533433099e`.
Next authorized task: `CLAIM_2_WORKLOAD_V3_MATERIALIZATION_PREPARATION` only.

This review uses the sealed [V1 result](PHASE_1C_CLAIM_2_TOKENIZATION_EXECUTION_RESULT.md),
[V2 result](PHASE_1C_CLAIM_2_WORKLOAD_V2_TOKENIZATION_EXECUTION_RESULT.md),
[V2 materialization](PHASE_1C_CLAIM_2_WORKLOAD_V2_MATERIALIZATION.md),
and both interpreted token-count records. It selects a proposed new cohort,
not materialized fixtures or admitted token counts. No server, tokenizer,
inference endpoint, or scored pilot was used.

## Historical evidence and failure interpretation

Both prior cohorts remain immutable. V1 is
`WORKLOAD_TOKEN_ADMISSION_FAILED`: CP01 and CP04 each saved 775 input tokens
at request 3, 25 short of `D3 >= 800`. Their `R3` values were 36.013% and
36.232%, and their `Rsum` values 18.171% and 18.002%; both ratio gates and
both context checks passed. Their baseline slot-3 inputs were 2,152 and
2,139, and intervention inputs 1,377 and 1,364. They are permanently
classified `V1_POSITIVE_FAILED_CONTEXT_PRESSURE_ADMISSION` and remain outside
the V3 cohort. No V1 bytes, measurements, identity, seal, or threshold changes.

V2 is `WORKLOAD_V2_TOKEN_ADMISSION_FAILED`. Its one hybrid pass returned
eight successful fresh unique-body counts, reused 14 sealed V1 unique counts,
mapped all 54 logical requests, and recorded zero inference. CP07 baseline
slot 3 was 6,811 and intervention 4,290 (`D3 = 2,521`, `R3 = 37.014%`,
`Rsum = 17.994%`). CP08 baseline slot 3 was 7,451 and intervention 4,999
(`D3 = 2,452`, `R3 = 32.908%`, `Rsum = 15.461%`). Both passed all reduction
gates but breached the frozen 6,000-input-token ceiling. CP08 also reached
8,475 with the reserved 1,024 output tokens, 283 above the 8,192 context.
They are permanently classified `V2_POSITIVE_FAILED_CONTEXT_ADMISSION` and
remain outside V3. Their complete plan/SOP, fixtures, counts, raw evidence,
interpreted result, identity, and seals remain unchanged.

The combined evidence **supports, within this bounded cohort**, the proposed
interpretation: CP01/04 had insufficient absolute whole-request removable
token mass, whereas CP07/08 had ample removable mass but excessive complete
baseline context. This is not an attachment-only token estimate or a general
relationship between document bytes and tokenizer counts. No accounting or
mechanism defect is visible. V1 recorded one readiness and 22 successful
unique-body token contacts; V2 recorded one readiness and eight successful
new-body contacts with sealed inheritance. Both completed their exact
hash-to-request mappings without inference. The unchanged research policy
selected exactly one legal `EXACT_DUPLICATE_PRUNE` target in each failed
positive: CP01 `e-repeat`, CP04 `e-recovery-copy`, CP07
`e-plan-final-reattachment`, and CP08 `e-sop-final-reattachment`. The original
same-state native Message was retained; the later byte-identical occurrence
had no consumer or protected relation. Correct selection does not establish
future model task success; neither cohort ran a scored pilot.

CP02 and CP03 demonstrate that the full token-gate combination is feasible
under the accepted instrument: CP02 measured baseline/intervention slot 3
of 4,078/3,093, `D3 = 985`, `R3 = 24.154%`, `Rsum = 17.104%`; CP03 measured
2,488/1,590, `D3 = 898`, `R3 = 36.093%`, `Rsum = 22.217%`. Both passed
context and reduction gates in V1 and V2. Retain their exact fixtures as
positive candidates. Retain CP05 (same bytes, different states) and CP06
(same-state duplicate with a protected consumer) as separate zero-mutation
controls; all three arms had identical bodies/counts at every slot, context
passed, and legal reduction was zero. Neither control is a positive saving.

Historical results bracket a **broad design question**, not a token target:
CP01/04 had small complete baseline requests but missed absolute saving;
CP07/08 had large complete requests and ample saving but failed context.
CP02/03 occupy feasible measured examples with different task semantics. A
new workflow should naturally carry a substantive single repeat while
keeping the complete three-request history bounded. No attachment byte count,
new-case token count, 800-adjacent saving, or 6,000-adjacent baseline is
prescribed. Future counts remain unknown until a separately authorized,
pre-frozen tokenization pass.

## V3 cohort decision

The smallest defensible next cohort retains two passing positives, two
distinct new positives, and both controls in case-major order:

| Order | Case | Role | Decision |
| ---: | --- | --- | --- |
| 1 | CP02 | positive | Retain exact V1/V2 repository path/symbol investigation. |
| 2 | CP03 | positive | Retain exact V1/V2 dependency/build metadata reconciliation. |
| 3 | CP09 | new positive | Synthetic software-release promotion from an immutable deployment cohort manifest. |
| 4 | CP10 | new positive | Synthetic observation-data publication from a versioned field dictionary and aggregation specification. |
| 5 | CP05 | control | Retain exact different-state, same-bytes no-op. |
| 6 | CP06 | control | Retain exact consumer-protected, same-state no-op. |

Three sequential model requests per arm and independently generated BASELINE,
NO_OP, and INTERVENTION trajectories retain the 54-request ceiling and 18
fresh-arm structure for any future scored run. That ceiling authorizes no
inference. CP09 and CP10 are genuinely new case IDs and workflows, not
revisions, repairs, or reclassifications of CP07/08. The proposed roster is a
design decision; materialization must reject a case whose ordinary workflow
cannot meet its stated provenance and dependency contract. A weak substitute
must not be invented merely to fill six rows.

### CP09 — software-release promotion

Use a synthetic release train and an immutable deployment cohort manifest
generated by a bounded release-packet process. The complete manifest is the
governing object for this release: image/build digests, environment targets,
rollout gates, owners, and exact rollback artifact IDs. Its completeness is
defined by release fields and bounded cohort membership before any byte or
token inspection. No filler entries or repeated prose are allowed.

Request 1 presents the promotion task and the original manifest as a native
non-protocol context Message; the model selects a finite inspection action.
Request 2 carries its own raw prior output and pinned receipt, then selects a
read-only provenance or rollout-gate check on the same frozen release state.
The normal change-board approval packet for request 3 includes that same
manifest again beside the distinct check receipts and final decision prompt.
The packet recipe must be documented as an ordinary release workflow and must
run regardless of size or benchmark arm. The later manifest Message is
byte-identical and explicitly same-state, has no consumer or protected edge,
and is the sole eligible `EXACT_DUPLICATE_PRUNE` target. Preserve the original
manifest, all unique check results, action/result links, and rollback facts.

The hidden deterministic evaluator checks an exact `PROMOTE`/`HOLD`/`ROLLBACK`
decision, release and artifact digests, gate IDs, and required rollback ID
against frozen records. It also checks both intermediate actions and receipts.
This is deployment-state authorization, not CP02 repository discovery or
CP03 package-version/dependency reconciliation. Risk: an approval packet
might cite the later copy as a legal consumer, or duplicate a receipt as
well. Either condition rejects the candidate; do not remove an edge,
relabel a tool result, or select a different duplicate.

### CP10 — observation-data publication

Use a synthetic, fixed observation collection with a complete versioned
field dictionary and aggregation specification. Its normal publication
workflow defines the bounded source shards, units, quality flags, time-zone
and rounding rules, and exact output fields before materialization. The
dictionary/specification is the complete governing object for that dataset,
not a truncated extract or padded document.

Request 1 presents the publication task and original specification as a
native non-protocol context Message, then asks for a finite source-catalog
inspection. Request 2 carries that arm's raw output and deterministic receipt
and selects a read-only gap/quality check of the unchanged collection. The
ordinary publication packet at request 3 reattaches the same specification
beside distinct catalog and quality receipts and asks for the final structured
rollup. The later copy must have the exact earlier bytes and same world-state
revision, no consumer or protected relation, and be the only eligible prune
target. The original, raw shard identities, quality results, and receipts
remain visible and load-bearing.

The hidden evaluator checks exact output rows, flagged intervals, integer or
fixed-decimal aggregates, source IDs, and publication/hold status. Its key
never enters a prompt or planner input. This is data transformation and
publication, distinct from CP02 symbol search, CP03 build metadata, CP09
release promotion, CP07 coverage adjudication, and CP08 cold-chain quality
disposition. Risk: real publication tooling may attach a transformed schema
instead of the unchanged specification, or give the later copy a consumer.
If so, reject the case rather than asserting same-state identity or changing
the mechanism.

The family search also considered database migration cutover, API sunset,
procurement award, incident remediation, manufacturing deviation,
archival-package review, and protocol certification. These are plausible
future studies, but some overlap prior compatibility/adjudication/quality
semantics or demand more complicated dependency closure. They are not V3
cases. CP09 and CP10 offer two different ordinary packet workflows with
bounded complete artifacts and exact, finite evaluators; their context
pressure is a hypothesis, not an admitted measurement.

## Frozen boundary and later evidence

Continue using `controlled-evidence-policy-v1`, scope `CONTROLLED_ONLY`,
unchanged. The frozen rule order starts with `EXACT_DUPLICATE_PRUNE`, and its
existing sequence rule selects the latest eligible exact, same-state native
Message. A positive must have one and only one eligible target at the
pre-request-3 checkpoint. Do not size-rank, aggregate, alter rule priority or
cardinality, prune historical tool outputs, change protection semantics, or
manufacture a duplicate. Offline materialization must establish source and
world revisions, event IDs, exact bytes, closed dependency graph, ordinary
packet provenance, and actual policy-selection/apply parity. If either new
workflow requires different semantics, return `MECHANISM_CHANGE_REQUIRED`.

Keep every frozen threshold: each positive requires `D3 >= 800`,
`R3 >= 0.20`, and `Rsum >= 0.08`; every complete rendered request requires
`input_tokens <= 6000` and `input_tokens + 1024 <= 8192`. CP05/06 require
legal reduction zero and exact unchanged arm requests. Broad design margin
is preferred, but it creates no new numerical gate. Freeze CP09/10 fixtures,
rendered bodies, dependencies, evaluator, and complete-artifact rules before
their token counts are inspected. A count failure must be reported, not
repaired with fixture edits or a threshold change.

V3 may inherit CP02/03/05/06's 14 sealed V1 unique counts over 36 logical
requests only if future V3 materialization proves each exact UTF-8 request
body and runtime/instrument identity unchanged. Verify the V1 identity seal,
raw SHA-256, interpreted-result seal, and the exact hash/count join; retain
their V1 measurement label. V2 already demonstrated that conditional
inheritance, but V3 has no request ledger yet, so reuse is **not yet
admitted**. Changed bodies or instrument semantics require new counting under
a separate identity and plan. CP09/10 have no token counts, and neither the
V1/V2 body byte sizes nor any proposed V3 artifact bytes can supply them.

The V2 complete cohort measured 149,246 logical input tokens. Historical
9–10 tokens/s prefill and 1.2–2 tokens/s decode rates imply a substantial
CPU burden; 18 fresh server starts remain unmeasured. This is planning
context, not V3 runtime or token admission. Retain the separate
`CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED` classification and the historical
`MODEL_CAPABILITY_ACCEPTED` instrument decision. No scored-pilot preparation
follows from this design review.

The next authorized work is offline
`CLAIM_2_WORKLOAD_V3_MATERIALIZATION_PREPARATION`: define the versioned V3
contract and CP09/10 fixture provenance, finite actions, receipts, hidden
evaluators, deterministic generators, and full rendered ledger while leaving
V1/V2 historical assets untouched. This review does not begin that work.

`CLAIM_2_WORKLOAD_V3_DESIGN_READY`
