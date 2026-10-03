# Claim-2 workload authoring contract, version 3

Contract ID: `prefixity.phase1c.claim2-workload-authoring-contract`

Contract version: `3`

Accepted design baseline: `28948116f7a76f2e4a8c529562995dbd3efc70e9`

This contract governs only the new V3 cohort. The V1 [`README.md`](README.md)
and V2 [`workload-contract-v2.md`](workload-contract-v2.md) remain historical.
Their fixtures, ledgers, measurements, evidence, identities, and failures are
not changed or reclassified. The existing case-manifest schema and frozen
renderer/policy are reused where their fields fit.

## Fixed cohort and trajectory

| Order | Case | Role | Provenance |
| ---: | --- | --- | --- |
| 1 | CP02 | positive | byte-identical retained V1/V2 case |
| 2 | CP03 | positive | byte-identical retained V1/V2 case |
| 3 | CP09 | positive | new synthetic software-release promotion |
| 4 | CP10 | positive | new synthetic observation-data publication |
| 5 | CP05 | control | byte-identical retained V1/V2 case |
| 6 | CP06 | control | byte-identical retained V1/V2 case |

Each case has BASELINE, NO_OP, and INTERVENTION arms and exactly three
sequential request slots. Each arm carries its own raw prior assistant output
and deterministic environment receipts. The complete cohort has 18 offline
arm trajectories and 54 rendered logical requests. These are prepared
requests, never live dispatch or inference authorization. No outcome is
copied between arms.

Requests 1 and 2 use the accepted canonical advancing-output protocol: raw
UTF-8 is exactly compact `{"action_id":"<expected-action-id>"}` with no BOM,
extra field, whitespace, newline, or alternate escaping. The finite action
domain has one accepted raw spelling at each transition. The corresponding
deterministic receipt and next state are pinned. Invalid model output in a
future separately authorized run is a failure, not normalized or retried.

## Frozen intervention and dependency contract

The research-only `controlled-evidence-policy-v1`, scope `CONTROLLED_ONLY`,
keeps its existing rule order, eligibility, target cardinality, and sequence
selection. The sole scored omission in each positive is the one latest
eligible native Message selected by `EXACT_DUPLICATE_PRUNE` immediately before
request 3. No size ranking, aggregation, new rule, relabelled tool output,
changed protection, or synthetic benchmark-only repeat is allowed.

CP09's complete immutable deployment-cohort manifest and CP10's complete
versioned field dictionary/aggregation specification are governing objects
defined by their ordinary workflows before any token inspection. For each,
the original native non-protocol context Message is retained, and the
ordinary final packet reattaches the same exact bytes from the same source
and world-state revision. The later copy must have zero consumers and zero
protected relations and must be the sole eligible PRUNE target. DEFER and
RELOCATE candidate sets must be empty. All genuine actions, tool/result
events, receipts, unique evidence, and their dependencies remain intact.
If the unchanged policy cannot select that target naturally, stop with
`MECHANISM_CHANGE_REQUIRED`; if several natural targets arise, return for
design review rather than deleting events or changing the rule.

CP05 and CP06 remain zero-mutation controls. CP05's equal bytes at different
states do not create a same-state duplicate. CP06's same-state later copy has
a protected consumer. They must select `DO_NOTHING`, preserve every request
body across arms, and have legal reduction zero.

All new sources are synthetic and deterministic. No real credentials,
personal/location-sensitive observations, or production systems may enter
the fixtures. Every event, revision, action/result link, receipt, dependency,
required/critical item, source hash, and provenance locator must resolve in
a closed acyclic graph. The hidden exact evaluator key is isolated from
planner-visible and model-visible inputs. Governing objects must be complete
for their workflows, with no filler or token-targeted truncation.

## Historical evidence and admission boundary

CP01/CP04 remain immutable failed V1 positives and CP07/CP08 immutable
failed V2 positives, outside the V3 primary cohort. CP02/03/05/06 fixtures,
manifests, actions, receipts, canonical outputs, graphs, hidden keys,
rendering, serialization, and policy outcomes must remain byte-for-byte
unchanged. Their sealed V1 token measurements may be inherited only after
V3 regeneration proves exact UTF-8 request-body identity and verifies V1
identity, raw-evidence and interpreted-result seals, exact hash/count joins,
and the same GGUF/tokenizer, llama build, reasoning-off and template settings,
endpoint, and other relevant runtime conditions. Reused counts retain the
label `INHERITED_V1_TOKEN_COUNT`. Any drift is
`V1_TOKEN_EVIDENCE_NOT_REUSABLE`; semantic equivalence is insufficient.
CP09/10 have no accepted token measurements.

The unchanged future positive gates are `D3 >= 800`, `R3 >= 0.20`, and
`Rsum >= 0.08`. Every complete rendered request must later meet
`input_tokens <= 6000` and `input_tokens + 1024 <= 8192`. Controls require
zero legal reduction. Exact request-body hashes and UTF-8 byte lengths are
offline identity and descriptive size evidence only; they cannot establish
token counts, savings, or context admission. Freeze fixtures and rendered
bodies before any separately authorized tokenization; no post-count tuning,
threshold change, adaptive retry, or reclassification of failed cases.

This materialization is offline only. It permits no llama startup, health
check, token-count endpoint, external tokenizer, completion inference,
scored-pilot preparation, or model-capability probe. The next task after an
accepted materialization is solely
`CLAIM_2_WORKLOAD_V3_TOKENIZATION_PREPARATION` under separate authorization.
