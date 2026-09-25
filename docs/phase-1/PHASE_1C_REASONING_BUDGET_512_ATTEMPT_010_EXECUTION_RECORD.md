# Phase 1C reasoning-budget calibration — Attempt 010 execution record

Status: `ATTEMPT_010_INTEGRITY_ACCEPTED`

Execution classification:

```text
ATTEMPT_010_EXECUTED_ONCE
ATTEMPT_010_INTEGRITY_ACCEPTED
ATTEMPT_010_CALIBRATION_ADMISSIBLE
```

This record reports the single live boundary of the accepted Attempt-010
preparation at candidate budget 512. The frozen supervisor launched the
frozen child exactly once; the child completed three inference requests
without retry. This is the first executed evidence at budget 512.

## A. Starting baseline

```text
HEAD        87cbacd3cc980d2019142e496014b58ff8548372
main        87cbacd3cc980d2019142e496014b58ff8548372
origin/main 87cbacd3cc980d2019142e496014b58ff8548372
worktree    clean
```

Before the live boundary the frozen child reported
`attempt-010-fingerprint` = PREPARED with identity
`9292e9ecdd2e89f695dfb34bc782ade41b70412c427807c6c3a5653b50ec16f7`, both frozen
objects matched their recorded SHA-256, size, and file ID, and
`attempt-010-repository-contract` was valid. The Attempt-010 evidence root was
absent.

## B. Runtime server verification

The server was started by the operator with the accepted command; the agent
did not start, restart, or reconfigure it:

```text
PID:        18048 (parent 14188), created 2026-09-25 12:32:16 local
executable: C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe
SHA-256:    cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617
size:       15277056
file index: 00060000001ea970
command:    C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 512 --host 127.0.0.1 --port 8080
```

The process-table arguments matched the contract token for token (only the
shell quoting of argv[0] differs). It was the sole llama process and the sole
owner of the single `Listen` socket on 127.0.0.1:8080, with no established
connections and no competing Prefixity/Qwen workflow process. Verification was
by read-only process and TCP tables; no port contact was made by the agent.

## C. Supervisor execution

```text
.\target\phase1c-attempt-010-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-010/supervisor.json -- .\target\phase1c-attempt-010-frozen\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-010
```

```text
started (UTC):  2026-09-25T11:34:18Z
finished (UTC): 2026-09-25T11:42:36Z
supervisor PID: 11740
child PID:      21240 (parent PID 11740)
child launches: 1
child retries:  0
child exit:     0
supervisor:     COMPLETED
```

The child passed the shared live prerequisites (identical to the offline
traversal), then the post-start gate recorded `EXCLUSIVE_POSTSTART`: exactly
one llama process, port-8080 owner equal to the verified server PID 18048,
`NO_UNEXPECTED_WORKFLOW_PROCESSES`.

## D. Request ledger

| ordinal | case | HTTP | finish reason | prompt | completion | cached | validation | request SHA-256 | response SHA-256 |
| ---: | --- | ---: | --- | ---: | ---: | ---: | --- | --- | --- |
| 1 | `rbcal-001` | 200 | `length` | 332 | 2048 | 0 | FAIL / structural invalid | `f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461` | `3129ce57f5d1253847775cbaef86e5ea24584ee63948cf72566c8536e71e002d` |
| 2 | `rbcal-002` | 200 | `stop` | 335 | 547 | 42 | PASS / structural valid | `e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3` | `aed7901ff788275e2a01367182d08dfdee0104451a7a6d4e1c5c633c8ad90f43` |
| 3 | `rbcal-003` | 200 | `stop` | 339 | 554 | 42 | FAIL / structural invalid | `2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e` | `52e70384f8e17af9b11a104b15577a64d5ab68a13830561456b99941378fb8ec` |

Recorded validation errors: rbcal-001 "total 2048-token ceiling exhausted
before terminal completion"; rbcal-003 "terminal content failed the exact
structural response schema". Transport elapsed times were 303237 ms,
100324 ms, and 85141 ms. All request hashes equal the frozen hashes; no
request carried a reasoning-budget field. No fourth request was issued.

## E. Runtime accounting

```text
model_server_startups:  1 verified operator-started server (agent started none)
readiness_contacts:     1 (passed)
HTTP_requests:          3
inference_requests:     3
automatic_retries:      0
fallback_requests:      0
adaptive_replicates:    0
attempt_009_executions_added: 0
attempt_010_executions: 1
total network_calls:    4
```

## F. Integrity gate

The independent post-run comparison passed:

```text
ATTEMPT_010_INTEGRITY_ACCEPTED
```

- Supervisor and child identities in the supervisor-generated handoff equal
  the prepared SHA-256, size, and file IDs; the handoff binding equals the
  identity binding; attempt 10, budget 512, identity `9292e9ec…16f7`.
- Native parent/child lineage: child 21240 has parent 11740 (the supervisor).
- Post-start ownership `EXCLUSIVE_POSTSTART` on the verified server PID.
- Exactly one passed readiness check; runtime launch arguments equal the
  contract server command; three requests with frozen hashes and the frozen
  generation settings; no retry, fallback, or replicate.
- The evidence root contains exactly the 23 expected raw artifacts.
- The server PID and creation time were unchanged after the run.

## G. Evidence freeze

Evidence root (ignored, not copied into source control):

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-010/
```

```text
evidence-manifest.json SHA-256:
5673e55b381d1f5171c38bc9f1721b3105adf829ece4ec029cbcca4db150c4b9

sidecar:
5673e55b381d1f5171c38bc9f1721b3105adf829ece4ec029cbcca4db150c4b9  evidence-manifest.json
```

Manifest verification confirmed all 23 recorded sizes and SHA-256 values. Key
raw artifact hashes:

```text
candidate-result.json      85c31918110c485ed69a6473290a694fa7349690b30bff4873da77625479cc21
preflight.json             d085eed7576bddd249cc2e7008c18b4fbd355b713b15c61571ec59c70e1e8c3b
readiness.json             dcccd1282bbe076dd6ae2e1461d7c7ed9a3556f39a22e78d1688e6698fdd49ad
runtime-confirmation.json  69229edd51241f62696f2aca3ee8593f05c9e9c447e0e9574c8da8ef1f38d2a8
supervisor.json            40c911c7815146583e2717cc4cf0c2dde3863461eb1cac0d74809c8a47f06065
```

`readiness.json` is byte-identical to Attempt 008's, as expected for a single
passed connect record with the same elapsed value. Raw outputs were not
edited, normalized, repaired, or retried.

## H. Calibration disposition

Because integrity was accepted, the frozen calibration protocol is
admissible. The raw candidate outcome is:

```text
rbcal-001: FAIL — finish_reason=length; 2048 completion tokens; structural validation false
rbcal-002: PASS — finish_reason=stop; 547 completion tokens
rbcal-003: FAIL — finish_reason=stop; 554 completion tokens; structural validation false

candidate state:      FAIL
case set complete:    true
selected budget:      null
protocol next_budget: 256
capability claims:    not permitted
```

The protocol's `next_budget=256` is recorded as the disposition of this
consumed run. It was not prepared or executed. Before any 256 attempt is
prepared, the known deferred budget-256 candidate-order defect (generic
predecessor lookup) must be remediated and validated.

## I. 1024 versus 512 descriptive comparison

Descriptive only; no post-hoc scoring rule was created.

```text
case       Attempt 008 (budget 1024)                 Attempt 010 (budget 512)
rbcal-001  HTTP 200, stop,   1125 tokens, PASS       HTTP 200, length, 2048 tokens, FAIL
rbcal-002  HTTP 200, length, 2048 tokens, FAIL       HTTP 200, stop,    547 tokens, PASS
rbcal-003  HTTP 200, length, 2048 tokens, FAIL       HTTP 200, stop,    554 tokens, FAIL (schema)
```

## J. Cleanup

After evidence freeze, the exact verified server PID 18048 (identity
re-confirmed immediately before) was terminated. Post-run checks confirmed:

```text
server exited:          yes
llama processes:        0
port-8080 listener:     none
supervisor exited:      yes
child exited:           yes
stale handoff:          none
execution lock:         none
```

Attempt 010 is permanently consumed by this one-time boundary and must not be
rerun. Attempt 008, Attempt 009, and earlier evidence were not modified.
