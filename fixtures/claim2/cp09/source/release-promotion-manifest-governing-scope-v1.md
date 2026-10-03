SYNTHETIC RELEASE OFFICE
DEPLOYMENT COHORT MANIFEST — REL-SYN-2026-10-03-04
Revision: 1.0
State: IMMUTABLE / APPROVAL REQUIRED

This complete manifest defines the only deployment units covered by this
promotion request. A valid copy contains the release identity, source/build/
image identities, environment targets, ordered rollout cohort membership,
required gate IDs and accountable owners, and the exact rollback artifact
identity for each production cohort. Missing values make the record
incomplete. The attached structured deployment record is authoritative for
field values; this page records its governing scope and review rules.

RELEASE IDENTITY

Release: REL-SYN-2026-10-03-04
Service: ledger-edge-api
Release train: 2026.10
Release revision: 4
Source revision: git:8a64e97d32f5c0146bc9d1087254ce219a76d331
Source tree digest: sha256:ac7c6ec3d3143b9af3273f014cb9308fe7c82c69a4b6f82d7c0ecde3b03a9c17
Build attestation: att-SYN-BLD-2026-1003-044
Image: registry.synthetic.invalid/ledger-edge-api:2026.10.3-4
Image digest: sha256:8f7b136c48d7721be30ca7d3f26fcb82578af930a15d4e296e7c113a2f1a4b80

ENVIRONMENT TARGETS AND COHORT ORDER

1. `staging-eu1`: synthetic staging validation target; all four service
   shards must report the same image digest before promotion eligibility.
2. `production-canary-eu1`: the first canary deployment unit receives 10
   percent of eligible production traffic; observe for 30 minutes before a
   board decision.
3. `production-wave-eu1`: the remaining 90 percent of that listed cohort;
   this is the requested production target and may start only after all gates
   pass and the rollback artifact is verified.

The bounded EU1 cohort consists of four immutable deployment units:
`ledger-edge-eu1-a`, `ledger-edge-eu1-b`, `ledger-edge-eu1-c`, and
`ledger-edge-eu1-d`. No other region, service, or shard is part of this
manifest. The canary consists of unit `ledger-edge-eu1-a`; the wave consists
of units `ledger-edge-eu1-b`, `ledger-edge-eu1-c`, and
`ledger-edge-eu1-d`.

REQUIRED PROMOTION GATES

| Gate | Required evidence | Accountable owner |
| --- | --- | --- |
| G-BUILD-01 | Source revision and builder attestation match the image digest. | Build Engineering |
| G-TEST-02 | Release validation suite passes for the pinned source revision. | Release Quality |
| G-SUPPLY-03 | Signed synthetic component inventory has no blocking finding. | Product Security |
| G-CANARY-04 | Canary availability and error rate remain inside the frozen bounds. | Runtime Operations |
| G-CAPACITY-05 | Canary latency remains below the release ceiling. | Runtime Operations |
| G-ROLLBACK-06 | Exact previous stable image and rollback bundle are retrievable. | Release Engineering |

ROLLBACK AUTHORITY

Every production target uses rollback artifact
`RB-SYN-REL-2026-10-03-04-PREV3`, pinned to the previous stable release
`REL-SYN-2026-10-03-03` and image digest
`sha256:2c88e74118de02583d908f90e9b94a46d31a29fa59e5f08c76d113f0b741ce62`.
The rollback bundle hash and retrieval state are present in the frozen
release-state snapshot. Do not substitute a similarly named artifact.

BOARD DISPOSITION

Promote only when source/build/image identity matches, all six required gates
pass, both canary measurements meet their stated limits, the requested cohort
matches the frozen membership, and the exact rollback artifact is verified.
If evidence is incomplete or a bounded gate is unknown, use `HOLD`. If an
identity mismatch or blocking security finding is established, use
`ROLLBACK`. The board records every required gate ID and the exact rollback
artifact ID in its final structured decision.
