# Phase 1C reasoning-budget calibration — Attempt 011 execution record

Status: `ATTEMPT_011_INTEGRITY_ACCEPTED` — terminal calibration result

Execution classification:

```text
ATTEMPT_011_EXECUTED_ONCE
ATTEMPT_011_INTEGRITY_ACCEPTED
ATTEMPT_011_CALIBRATION_ADMISSIBLE
```

This record reports the single live boundary of the accepted Attempt-011
preparation at candidate budget 256, the last registered calibration
candidate. The frozen supervisor launched the frozen child exactly once; the
child completed three inference requests without retry. The admissible
candidate result is `FAIL`, which under the frozen protocol is terminal.

## A. Starting baseline and provenance

```text
HEAD / main / origin/main  427cfc9847b698bb3516c209895ea8db7cd5311a
worktree                   clean
source provenance          4dd619e82a4d494f0ee8ca954440d46bb30f300e
attempt-011 identity       60b442a1b09a4ff817b1ca40c103853f6972baeb3d13c91f551cb58ff5bd03ed
```

The identity hash was confirmed independently, against its sidecar, and by
the frozen child. The bound source equalled `4dd619e`, the transition-registry
hash matched the identity, and the repository contract resolved the
authoritative predecessor Attempt 010 (512 `FAIL` -> 256).

## B. Frozen executable identities

| Object | SHA-256 | Size | Windows file ID |
| --- | --- | --- | --- |
| supervisor | `9dbf6dff7d914bce6f620fb2455bc64d304015ed6c2321d5ca7a7f935f72b8b1` | 1259008 | `volume=ba2f80f4;index=0006000000387bbc` |
| child | `10533899e3d9bc1e35b6ae83e518f499e66ee7d819a2b64f8626fae76e8e15d5` | 9281536 | `volume=ba2f80f4;index=0005000000387bbd` |

Both matched on disk before the run and in the supervisor-generated runtime
handoff.

## C. Runtime server verification

The server was started by the operator with the accepted command; the agent
did not start, restart, or reconfigure it:

```text
PID:        6880 (parent 29520), created 2026-09-25 14:29:56 local
executable: C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe
SHA-256:    cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617
size:       15277056
file index: 00060000001ea970
command:    C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 256 --host 127.0.0.1 --port 8080
```

The process-table command line matched the accepted command exactly. It was
the sole llama process and sole owner of the single `Listen` socket on
127.0.0.1:8080, with no competing Prefixity or model-server workflow.
Verification used read-only process and TCP tables; the agent made no port
contact.

## D. Supervisor execution

```text
.\target\phase1c-attempt-011-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-256-attempt-011/supervisor.json -- .\target\phase1c-attempt-011-frozen\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-011
```

```text
started (UTC):  2026-09-25T13:31:50Z
supervisor PID: 6220
child PID:      29544 (parent PID 6220)
child launches: 1
child retries:  0
child exit:     0
supervisor:     COMPLETED (stderr empty)
```

The child passed the shared live prerequisites (`RUNTIME_DEPENDENCIES_COMPLETE`,
`PREDECESSOR_TRANSITION_VALID`, `CANDIDATE_ORDER_VALID`, runtime/contract
parity, source 10 / 512 -> 256), then the post-start gate recorded
`EXCLUSIVE_POSTSTART` with the port-8080 owner equal to the verified server
PID 6880.

## E. Request ledger

| ordinal | case | HTTP | finish reason | prompt | completion | cached | validation | request SHA-256 | response SHA-256 |
| ---: | --- | ---: | --- | ---: | ---: | ---: | --- | --- | --- |
| 1 | `rbcal-001` | 200 | `stop` | 332 | 359 | 0 | FAIL / structural invalid | `f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461` | `c4a4820e1a6aee423229c4098e1c4f842ae9e43ffddaf140e62fd83bfe2a46aa` |
| 2 | `rbcal-002` | 200 | `length` | 335 | 2048 | 42 | FAIL / structural invalid | `e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3` | `9fb330fdee38520ac4206665e37307fcef02c1202f86844bb6b6f60e8647d894` |
| 3 | `rbcal-003` | 200 | `stop` | 339 | 298 | 42 | FAIL / structural invalid | `2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e` | `57b3cb4d989f3b9ca3789fd3fce5e525b80db3583c9e13a37a0f2ca92cf93fc0` |

Failure classes: rbcal-001 and rbcal-003 — exact structural/schema failure
("terminal content failed the exact structural response schema"); rbcal-002
— token exhaustion before completion ("total 2048-token ceiling exhausted
before terminal completion"). Transport elapsed times were 35405 ms,
257989 ms, and 37457 ms. All request hashes equal the frozen hashes; no
request carried a reasoning-budget field. No fourth request was issued.
Response and reasoning text is not reproduced.

## F. Runtime accounting

```text
operator_model_server_startups = 1
agent_model_server_startups    = 0
readiness_contacts             = 1
http_model_requests            = 3
inference_requests             = 3
retry_requests                 = 0
fallback_requests              = 0
adaptive_replicates            = 0
warmup_requests                = 0
attempt_011_executions         = 1
total network_calls            = 4 (1 readiness + 3 inference)
```

## G. Integrity gate

The independent post-run comparison passed all checks before any calibration
outcome was read:

```text
ATTEMPT_011_INTEGRITY_ACCEPTED
```

- Supervisor completed once: one launch, no retry, exit 0, no supervisor
  network or inference; child arguments exactly `run-attempt-011`.
- Handoff bound attempt 11, budget 256, identity `60b442a1…03ed`; supervisor
  and child runtime identities equal the frozen binding.
- Native lineage: child 29544's parent is supervisor 6220.
- Live prerequisites all true; runtime candidate order equals the contract.
- Post-start `EXCLUSIVE_POSTSTART` on verified server PID 6880; the server's
  PID and creation time were unchanged after the run.
- Exactly one passed readiness check; runtime launch arguments equal the
  contract server command.
- Three requests in the fixed case order with the frozen hashes and
  generation settings; no retry, fallback, or replicate.
- The evidence root contains exactly the 23 expected raw artifacts, all
  matching the sealed manifest.

## H. Evidence freeze

Evidence root (ignored, not copied into source control):

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-256-attempt-011/
```

```text
evidence-manifest.json SHA-256:
5e8ccbc7be4fe5cb9b6d4041a3fc4c47943d5a889f0a4916ebab4e8fbebf73cd

sidecar:
5e8ccbc7be4fe5cb9b6d4041a3fc4c47943d5a889f0a4916ebab4e8fbebf73cd  evidence-manifest.json
```

The manifest was generated before any outcome was read. Verification confirmed
all 23 recorded sizes and SHA-256 values, with no unlisted files, at sealing
and again before publication. Key raw artifact hashes:

```text
candidate-result.json      e1fc6c9b381fd63617859171e555c330db2df945bca3e893f27726b2cba9efa2
preflight.json             0b7e37f7b73c47cff22de0d55c4f4e842765f90f257296f8a4fe25cda968dff6
readiness.json             dcccd1282bbe076dd6ae2e1461d7c7ed9a3556f39a22e78d1688e6698fdd49ad
runtime-confirmation.json  bc0d95224e805b0b67163babadd567c57ed0a344b529e5275bf76bf5518117e6
supervisor.json            4f6d2673face9fa4afa4e6864e841b6efe505d7617210e056d187619a412cb11
```

Raw outputs were not edited, normalized, repaired, or retried.

## I. Calibration disposition

Because integrity was accepted, the frozen calibration protocol is
admissible:

```text
rbcal-001: FAIL — finish_reason=stop;   359 completion tokens; exact structural/schema failure
rbcal-002: FAIL — finish_reason=length; 2048 completion tokens; token exhaustion before completion
rbcal-003: FAIL — finish_reason=stop;   298 completion tokens; exact structural/schema failure

CANDIDATE_BUDGET_256 = FAIL
CASE_SET_COMPLETE    = true
SELECTED_REASONING_BUDGET = null
NEXT_BUDGET          = null
capability claims    = not permitted
```

The frozen candidate order is 1024, 512, 256. Budget 256 is the last
registered candidate, so the protocol produces its terminal classification,
recorded in the candidate result exactly as defined by
`PHASE_1C_REASONING_BUDGET_CALIBRATION_DESIGN.md` and calibration manifest V1:

```text
REASONING-ON / 2048-TOKEN SCORED CONFIGURATION NOT FEASIBLE
(REASONING_ON_2048_TOKEN_SCORED_CONFIGURATION_NOT_FEASIBLE)
```

Meaning: under the frozen Phase 1C calibration protocol, none of the
registered reasoning-on budgets 1024, 512, and 256 produced a complete
passing three-case set within the fixed 2048-token output ceiling. The claim
is bounded to `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0` (Q4_0), the three registered
calibration cases, this reasoning-on configuration, the 2048-token ceiling,
and this frozen protocol. It is not a claim about Qwen models generally,
reasoning generally, other quantizations, larger context or output budgets,
or other workloads. No lower budget is registered and none was created.

## J. Descriptive calibration history

Descriptive observations only; no comparative scoring, weighting, or trend
claim is made, and prior classifications are unchanged.

```text
case       Attempt 008 (1024)   Attempt 010 (512)   Attempt 011 (256)
rbcal-001  PASS                 FAIL                FAIL
rbcal-002  FAIL                 PASS                FAIL
rbcal-003  FAIL                 FAIL                FAIL
```

## K. Experimental state

```text
LAST_ADMISSIBLE_ATTEMPT   = 011
LAST_ADMISSIBLE_BUDGET    = 256
LAST_ADMISSIBLE_STATE     = FAIL
CALIBRATION_TERMINAL      = true
SELECTED_REASONING_BUDGET = null
NEXT_CANDIDATE_BUDGET     = null
ATTEMPT_011_CONSUMED      = true
```

The authoritative transition registry is unchanged: it records only
admissible `FAIL` transitions to a protocol successor, and 256 has none.

## L. Cleanup

After evidence sealing and integrity classification, the exact verified
server PID 6880 (identity re-confirmed immediately before) was terminated.
Post-run checks confirmed:

```text
server exited:          yes
llama processes:        0
port-8080 listener:     none
supervisor exited:      yes
child exited:           yes
stale handoff:          none
execution lock:         none
```

Attempt 011 is permanently consumed by this one-time boundary and must not be
rerun. Attempt 008, 009, and 010 evidence was re-verified and not modified.
