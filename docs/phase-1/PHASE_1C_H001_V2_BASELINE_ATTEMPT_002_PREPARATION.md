# Phase 1C h001 V2 BASELINE attempt 002 preparation

Status: `PREPARATION COMPLETE — NO LIVE AUTHORIZATION`

This record documents the authorized offline remediation of V2 BASELINE launch
attempt 001 and preparation of distinct launch attempt 002. It does not
authorize starting Qwen, performing readiness, dispatching BASELINE, or
running any other arm, case, or replicate.

## Attempt 001 preservation

The prior launch remains immutable at:

`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline/supervisor-result.json`

Its SHA-256 is
`bf65a2b4e9a326354566df8852d4d05e858f263e628cd46e046cb8d2f0492e5`.
It records `CHILD_FAILED`, child exit code `1`, one child launch, zero
retries, zero readiness checks, zero HTTP model requests, and zero inference
requests. No attempt-001 request or response evidence exists.

## Minimal CLI remediation

The defect was the V2 child parser matching
`(None, Some(flag), None)` after `run`, although the canonical argv is
`run --confirm-fresh-runtime`. The parser now accepts exactly that canonical
argv through a pure shared parser. Missing confirmation and malformed or
misplaced arguments fail closed before the live execution function.

The supervisor's registered child argv is provided by the same checked-in
helper and is tested against the child parser. The corrected canonical command
is:

`prefixity-phase1c-h001-v2 run --confirm-fresh-runtime`

No model, socket, HTTP, or credential behavior was changed.

## Attempt 002 identity and evidence boundary

The new identity is:

`docs/phase-1/PHASE_1C_H001_V2_BASELINE_ATTEMPT_002_IDENTITY_V1.json`

Canonical SHA-256:

`e8a28fb795f6c0ea635f2a8d9b5533f39b0ed33b7284d71449e8aa3b88f1a95c`

It binds V2 contract SHA
`75dcc6a8a4db162e38557487c516ebe102ffebaa55c3e10a89a33d7d2c76b620`, V2
pilot SHA
`f8548455180f0e75d3e35a5662bbef3f79e2aca4ef506e281800c82a8feefb9e`, the
frozen h001 artifacts, the 1310-byte BASELINE projection
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`,
replicate 1, BASELINE, attempt 002, and the immutable attempt-001 supervisor
record/root cause.

Attempt-002 evidence is isolated under:

`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline-attempt-002/`

The directory does not exist during preparation. Attempt 001's directory and
supervisor result were not moved, rewritten, or reused.

## Offline validation

The corrected child preflight and dry-run report `PREPARED`/`DRY_RUN`, zero
network calls, zero credential reads, zero listener checks, and zero inference
requests. The model-visible request remains exactly 1310 bytes with the frozen
V1/V2 BASELINE hash.

Focused tests cover canonical parsing, missing confirmation, malformed
placement, attempt-002 identity/preflight, request preservation, supervisor
classification, and supervisor-to-child argv compatibility. No live process
was started and no localhost contact occurred.

The V2 runtime contract remains unchanged: request timeout 1200000 ms,
supervisor deadline 1320000 ms, Qwen3.5-0.8B Q4_0, context 8192, one slot,
reasoning on, temperature 0, top-p 1, seed 1, max tokens 2048, and zero
retries.

This preparation ends before live execution. A new fresh-server confirmation
and separate live authorization are required for attempt 002.
