# Phase 1C reasoning-budget calibration — Attempt 008 execution record

Status: `ATTEMPT_008_EXECUTION_ACCEPTED`

Execution classification:

```text
ATTEMPT_008_EXECUTED_ONCE
ATTEMPT_008_INTEGRITY_ACCEPTED
ATTEMPT_008_CALIBRATION_ADMISSIBLE
```

This record reports the separately authorized single live execution from the
published preparation baseline. The raw response bodies and reasoning
diagnostics remain in the ignored local evidence root; this tracked record
publishes bounded metadata and hashes only.

## A. Starting baseline

The live gate was entered from the exact prepared publication:

```text
HEAD        b7f756d2d9ba538736bf5ef9765a5a2112fb3d31
main        b7f756d2d9ba538736bf5ef9765a5a2112fb3d31
origin/main b7f756d2d9ba538736bf5ef9765a5a2112fb3d31
worktree    clean
```

The Attempt-008 evidence root was absent before the live boundary. The
identity sidecar, preparation record, accepted v2 certification, and both
frozen execution objects passed their pre-live checks.

## B. Virgin and exclusivity gates

The pre-live state was virgin:

```text
ATTEMPT_008_VIRGIN
```

The native prestart inspection captured `EXCLUSIVE_PRESTART` before the
server was used. The human-started server was then verified as the one
accepted process; no competing llama process, Prefixity supervisor, child,
stale handoff, or competing Phase 1C workflow was present.

## C. Identity verification

Attempt-008 identity:

```text
917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5
```

Calibration manifest:

```text
4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b
```

Accepted v2 certification:

```text
acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194
```

The runtime supervisor and child matched the prepared object identities:

```text
supervisor SHA:      f07a9302786f3abe5a01eb9fa9eda85f50de9fe9f3d34feba14ba0b2e09c2088
supervisor size:     1259008
supervisor file ID:  volume=ba2f80f4;index=0007000000387abb

child SHA:           0b859faae14dbcf5dad65612b646967fd79977173d67e5b2dabb570b861e5230
child size:          8657920
child file ID:       volume=ba2f80f4;index=000c00000038d6f2
```

## D. Runtime server

The verified server command was:

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 1024 --host 127.0.0.1 --port 8080
```

The exact live process was:

```text
PID:        4608
executable: C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe
SHA:        cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617
size:       15277056
file ID:    volume=c4c93b54;index=00060000001ea970
```

The command line was independently recovered from the Windows process table
and matched the accepted configuration exactly. Readiness used the sole
permitted TCP listener-connect check to `127.0.0.1:8080`:

```text
readiness result:   passed
readiness contacts: 1
inference requests: 0 during readiness
```

## E. Supervisor execution

The exact frozen invocation was:

```text
.\target\phase1c-attempt-008-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-008/supervisor.json -- .\target\phase1c-attempt-008-frozen\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-008
```

Runtime process chain:

```text
supervisor PID: 120
child PID:      6312
child parent:   120
child launches: 1
child retries:  0
child exit:     0
supervisor:     COMPLETED
```

## F. Request ledger

| ordinal | case | HTTP | finish reason | prompt | completion | cached | validation | request SHA-256 | response SHA-256 |
| ---: | --- | ---: | --- | ---: | ---: | ---: | --- | --- | --- |
| 1 | `rbcal-001` | 200 | `stop` | 332 | 1125 | 0 | PASS / structural valid | `f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461` | `e90ae5b6807c525c53cb1bdbdc9ead83c10a7dbf9fcf58f4f34ccf692fd03161` |
| 2 | `rbcal-002` | 200 | `length` | 335 | 2048 | 42 | FAIL / structural invalid | `e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3` | `8308c6a77ec08e1aefe2f52fea6412dbd8c6c7d4820391cf90f0f3fd0607087d` |
| 3 | `rbcal-003` | 200 | `length` | 339 | 2048 | 42 | FAIL / structural invalid | `2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e` | `e5fdfb5ec13563e53fbff09e27360919a86387f44b72c19d678076c4f9b46177` |

Transport elapsed times were 187832 ms, 226160 ms, and 388402 ms in case
order. No fourth request was issued.

## G. Runtime accounting

```text
model_server_startups: 1 verified fresh candidate-1024 server process
readiness_contacts:    1
HTTP_requests:         3
inference_requests:    3
automatic_retries:     0
fallback_requests:     0
adaptive_replicates:   0
attempt_008_executions: 1
total network_calls:   4
```

The server was already running when the live prestart inspection was
performed, so no second server was started or substituted by the agent.

## H. Integrity gate

The independent post-run comparison passed:

```text
ATTEMPT_008_INTEGRITY_ACCEPTED
```

The runtime supervisor and child matched the prepared SHA-256, size, file ID,
and final-path identities. The native handoff confirmed supervisor PID `120`,
child PID `6312`, child parent PID `120`, and no competing workflow.

## I. Evidence freeze

Evidence root:

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-008/
```

The raw evidence inventory contains 23 files. The deterministic evidence
manifest and sidecar are:

```text
evidence-manifest.json SHA-256:
f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779

sidecar:
f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779  evidence-manifest.json
```

Manifest verification confirmed all 23 recorded file sizes and SHA-256
values. Key raw artifact hashes are:

```text
candidate-result.json  d482dfe73d43456481d2caf57a4968615ec4d04e1d24f445ff16c6c8fcef5b7e
preflight.json         b379ec5d34dd30c88ac88c8647552d3119d2bd175efcb5c26ac1fb4be375168a
readiness.json         dcccd1282bbe076dd6ae2e1461d7c7ed9a3556f39a22e78d1688e6698fdd49ad
supervisor.json        a5dc593f8d0e634dbb6646e927b04e5240ad117b27dd0834c51f7d8ada340787
```

Raw outputs were not edited, normalized, repaired, or retried.

## J. Calibration disposition

Because integrity was accepted, the frozen calibration protocol is admissible.
The raw candidate outcome is:

```text
rbcal-001: PASS
rbcal-002: FAIL — finish_reason=length; structural validation false
rbcal-003: FAIL — finish_reason=length; structural validation false

candidate state:       FAIL
case set complete:     true
selected budget:       null
protocol next_budget:   512
capability claims:     not permitted
```

The protocol’s `next_budget=512` is recorded as the disposition of this
consumed run. It was not executed under this authorization.

## K. Cleanup

The exact verified server PID `4608` was terminated after evidence freeze.
Post-run checks confirmed:

```text
server exited:          yes
port-8080 listener:     none
supervisor exited:      yes
child exited:           yes
stale handoff:          none
execution lock:         none
```

## L. Publication boundary

This result record is published separately from the preparation commit on:

```text
codex/phase1c-attempt-008-result
```

The ignored raw evidence root is intentionally not copied into tracked source
control. The bounded result metadata, raw hashes, and manifest verification are
published here without exposing response or reasoning contents.

Attempt 007 historical evidence and Attempt 006 preserved evidence were not
modified. No subsequent budget was executed.
