# Phase 1C reasoning-budget calibration — Attempt 008 preparation

Status: `ATTEMPT_008_PREPARATION_ACCEPTED`

Execution status: `ATTEMPT_008_NOT_EXECUTED`

This record authorizes preparation and offline certification only. It does not
authorize Qwen/llama startup, port-8080 contact, readiness, HTTP, inference, or
the Attempt-008 live boundary.

## Starting state and eligibility

The accepted starting baseline was reverified before preparation:

```text
HEAD       417673c06b141e07d54ad0bc101e1105e1a734de
main       417673c06b141e07d54ad0bc101e1105e1a734de
origin/main 417673c06b141e07d54ad0bc101e1105e1a734de
worktree   clean
```

The frozen calibration design states that a valid failing candidate advances to
the next registered candidate, but Attempt 007 was not a valid calibration
candidate: its executable identity was rejected after execution. The accepted
Attempt-007 remediation record therefore preserves its raw `FAIL` and
`next_budget=512` without admitting either to calibration state. A consumed,
integrity-invalid attempt does not advance the accepted candidate state. The
last admissible candidate before Attempt 007 was budget `1024`.

The durable eligibility result is:

```text
ATTEMPT_008_ELIGIBLE
CANDIDATE_BUDGET=1024
```

The deterministic budget provenance fixture is
`fixtures/phase1c/attempt-008-budget-provenance.json`. It explicitly records
that Attempt 007's raw `next_budget=512` did not influence the selection.

Historical preservation was independently checked:

```text
ATTEMPT_007_EXECUTED_ONCE
ATTEMPT_007_CALIBRATION_INADMISSIBLE
ATTEMPT_007_PERMANENTLY_CONSUMED
```

The Attempt-006 supervisor evidence remains unchanged at SHA-256
`c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07`.
The Attempt-007 canonical identity remains unchanged at SHA-256
`fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8`.
The preserved Attempt-007 candidate result remains `FAIL` with raw
`selection.next_budget=512`; the offline forensic validator returns
`INTEGRITY_REJECTED / FROZEN_EXECUTABLE_IDENTITY_MISMATCH` without interpreting
model output.

## Frozen experimental contract

Only the attempt identity and execution artifact lineage differ from the
consumed Attempt 007. The registered experimental contract is:

| Field | Attempt-008 value |
| --- | --- |
| Attempt | `008` |
| Candidate budget | `1024` |
| Candidate identity | `phase1c-reasoning-budget-1024` |
| Model reference | `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0` |
| Quantization | `Q4_0` |
| Fixed cases | `rbcal-001`, `rbcal-002`, `rbcal-003` |
| Case order | `rbcal-001 -> rbcal-002 -> rbcal-003` |
| Request ceiling | `3` |
| Reasoning mode | `on` |
| Reasoning budget | Server-side `--reasoning-budget 1024` |
| Context size | `8192` |
| Parallel slots | `1` |
| Metrics | enabled |
| Temperature | `0` |
| Top-p | `1` |
| Seed | `1` |
| Output-token ceiling | `2048` |
| Stream | `false` |
| Connect timeout | `1000 ms` |
| Request timeout | `1200000 ms` |
| Supervisor timeout | `1320000 ms` |
| Automatic retries | `0` |
| Fallback requests | `0` |
| Adaptive replicates | `0` |
| Fresh runtime | fresh llama.cpp server per candidate |
| Warmup | none |
| Endpoint | `http://127.0.0.1:8080/v1/chat/completions` |
| Metrics requirement | enabled |
| Workflow exclusivity | required |
| Workflow certification dependency | accepted 900-v2 plus Attempt-008 exact-chain certification |
| Executable binding | v2 frozen supervisor + child object identity |

The manifest SHA-256 is
`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b`.
The three request hashes are unchanged from the accepted manifest:

```text
rbcal-001  f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461
rbcal-002  e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3
rbcal-003  2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e
```

## Build and frozen execution objects

Build-dependent work completed before freezing:

```text
source/build commit: ddc05be5951e3cc3da36a659f15303f9760d952a
Cargo.lock SHA-256: 79f4fb5ea5e2b698c9785ac609294ed44e793dd8674f49cd53001792a2491cf8
toolchain: cargo/rustc 1.97.1, x86_64-pc-windows-msvc
build: cargo build -p prefixity-controlled-benchmark --bins --locked --offline
```

The full formatting, strict Clippy, and workspace test suite passed before the
freeze. The accepted non-overwriting `freeze_copy` helper then copied the
final build outputs into the bounded, ignored directory
`target/phase1c-attempt-008-frozen/`.

Supervisor:

```text
source:  D:\Users\fleur\Prefixity\target\debug\prefixity-phase1c-live-supervisor.exe
frozen:  D:\Users\fleur\Prefixity\target\phase1c-attempt-008-frozen\prefixity-phase1c-live-supervisor.exe
size:    1259008
sha256:  f07a9302786f3abe5a01eb9fa9eda85f50de9fe9f3d34feba14ba0b2e09c2088
file_id: volume=ba2f80f4;index=0007000000387abb
```

Child:

```text
source:  D:\Users\fleur\Prefixity\target\debug\prefixity-phase1c-reasoning-budget-calibration.exe
frozen:  D:\Users\fleur\Prefixity\target\phase1c-attempt-008-frozen\prefixity-phase1c-reasoning-budget-calibration.exe
size:    8657920
sha256:  0b859faae14dbcf5dad65612b646967fd79977173d67e5b2dabb570b861e5230
file_id: volume=ba2f80f4;index=000c00000038d6f2
```

The staged copies have their own Windows file identities; they are not claimed
to share the source build-output file IDs. The future live command uses only
the frozen paths above. Neither future launch command uses a mutable
`target\debug` execution object.

## Certification dependency and identity

The accepted v2 workflow certification recovered from the bounded local
certification namespace is:

```text
identity: target/phase1c-workflow-identity-certification-v2-final/identity.json
canonical SHA-256: acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194
state: WORKFLOW_IDENTITY_CERTIFIED
namespace: 900-v2
```

It records frozen supervisor/child binding, stable object identity, native
parent/child validation, pre-child-spawn mismatch rejection, independent
evidence validation, and no path-only fallback. Its exact staged binaries and
zero-contact accounting were reverified as part of Attempt-008 preflight.

The Attempt-008 identity is:

```text
identity: docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.json
sidecar:  docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.sha256
canonical SHA-256: 917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5
```

It binds the accepted baseline, manifest and case hashes, candidate budget and
provenance, complete runtime contract, accepted v2 certification, Attempt-008
frozen supervisor and child objects, staging policy, workflow exclusivity, and
the virgin evidence root.

## Virgin state and offline validation

The required classification was:

```text
ATTEMPT_008_VIRGIN
```

Before acceptance, no Attempt-008 identity, execution result, request ledger,
retry record, execution lock, or stale handoff existed. The Attempt-008
evidence root remained absent. No live supervisor/child or model server was
running, and the native pre-start inspection reported `EXCLUSIVE_PRESTART` with
zero llama processes and zero port-8080 listeners.

The offline commands passed:

```text
attempt-008-fingerprint
attempt-008-preflight
attempt-008-dry-run
attempt-008-validate-preparation
```

The dry run projected exactly the three fixed cases at budget 1024 with zero
network calls, zero listener checks, and zero inference requests.

Preparation-only workflow certification using the actual Attempt-008 frozen
supervisor and child returned `WORKFLOW_IDENTITY_CERTIFIED`. The child PID was
validated as a child of the supervisor PID. The certification namespace was
`target/phase1c-attempt-008-certification/`; it did not create an Attempt-008
execution result or consume the live Attempt-008 evidence root.

Substitution checks were performed before child spawn:

| Check | Result |
| --- | --- |
| Correct frozen supervisor + child | accepted; `WORKFLOW_IDENTITY_CERTIFIED` |
| Wrong supervisor (`target\debug`) | rejected: `FROZEN_EXECUTABLE_IDENTITY_MISMATCH` |
| Wrong child (`target\debug`) | rejected: `FROZEN_EXECUTABLE_IDENTITY_MISMATCH` |
| Same-content mutable-path object | rejected because the frozen Windows file identity differs |

No rejection path contacted the model or performed inference, and no
substitution evidence file was created.

## Future commands — not executed

Model-server command:

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 1024 --host 127.0.0.1 --port 8080
```

`NOT_EXECUTED`

Frozen-supervisor invocation:

```text
.\target\phase1c-attempt-008-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-008/supervisor.json -- .\target\phase1c-attempt-008-frozen\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-008
```

`NOT_EXECUTED`

The invocation binds `attempt_id=008`, budget `1024`, request ceiling `3`,
retries `0`, fallback `0`, adaptive replicates `0`, fixed case order, and the
staged supervisor/child objects. The supervisor will reject any object
substitution before child spawn.

## Accounting and final state

```text
model_server_startups=0
port_8080_contacts=0
tcp_readiness_contacts=0
http_model_requests=0
inference_requests=0
attempt_007_executions_added=0
attempt_008_executions=0
```

Required final state:

```text
ATTEMPT_008_PREPARED
ATTEMPT_008_NOT_EXECUTED
FROZEN_EXECUTABLE_IDENTITY_BOUND
```
