# Claim-2 workload authoring contract, version 2

Contract ID: `prefixity.phase1c.claim2-workload-authoring-contract`

Contract version: `2`

Accepted design baseline: `baeead2d1ca9710f50dafae0e02b4e5c2ed1515e`

This contract governs the new V2 workload cohort. The original
[`README.md`](README.md) remains the historical V1 fixture contract and is not
reinterpreted by this document. V2 reuses the existing case-manifest format
where its fields fit; this cohort contract does not change the V1 manifest
schema or any V1 evidence.

## Frozen cohort

The case-major order and role are fixed:

| Order | Case | Role | V2 provenance |
| ---: | --- | --- | --- |
| 1 | CP02 | positive | retained V1 case |
| 2 | CP03 | positive | retained V1 case |
| 3 | CP07 | positive | new V2 case |
| 4 | CP08 | positive | new V2 case |
| 5 | CP05 | control | retained V1 case |
| 6 | CP06 | control | retained V1 case |

Each case has three sequential request slots in each of `BASELINE`, `NO_OP`,
and `INTERVENTION`, for a ceiling of 54 future logical inference slots. This
ceiling does not authorize inference. CP01 and CP04 remain failed V1 positives
and are outside the V2 primary cohort.

## Frozen mechanism and trajectory

Use `controlled-evidence-policy-v1` without changing its rules, rule order,
candidate cardinality, dependency eligibility, or sequence behavior. The sole
scored omission for each positive remains the latest eligible duplicate
selected by `EXACT_DUPLICATE_PRUNE`. Do not size-rank, aggregate, relabel,
delete, or manufacture candidate events. Each control must preserve exact
request identity across arms and produce zero legal reduction.

Requests 1 and 2 use the accepted canonical advancing-output protocol. The
only accepted raw UTF-8 response for a transition is the compact
fixture-derived object `{"action_id":"<expected-action-id>"}` with exactly
one field, exact expected field/value ordering, and no leading or trailing
bytes. Keep each arm's raw response and deterministic receipt unchanged. Do
not normalize a response or substitute one arm's history for another.

## Retained-case and V1 evidence rules

CP02, CP03, CP05, and CP06 remain byte-for-byte identical to their accepted V1
fixtures and rendered request bodies. Check their manifests, assets, prompts,
menus, receipts, trace, state identities, dependency relations, hidden keys,
rendering, message ordering, and wire serialization. A changed retained
request body is `V1_TOKEN_EVIDENCE_NOT_REUSABLE`; semantic equivalence does not
permit inheritance.

The V2 inheritance map binds counts only by exact request-body SHA-256 and
exact bytes, and records the corresponding V1 logical request identities. An
inherited count remains a V1 measurement; it is not a new V2 contact. Reuse
also requires the same model/tokenizer, GGUF, llama build, reasoning-off
setting, template behavior (including absent `chat_template_kwargs`),
endpoint, and other relevant runtime conditions. If any seal or runtime
binding fails verification, do not reuse counts; a complete fresh V2
tokenization pass is required before any later admission review.

The bound V1 evidence is:

| Binding | Value |
| --- | --- |
| Tokenization identity | `claim2-tokenization-v1-a5a6b896555db8296318f38010b7120dad8ad191e1329f030e0f738f31b90b91` |
| Canonical identity seal | `4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16` |
| Raw evidence SHA-256 | `caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264` |
| Interpreted-result seal | `03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040` |

The accepted design expects 14 retained unique request hashes covering 36
logical requests. The reproducible inheritance map must derive and verify
these counts from sealed V1 inputs; it must fail closed on any discrepancy or
retained-body drift.

## New cases and admission boundary

CP07 is a synthetic benefits / coverage adjudication case. CP08 is a synthetic
cold-chain lot-disposition case. Their ordinary workflows must naturally
reattach the same complete, versioned policy/SOP as an unchanged, same-state
native `Message`. The earlier original remains; the later duplicate has no
consumer or protected relation and is the sole eligible PRUNE target. Stop a
case for supervisor review if the ordinary workflow produces multiple
eligible targets; return `MECHANISM_CHANGE_REQUIRED` if the frozen mechanism
cannot select its natural target.

Use deterministic read-only source checks, finite action menus, action-bound
receipts, closed-world dependency graphs, and exact structured hidden-key
evaluators. Keep evaluation keys outside all model-visible inputs. Do not use
real personal, patient, or production records. Do not add padding, filler,
repeated text for size, or workflow steps whose purpose is duplicate creation.

The frozen positive gates remain request-3 absolute saving `D3 >= 800`,
request-3 ratio `R3 >= 0.20`, and cumulative ratio `Rsum >= 0.08`. Every
rendered request must later meet the existing 6,000 input-token ceiling and
8,192 context with a 1,024-token output reservation. UTF-8 byte sizes are
descriptive only; they do not establish token counts or admission.

Fixtures and request bodies are materialized and frozen before any later
tokenizer contact. Post-tokenization fixture tuning, threshold changes,
adaptive retries, and rescoring with revised material are prohibited. This
contract authorizes offline preparation only: no server contact, tokenizer
use, token-count inference, or model inference.
