# Phase 1C Reasoning-Budget 1024 Attempt 006 Execution Record

## Classification

`REASONING BUDGET 1024 ATTEMPT 006 INVALID — STOP FOR REVIEW`

Attempt 006 is infrastructure-invalid before readiness and inference. The
registered supervisor generated and transmitted canonical handoff metadata,
but the child/native poststart ownership gate returned:

`EXPECTED_CHILD_PATH_MISMATCH`

The authorization requires an immediate stop for this condition. No Attempt
007 may be created under the Attempt 006 authorization.

## Accounting

- Qwen startup: `1`
- Model loaded: `1`
- Listener announcement: `1`
- Supervisor launches: `1`
- Poststart checks: `1`
- Readiness checks: `0`
- Calibration requests dispatched: `0`
- Inference requests: `0`
- Automatic retries: `0`
- Raw request artifacts: absent
- Raw response artifacts: absent
- Case results: absent
- Candidate result: infrastructure-invalid; 1024 remains unresolved

The server was stopped after the gate failure. Native verification confirmed
zero llama processes and no listener on port 8080.

## Frozen runtime and handoff

The authorized fresh server command used the frozen Attempt 006 settings:

- `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`
- Q4_0
- context `8192`
- one slot
- metrics enabled
- reasoning `on`
- reasoning budget `1024`
- reasoning-budget message unset
- host `127.0.0.1`
- port `8080`

The known non-fatal HTTPLIB repository lookup warning appeared before model
load. The model then loaded and announced the local listener. This warning was
not the stop cause.

The supervisor-generated launch identity was:

`phase1c-attempt-6-budget-1024-1d326578deb5a5280eafbed5d844a754d1468429c4990b25a0ee207652ca28a1`

The handoff bound the Attempt 006 identity SHA
`1d326578deb5a5280eafbed5d844a754d1468429c4990b25a0ee207652ca28a1`, budget
1024, candidate identity, Attempt 006 evidence root, supervisor PID/path, and
child PID/path through the single
`PREFIXITY_PHASE1C_WORKFLOW_HANDOFF` transport.

The observed native result also confirmed one llama process and a matching
port owner, but the expected child executable identity did not match the
native process-table representation. No readiness check was attempted after
that failure. The precise representation/normalization cause requires offline
forensic review; it must not be repaired during this sealed live epoch.

## Persisted evidence

Evidence root:

`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-006/`

Persisted artifact:

`supervisor.json`

SHA-256:

`c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07`

The supervisor artifact records one child launch, zero child retries, the
complete supervisor-generated handoff metadata, and zero supervisor
inference/network calls. The child’s poststart diagnostic was observed in the
supervisor invocation output but was not separately persisted by the checked-in
supervisor; no synthetic child-result artifact is created here.

Attempts 001–005 were not modified. Budgets 512 and 256 remain unexecuted.
No semantic candidate conclusion is supported.
