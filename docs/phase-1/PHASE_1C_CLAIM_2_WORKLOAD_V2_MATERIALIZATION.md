# Phase 1C Claim-2 V2 workload materialization

Status: `CLAIM_2_WORKLOAD_V2_MATERIALIZATION_READY_FOR_TOKENIZATION_PREPARATION`

Starting accepted `main`: `baeead2d1ca9710f50dafae0e02b4e5c2ed1515e`

Source branch: `codex/phase1c-claim2-v2-materialization`

This is an offline workload artifact. No server was started, no health or input-token
endpoint was contacted, no tokenizer was used, and no model inference occurred.
The V1 `WORKLOAD_TOKEN_ADMISSION_FAILED` result, including CP01 and CP04's
775-token request-3 savings, remains unchanged. This document records V2
materialization, not V2 token admission or model capability.

## Frozen cohort and contract

The versioned [V2 authoring contract](../../fixtures/claim2/workload-contract-v2.md)
fixes the primary case order as CP02, CP03, CP07, CP08, CP05, CP06. The first
four are positives; CP05 and CP06 are zero-mutation controls. Each case has
BASELINE, NO_OP, and INTERVENTION arms with three request slots. These are 54
future logical inference slots, not contacts in this preparation. The frozen
mechanism is `controlled-evidence-policy-v1`, with exactly one eligible
`EXACT_DUPLICATE_PRUNE` target for each positive and no change to rule order,
eligibility, or projection. The V1 fixture contract remains historical.

The retained CP02, CP03, CP05, and CP06 manifests and all fixture assets are
byte-identical to the accepted V1 baseline. The current adapter regenerates
the complete V1 request ledger byte-for-byte, including each retained
rendered request. The reproducible [inheritance map](../../fixtures/claim2/token-evidence-inheritance-map-v2.json)
verifies V1 ledger, plan, identity, sidecars, raw evidence, interpreted result,
and runtime bindings before admitting a count. It maps 14 exact V1 hashes to
36 retained logical requests: CP02 has four unique hashes over nine requests,
CP03 four over nine, CP05 three over nine, and CP06 three over nine. A changed
retained body fails closed as `V1_TOKEN_EVIDENCE_NOT_REUSABLE`.

The inherited measurements are bound to V1 tokenization identity
`claim2-tokenization-v1-a5a6b896555db8296318f38010b7120dad8ad191e1329f030e0f738f31b90b91`,
canonical identity seal `4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`,
raw evidence SHA-256 `caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264`,
and interpreted-result seal `03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040`.
They remain **V1 measurements reused in V2**, never new V2 contacts. Reuse in
a later count plan still requires the exact V1 GGUF/tokenizer, llama build,
reasoning/template behavior, endpoint, and other relevant runtime settings.
Git checks out four historical identity/result text files with CRLF on Windows
and LF on Unix. The inheritance audit pins the SHA-256 of both physical forms,
verifies the unchanged canonical JSON seals, and emits the same historical
CRLF-form binding in its map. The raw token evidence, request ledger, and
contact plan retain exact byte pins without text conversion. Any other text
content or line-ending form fails the inheritance gate.

## New-case evidence

CP07 is a wholly synthetic benefits adjudication. Its complete versioned
Northstar imaging plan is 10,976 UTF-8 bytes, SHA-256
`d171809f32419b8a978f8264204f9b224e41d2db3d626f5bc28898f06a2a12ee`.
The frozen case snapshot is read-only. Eligibility and benefit-arithmetic
actions produce pinned receipts. An ordinary final-adjudication packet recipe
reattaches the same controlling plan document regardless of document size or
benchmark condition. The original and final native user Message contain
identical bytes, source revision, and world-state revision. The later message
has zero consumers and zero protected relations. The actual frozen policy
selects only `e-plan-final-reattachment` by `EXACT_DUPLICATE_PRUNE`; DEFER and
RELOCATE candidate sets are empty. The hidden exact JSON answer evaluates
coverage, clause IDs, deductible, coinsurance, annual limit, exclusion, and
final disposition from the fixed facts. Its key is outside model-visible
assets. The [workflow](../../fixtures/claim2/cp07/workflow.md) and generated
fixture state the ordinary packet rule and its provenance.

CP08 is a wholly synthetic cold-chain lot disposition. Its complete versioned
`CC-SOP-04` revision 3.2.1 is 11,019 UTF-8 bytes, SHA-256
`eb5605ad1cc8253b6c04ee8ff35e1289f8b9620dc35bee2eeffb1dee9e7b8e8e`.
The lot, logger, and calibration source records are frozen and read-only.
Finite inspection and exposure-calculation actions return source-bound
receipts. Every ordinary final quality-review packet reattaches the effective
SOP independent of size or benchmark condition. Its original and final native
user Message have identical body bytes and revisions; the latter has zero
consumers and zero protected relations. The actual frozen policy selects only
`e-sop-final-reattachment` by `EXACT_DUPLICATE_PRUNE`, with no DEFER or
RELOCATE candidate. Independent integer arithmetic finds a valid +2-tenths-C
calibration correction, one 15-minute high interval, 75 tenths of a C-minute
of cumulative exposure, and disposition `HOLD` under CC-08. The hidden exact
JSON key is isolated from prompts. The [workflow](../../fixtures/claim2/cp08/workflow.md)
records the ordinary packet rule and source identity.

Both new-case generators reproduce every fixture byte. Their tests check
closed event references, source hashes and revisions, action/result and
receipt bindings, required and critical preservation, hidden-key isolation,
sole candidate selection, and three offline arm replays. The accepted raw
advancing output at each of their two transitions is exactly compact
`{"action_id":"<expected-action-id>"}` with no alternative spelling or
trailing byte. CP07's two SHA-256 values are
`3c324f5a7413b3b0041cb829bd77aec221e6ad30ecd44f2cb815bbb187585e7c`
and `e566f428d4ca4a776604b58023080ef932ccb98583a5d13627b9ed1aa2e5a753`;
CP08's are `e70eef1974f48fb9083560996379f643ff35eaa3b4d551805c772db0bc80d839`
and `87c9328255c33b2c504a89f93c1b5416e34a136ac833e98a14d4ba552047ae68`.
Each point also pins its deterministic receipt. The aggregate
[V2 finite domain](../../fixtures/claim2/advancing-output-domain-v2.json)
has 12 points and 36 independent arm transitions.

## Exact request ledger and size evidence

The generated [V2 request ledger](../../fixtures/claim2/workload-request-ledger-v2.json)
contains exactly 54 case-major records. Each records its case, arm, slot,
ordered rendered messages, exact serialized body and SHA-256, message hash,
UTF-8 byte length, canonical prior output and receipt hashes, selected policy
decision, and per-request evidence classification. BASELINE equals NO_OP at
every slot; INTERVENTION equals them before treatment and differs only at
positive slot 3. Controls remain exact across all arms and slots. The 54 bodies
form 22 unique hashes: 14 inherited V1 hashes covering 36 logical requests,
and eight new hashes covering 18 logical requests. No new token counts are
recorded. The [version-3 successor](../../fixtures/claim2/materialization-report-v3.json)
binds the historical version-2 report, exact ledger, new case fixtures,
finite-domain evidence, source hashes, and offline boundary.

| New case | Slot 1 bytes | Slot 2 bytes | Baseline/NO_OP slot 3 bytes | Intervention slot 3 bytes | Duplicate body bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| CP07 | 14,210 | 15,180 | 27,875 | 16,680 | 10,976 |
| CP08 | 15,356 | 16,965 | 29,781 | 18,550 | 11,019 |

These are exact UTF-8 lengths, not token counts. The duplicated-body sizes
are materially larger in bytes than the failed V1 request-3 serialized-body
differences: 3,063 for CP01 and 3,491 for CP04. This is a design-margin
observation only. It does not imply the 800-token absolute saving, either
ratio gate, the 6,000-input-token ceiling, or 8,192-context fit. Those remain
unknown until a separately authorized non-inference count. No fixture was
enlarged after observing a token result.

## Reproduction and validation boundary

From the repository root, the offline artifacts regenerate with:

```text
cargo run -p prefixity-controlled-benchmark --example claim2_cp07_fixture --offline --locked -- --write fixtures/claim2/cp07
cargo run -p prefixity-controlled-benchmark --example claim2_cp08_fixture --offline --locked -- --write fixtures/claim2/cp08
cargo run -p prefixity-controlled-benchmark --example claim2_v2_token_evidence_inheritance --offline --locked -- --write fixtures/claim2/token-evidence-inheritance-map-v2.json
cargo run -p prefixity-controlled-benchmark --example claim2_v2_workload --offline --locked -- domain
cargo run -p prefixity-controlled-benchmark --example claim2_v2_workload --offline --locked -- ledger
cargo run -p prefixity-controlled-benchmark --example claim2_v2_workload --offline --locked -- successor
```

The new-case fixture regressions verify all generated bytes without rewriting
tracked files. The V2 workload generator writes only the named versioned artifacts.
Validation covers formatting, warnings-denied Clippy, the full locked/offline
workspace suite, Rust 1.86 locked/offline workspace check, byte-for-byte
artifact regeneration, V1 historical integrity checks, and staged diff
whitespace review. No tokenization, scored-pilot preparation, or inference is
authorized by this artifact. The exact next task is
`CLAIM_2_WORKLOAD_V2_TOKENIZATION_PREPARATION`.
