# Phase 1C h001 V2 BASELINE live execution record

Status: `CHILD_FAILED BEFORE MODEL CONTACT — STOP`

This record covers the single authorized launch for h001 V2 BASELINE,
replicate 1. It does not authorize a retry, a second BASELINE launch, a code
patch-and-rerun, NO_OP, INTERVENTION, h004, or another replicate.

## A. Pre-live state

The pre-live integrity gate passed on branch
`agent/phase-1c-h001-timeout-v2-prep` at
`df46280e9078762b81d62f3043177d151b524003`. The remote preparation branch
matched that SHA and `origin/main` remained at
`80152eb5ef55fa8e1c38d6ea97602cccbc615e63`. The V2 contract, pilot manifest,
and h001 identity hashes matched their sealed sidecars. The V2 evidence root
was absent, the worktree was clean, no Git lock existed, and all six V1
evidence hashes matched their frozen values. Pre-existing Phase 1C live
accounting was historical schema-smoke `2`, V1 h001 BASELINE `1`, and total
`3`.

The frozen BASELINE projection was revalidated as 1310 bytes with SHA-256
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`.

## B. Fresh runtime

The operator supplied current confirmation for the requested local Qwen
runtime: `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, Q4_0, context 8192, one slot,
metrics enabled, reasoning on, and `127.0.0.1:8080`. The server was confirmed
fresh for this V2 BASELINE with zero inference requests and no manual, browser,
warmup, or model-generating contact.

## C. Readiness

No TCP listener check was reached. The child exited during CLI argument parsing
before V2 preflight and before the runner's single permitted TCP readiness
check. Therefore readiness attempts were `0`, readiness elapsed time was not
observed, and readiness inference requests were `0`.

## D. Supervisor

The launch used the checked-in `prefixity-phase1c-live-supervisor` with the
already-built `prefixity-phase1c-h001-v2` child. The request timeout was
1200000 ms and the enforced supervisor deadline was 1320000 ms. The supervisor
launched exactly one child, retried zero times, and recorded `CHILD_FAILED`
with child exit code `1`; it did not terminate the child because the child
exited normally after printing usage. Supervisor network calls and inference
requests were both `0`.

The failure is a prepared-runner CLI defect at
`crates/prefixity-controlled-benchmark/src/bin/phase1c_h001_v2.rs:13-14`:
the parser expects `(None, Some(flag), None)` after `run`, while the supplied
arguments are `(Some(flag), None, None)`. The child therefore printed usage
and never entered `execute_v2_baseline`.

## E. BASELINE turns

No model request was dispatched. Transport attempts, HTTP results, prompt and
completion tokens, reasoning evidence, cache telemetry, finish reason,
response size, transport timing, request fingerprint, and response fingerprint
are all `NOT OBSERVED`. Total V2 BASELINE requests are `0`; completed turns are
`0`; terminal response and final-answer status are `NOT AVAILABLE`.

## F. Capability evaluation

Not evaluable. No certifiable model response or normalized terminal content
exists, so the deterministic evaluator was not invoked. This is a
pre-inference runner failure, not a capability result.

## G. Evidence

The only V2 evidence file is:

`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline/supervisor-result.json`

Its ordinary SHA-256 is
`bf65a2b4e9a326354566df8852d4d05e858f263e628cd46e046cb8d2f0492e5`. It
records the supervisor classification, child path/arguments, one launch, zero
retries, and zero supervisor network/inference calls. No request, response,
normalized, trajectory, or arm-result artifact was fabricated. The V2 runtime
directory is ignored; this supervisor result is retained as the bounded
execution record and was not force-added.

## H. Repository

This execution record is a documentation-only follow-up on the V2 preparation
branch. No V2 contract, manifest, identity, or Prefixity production behavior
was modified. The V1 evidence tree remains unchanged. A follow-up record
commit may be pushed to the same branch; no merge to `main` is authorized.

## I. Accounting and stop

- historical schema-smoke requests: `2`;
- V1 h001 BASELINE requests: `1`;
- V2 h001 BASELINE requests: `0`;
- NO_OP requests: `0`;
- INTERVENTION requests: `0`;
- h004 requests: `0`;
- retries: `0`;
- Prefixity behavioral changes: `0`.

The authorized V2 BASELINE launch is consumed as a child failure before model
contact. No retry or code-remediation rerun is permitted under this
authorization. A corrected runner requires separate offline remediation and
new live authorization.

`H001 V2 BASELINE CHILD_FAILED BEFORE MODEL CONTACT — STOP`
