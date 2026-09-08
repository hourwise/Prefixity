# Phase 1C reasoning-budget calibration — Attempt 009 execution record

Status: `ATTEMPT_009_INTEGRITY_REJECTED`

Execution classification:

```text
ATTEMPT_009_EXECUTED_ONCE
ATTEMPT_009_INTEGRITY_REJECTED
ATTEMPT_009_CALIBRATION_INADMISSIBLE
```

This record reports the separately authorized single live boundary from the
accepted Attempt-009 preparation. The frozen supervisor launched the frozen
child exactly once, but the child failed before readiness or inference during
candidate-order validation. The raw supervisor record and deterministic
failure manifest remain in the ignored local evidence root. No rerun, repair,
replacement request, or calibration interpretation was performed.

## A. Starting baseline

The live gate was entered from the exact promoted preparation:

```text
HEAD        d3895008932327ed625701ac86c3efb9fbfd34f0
main        d3895008932327ed625701ac86c3efb9fbfd34f0
origin/main d3895008932327ed625701ac86c3efb9fbfd34f0
worktree    clean
```

The Attempt-009 identity sidecar, preparation record, accepted v2
certification, and both frozen execution objects passed their pre-live checks.

## B. Virgin and exclusivity gates

Before the live boundary:

```text
ATTEMPT_009_VIRGIN
EXCLUSIVE_PRESTART
```

The Attempt-009 evidence root, request ledger, result, retry record,
execution lock, and stale handoff were absent. There was no Prefixity
supervisor or calibration child. One exact operator-started llama server was
present and owned port 8080 exclusively.

## C. Identity verification

Attempt-009 identity:

```text
0929aae1d371415d3efd92e4e7310490ad06fb69fd0398aa76aa9d857818f812
```

Calibration manifest:

```text
4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b
```

Accepted v2 certification:

```text
acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194
```

Prepared frozen objects:

```text
supervisor SHA:      aaba6a8202ccc88c4ea6c277b5c20058588cabdd98c5e5d101756e40e33aa2fb
supervisor size:     1259008
supervisor file ID:  volume=ba2f80f4;index=000500000038291e

child SHA:           c5c4913d1b4719b333d972ecfff11530fadbf24aaac99fde71f5a5bb8f4e1beb
child size:          8813056
child file ID:       volume=ba2f80f4;index=000a000000382aae
```

The supervisor-generated runtime handoff recorded the same supervisor and
child SHA-256, size, file ID, and final-path identities. Runtime identity
equality therefore passed. The child failed before the post-start gate, so
an independent post-start ownership/parent-lineage record was not produced.

## D. Runtime server

The exact verified server command was:

```text
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 512 --host 127.0.0.1 --port 8080
```

The server was already running when the gate was entered; no second server
was started by the agent:

```text
PID:        25476
executable: C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe
SHA:        cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617
size:       15277056
```

The process-table command line matched the accepted configuration exactly.
It was the sole llama/Qwen process and sole owner of port 8080.

## E. Supervisor execution

The exact frozen invocation was:

```text
.\target\phase1c-attempt-009-frozen\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-009/supervisor.json -- .\target\phase1c-attempt-009-frozen\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-009
```

The supervisor evidence records:

```text
supervisor PID: 260
child PID:      19444
child launches: 1
child retries:  0
child exit:     1
supervisor:     CHILD_FAILED
```

The child failed before readiness with the exact validation error:

```text
candidate order is not satisfied: prior candidate result is absent
```

This exposed a harness validation defect: the Attempt-009 child’s candidate
order precondition looks for a prior candidate result in the default
candidate path and does not resolve the accepted Attempt-008 evidence root.
The defect is preserved for separate remediation and was not changed in this
result task.

## F. Request ledger

No request rows exist. The failure occurred before the single permitted
readiness contact and before any HTTP or inference request.

```text
ordinal | case | HTTP | finish_reason | prompt | completion | cached | structural result | request hash | response hash
none
```

## G. Runtime accounting

```text
model_server_startups: 1 verified exact operator-started server process
readiness_contacts:    0
HTTP_requests:         0
inference_requests:    0
automatic_retries:     0
fallback_requests:     0
adaptive_replicates:   0
attempt_009_executions: 1
total network_calls:   0
```

## H. Integrity gate

The prepared and runtime supervisor/child object identities matched. However,
the run did not reach readiness, produced no complete case evidence, and did
not produce the required post-start lineage record. The independent integrity
gate therefore rejects the consumed run:

```text
ATTEMPT_009_INTEGRITY_REJECTED
ATTEMPT_009_CALIBRATION_INADMISSIBLE
REASON: CHILD_FAILED_BEFORE_READINESS
```

This is not a frozen-executable identity mismatch. The prepared frozen object
binding remains unchanged.

## I. Calibration result

The frozen calibration protocol was not applied because integrity was not
accepted:

```text
rbcal-001: NOT_RUN
rbcal-002: NOT_RUN
rbcal-003: NOT_RUN
candidate_budget: 512
candidate_state: NOT_APPLIED
case_set_complete: false
selected_budget: null
next_budget: null
```

## J. 1024 versus 512 descriptive comparison

Attempt 008 remains the last admissible calibration result:

```text
case       Attempt 008 (budget 1024)                  Attempt 009 (budget 512)
rbcal-001  HTTP 200, stop, 1125 tokens, PASS          NOT_RUN
rbcal-002  HTTP 200, length, 2048 tokens, FAIL       NOT_RUN
rbcal-003  HTTP 200, length, 2048 tokens, FAIL       NOT_RUN
```

Attempt 009 has no finish reason, completion token count, or structural
validation result because it made no model request. No post-hoc scoring rule
was created.

## K. Evidence freeze

The bounded raw evidence root is:

```text
experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-009/
```

The raw supervisor record was not edited:

```text
supervisor.json bytes: 3819
supervisor.json SHA-256: 83d4fee4203672881f9ca8e26d7440c3fccae19f67315eb0352e1fc74bf8e077
```

The deterministic failure manifest and sidecar are:

```text
evidence-manifest.json SHA-256:
3f38b73f46800c1682f8e5dcc2f23ccc6788016fa48e00af86c6982810d3cc27

sidecar:
3f38b73f46800c1682f8e5dcc2f23ccc6788016fa48e00af86c6982810d3cc27  evidence-manifest.json
```

The manifest contains one raw file, records the exact pre-readiness failure,
and validates successfully against the sidecar. No response, request,
readiness, result, retry, or repair artifacts were created.

## L. Cleanup

After evidence freeze, the exact verified server PID `25476` was terminated.
Post-run checks confirmed:

```text
server exited:          yes
port-8080 listener:     none
supervisor exited:      yes
child exited:           yes
stale handoff:          none
execution lock:         none
```

Attempt 009 is permanently consumed by this one-time boundary. It must not be
rerun. Any harness remediation requires a separate task and cannot change this
record or reinterpret it as calibration evidence.

## M. Publication boundary

This bounded failure result is published separately from the preparation
commit on the dedicated Attempt-009 result branch. The ignored raw evidence
root is not copied into tracked source control. Attempt 008 admissible
evidence, Attempt 007 historical evidence, and Attempt 006 preserved evidence
were not modified.
