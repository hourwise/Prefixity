# Phase 1C Claim-2 V3 workload materialization

Status: `CLAIM_2_WORKLOAD_V3_MATERIALIZATION_READY_FOR_TOKENIZATION_PREPARATION`.
Starting accepted `main`: `28948116f7a76f2e4a8c529562995dbd3efc70e9`.
Source branch: `codex/phase1c-claim2-v3-materialization`.

This is offline workload preparation only. The new CP09/CP10 request bodies
have **unknown token counts** and no context or saving admission result. No
llama process, health endpoint, input-token endpoint, external tokenizer,
completion inference, capability probe, or scored pilot was used. The sealed
V1 `WORKLOAD_TOKEN_ADMISSION_FAILED` and V2
`WORKLOAD_V2_TOKEN_ADMISSION_FAILED` cohorts remain historical and unchanged.
CP01/04 and CP07/08 remain failed positives outside this primary cohort.

## Contract, roster, and inherited evidence

The versioned [V3 authoring contract](../../fixtures/claim2/workload-contract-v3.md)
has contract ID `prefixity.phase1c.claim2-workload-authoring-contract`,
version 3, SHA-256
`28d48448719433a9baa28c0b668cafade63c7858655be00a1fd9effbb638ac5e`.
It fixes the case-major order CP02, CP03, CP09, CP10, CP05, CP06: four
positives, then two zero-mutation controls. Every case has BASELINE, NO_OP,
and INTERVENTION arms at three sequential request slots. All 18 arm histories
were rendered offline, yielding 54 logical requests. This is a future
inference ceiling, not authorization for those calls. The controlled-only
`controlled-evidence-policy-v1`, canonical raw advancing-output protocol,
one-target `EXACT_DUPLICATE_PRUNE`, control rules, and token/context gates
remain unchanged.

CP02/03/05/06 fixtures are byte-identical to the accepted repository baseline.
For their 36 logical requests, regenerated V1, pinned V2, and V3 request
bodies, messages, hashes, fixture identities, canonical prior outputs, and
receipts agree exactly. The separate reproducible
[V3 inheritance map](../../fixtures/claim2/token-evidence-inheritance-map-v3.json)
has SHA-256
`6a53ea43c74d20d8c5bf99483ffdf79e6706656076014fd4eb7728198bcb7aa2`.
It verifies the V1 identity canonical seal
`4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`,
raw evidence SHA-256
`caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264`,
interpreted-result seal
`03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040`,
and exact hash/count joins. The retained requests cover 14 measured unique
hashes. Their counts remain inherited **V1 measurements**, conditional on
the same accepted GGUF/tokenizer, llama build, reasoning-off/template and
endpoint semantics in any later plan. Changed bytes fail closed as
`V1_TOKEN_EVIDENCE_NOT_REUSABLE`.

## New case CP09: software-release promotion

The [CP09 workflow](../../fixtures/claim2/cp09/workflow.md) is a wholly
synthetic release-board promotion review. Its complete immutable deployment
cohort manifest specifies the release/build/image identities, bounded
deployment units and environments, required rollout gates, owners, canary
limits, and rollback artifact IDs. The manifest is 5,706 UTF-8 bytes,
SHA-256
`b3f59db8d7a37791ff0dc2d33a4c74a43bf5032ba9fdd9593d58e490bf589c0b`;
the public case manifest SHA-256 is
`e9901ea2f23bed39ffcac31bd7f3074f8836483ed6893536365b39f221d5c708`.
The deterministic generator reproduces its 20 pinned fixture files.

Request 1 opens the frozen release state and original native manifest
Message, then selects read-only `verify_build_provenance`. Request 2 carries
that arm's exact raw output and pinned result, then selects read-only
`verify_required_rollout_gates`. The ordinary change-board packet recipe
requires the complete manifest again alongside the unique provenance and
gate receipts for every promotion review, irrespective of document size or
benchmark arm. The later Message is byte-identical to the original, with the
same source and world revision; the original remains. No receipt is relabelled
or duplicated. The later occurrence has zero consumers and zero protected
relations. The unchanged policy selects only
`e-release-manifest-final-reattachment` by `EXACT_DUPLICATE_PRUNE`, with one
PRUNE, zero DEFER, and zero RELOCATE candidates. BASELINE and NO_OP requests
match; INTERVENTION differs only at request 3.

All three independent offline arm replays pass the finite actions, pinned
action/result receipts, required and critical preservation, and hidden exact
JSON evaluation of promotion decision, release/build/image digests, gates,
cohort, rollback ID, and reason code. The hidden key is outside all model-
visible and planner-visible assets. The per-case
[advancing domain](../../fixtures/claim2/cp09/advancing-output-domain-v3.json)
SHA-256 is
`c55092b1002467b39f44f080930ac9138ef526a8b23d34d050e9f5abb8c7f5bd`.
Its two exact raw output SHA-256 values are
`5a417af5b18e746beee3eb3299627bc4e095bf4dd9642c7c899f1866fcd61174`
and `e7f34235a32f1d7453b8252677012e34174aa21738c41e0715db9a33f2031a56`.

## New case CP10: observation-data publication

The [CP10 workflow](../../fixtures/claim2/cp10/workflow.md) is a wholly
synthetic publication of 12 immutable readings in three bounded shards from
two fictional instruments. Its complete versioned field dictionary and
aggregation specification defines every input field, unit, null and quality
flag, UTC normalization, grouping, integer rounding, output column, and
quality disposition. The specification is 3,153 UTF-8 bytes, SHA-256
`ddbdbecf474690bb928871faaaeb22bce58a02ae04a629639d61f9471ef06de5`;
the public case manifest SHA-256 is
`56197ec30757e6d983225abc35edadc830b062074b0a67f87753b4493a107ca3`.
All source records are synthetic and immutable.

Request 1 presents the original native specification Message and collection,
then selects `inspect_observation_catalog`. Request 2 carries exact arm-local
output and catalog receipt and selects `inspect_quality_and_coverage`. The
normal publication packet recipe carries the distinct catalog/quality
receipts and reattaches the exact effective specification for every
publication, regardless of size or arm. The later Message has the original
bytes, source and world revision; the original remains. The later occurrence
has zero consumers and zero protected relations. The unchanged policy selects
only `e-spec-final-reattachment` by `EXACT_DUPLICATE_PRUNE`, with one PRUNE,
zero DEFER, and zero RELOCATE candidates. BASELINE and NO_OP match;
INTERVENTION differs only at request 3.

The deterministic evaluator derives exact UTC rows, flagged intervals,
source IDs, fixed-integer aggregates, publication status, and reason from the
frozen shards and rules. Intermediate actions, result/receipt links, required
and critical context, hidden-key isolation, and all three offline arms pass.
The per-case [advancing domain](../../fixtures/claim2/cp10/advancing-output-domain-v3.json)
SHA-256 is
`c13ab2bbee93a032df8a815cbd9162fd44b1ca1001b3a4c19c7b03ac0abc7f39`.
Its two exact raw output SHA-256 values are
`1a1a248cb606bfb72b6d8c00020a32d45bf84db8bd107df7524b8d7b7b0206c5`
and `bba28ea63ac927d514bd0b1c6d6e23cd349d6d535a7202165457f7595c4e9d23`.

CP09 evaluates deployment promotion state and rollback authority; CP10
evaluates observation transformation and publication. Neither repeats
CP02's repository symbol search, CP03's build metadata reconciliation,
CP07's benefit adjudication, or CP08's cold-chain quality disposition.
Their complete governing objects and packet rules were defined by workflow,
not by a token target or post-count edit.

## Exact offline request and size evidence

The aggregate [V3 advancing domain](../../fixtures/claim2/advancing-output-domain-v3.json)
has 12 points and SHA-256
`d9f2de3056734026e364f0b2b89af8af8bf3cfbba78ded8e06533d9fc20c4186`.
The exact [V3 request ledger](../../fixtures/claim2/workload-request-ledger-v3.json)
has SHA-256
`406586d71839d91bc565b9db5da69a3d4152193c4109f7876e6a1dc56d89dce2`.
Each of its 54 rows binds ordered messages, exact serialized future token-
counter body, body/message hashes and byte lengths, case/arm/slot, canonical
prior output and receipt identities, policy decision, and inherited/new
classification. There are 22 unique exact-body hashes: 14 inherited V1
hashes covering 36 logical requests and eight new hashes covering the 18
CP09/CP10 logical requests. These are measured *identities and bytes*, not
token counts. No request is live-dispatchable from this ledger.

| Case | Slot 1 bytes | Slot 2 bytes | BASELINE/NO_OP slot 3 bytes | INTERVENTION slot 3 bytes | Duplicate object bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| CP09 | 11,692 | 12,716 | 20,032 | 13,732 | 5,706 |
| CP10 | 8,688 | 10,069 | 15,800 | 12,304 | 3,153 |

These complete serialized request sizes are descriptive. CP09/CP10
baseline slot-3 bytes are below V2 CP07/CP08's 27,875/29,781 bytes by
ordinary workflow construction, with no truncation of their governing
objects. They cannot establish either positive reduction gate or either
context gate. Both new cases remain `NEW_TOKEN_COUNT_REQUIRED`.

The [version-4 materialization successor](../../fixtures/claim2/materialization-report-v4.json)
has SHA-256
`dbf40e192c60b6f27b3e0112eb639be29a91e71b59d369b4f43ad682337c1e03`.
It binds the pinned V2 version-3 predecessor, V3 contract, new-case fixture
identities/domains, aggregate V3 domain, exact ledger, inheritance map,
current source provenance, 18 arm trajectories, 54 rendered requests, and
offline boundary without overwriting V1/V2 artifacts.

## Validation and boundary

Focused CP09 and CP10 fixture tests prove generator determinism, complete
governing assets, ordinary packet provenance, exact same-state native
reattachment, sole frozen-policy target, closed action/receipt dependencies,
hidden-key isolation, canonical raw outputs, and three-arm replay. V3 ledger
and inheritance-map tests reproduce tracked bytes and reject altered V1 or
retained V3 request bodies. Historical V2 successor regeneration initially
detected the shared manifest loader's additive CP09/10 admission change:
its original source hash is historical provenance, not the hash of current
code. The V2 successor generator now emits its frozen accepted V2 renderer
and generator hashes while continuing to reproduce the same historical
successor bytes; its archived artifact, fixture data, counts, and seals were
not changed. The V3 successor records the current source provenance.

Full locked/offline workspace tests, formatting, warnings-denied Clippy,
Rust 1.86 locked/offline check, all artifact regeneration, historical
regression checks, staged-diff review, and exact candidate CI are publication
requirements. The exact validation/CI outcome and promoted SHA are reported
in the supervisor closeout. No tokenization or inference follows this task.
The next authorized task is
`CLAIM_2_WORKLOAD_V3_TOKENIZATION_PREPARATION` only.

`CLAIM_2_WORKLOAD_V3_MATERIALIZATION_READY_FOR_TOKENIZATION_PREPARATION`
