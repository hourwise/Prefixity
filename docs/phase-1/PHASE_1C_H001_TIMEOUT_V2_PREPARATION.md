# Phase 1C h001 timeout-only V2 preparation

Status: `PREPARATION COMPLETE — NO LIVE AUTHORIZATION`

This record covers only the approved timeout-only remediation preparation for
the h001 scored capability slice. It does not authorize a server start,
listener check, BASELINE request, NO_OP, INTERVENTION, another case, or
another replicate.

## Lineage and frozen identity

The accepted h001 V1 history was promoted from
`agent/phase-1c-scored-pilot-h001` to `main` at
`80152eb5ef55fa8e1c38d6ea97602cccbc615e63`, and
`agent/phase-1c-h001-timeout-v2-prep` was created from that promoted commit.
The original V1 request/evidence remains immutable. Its BASELINE request
projection is 1310 bytes with canonical SHA-256
`26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a`.

The V2 preparation artifacts are:

- `PHASE_1C_SCORED_RUNTIME_CONTRACT_V2.json` — canonical SHA-256
  `75dcc6a8a4db162e38557487c516ebe102ffebaa55c3e10a89a33d7d2c76b620`;
- `PHASE_1C_SCORED_PILOT_MANIFEST_V2.json` — canonical SHA-256
  `f8548455180f0e75d3e35a5662bbef3f79e2aca4ef506e281800c82a8feefb9e`;
- `PHASE_1C_H001_V2_BASELINE_IDENTITY_V1.json` — canonical SHA-256
  `dea4c693bcea61e7951c49d8d95b7d83294d10409d5c606b6d0f6a004f154bcc`.

Each artifact has a checked-in SHA-256 sidecar. The V2 identity binds the
frozen V1 task, source, required-state, tool, evaluator, arm-materialization,
and BASELINE projection hashes, and records the V1 timeout forensic review as
lineage.

## Timeout-only contract delta

The V2 validation compares the canonical V2 contract with V1 after allowing
only version/experiment/status, lineage, evidence-location/fingerprint
metadata, and the timeout policy to differ. Model, quantization, context,
parallelism, metrics, chat template, reasoning mode, sampling, seed, output
bound, retry policy, request ceilings, task/evaluator semantics, arm matching,
and cache/freshness rules remain frozen.

The applied timeout policy is:

| Setting | V1 | V2 |
| --- | ---: | ---: |
| Connect timeout | 1000 ms | 1000 ms |
| Complete request timeout | 600000 ms | 1200000 ms |
| Enforced outer supervisor deadline | 660000 ms declared-only | 1320000 ms |

The V2 evidence root is
`experiments/runs/phase1c-scored-capability-v2/`; the future h001 BASELINE
path is
`experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline/`.
No V2 evidence directory or live artifact was created during preparation.

## Supervisor boundary

The experiment-only `prefixity-phase1c-live-supervisor` binary launches an
already-built child runner supplied as an executable plus arguments. It uses
the production 1320000 ms deadline, permits exactly one child launch, performs
no retry or arm advance, and records one of:

- `COMPLETED` for a successful child;
- `CHILD_FAILED` for a child that exits unsuccessfully;
- `SUPERVISOR_TIMEOUT` after terminating and waiting for the child at the
  outer deadline.

The supervisor itself performs zero network calls and zero inference requests.
Its bounded unit tests cover all three classifications using short-lived
local child processes; no test waits for the production deadline.

The future V2 h001 runner consumes the V2 contract and therefore binds the
HTTP request timeout to 1200000 ms while retaining the frozen V1 model-visible
request. The live `run --confirm-fresh-runtime` path was not invoked.

## Offline validation and stop condition

Completed offline checks:

- V2 `fingerprint`, `preflight`, and `dry-run`;
- five V2 contract/manifest/identity/request/evidence-absence tests;
- three supervisor classification tests;
- `cargo fmt --all`;
- workspace strict Clippy with `-D warnings`.

The preflight and dry-run report zero network calls, zero credential reads,
and zero inference requests. The preparation branch is ready for a separate
operator authorization, but this task stops here. NO_OP, INTERVENTION, other
cases, and another replicate remain blocked.

## Security tooling boundary

Forensic and structural work used checked-in Rust validation, bounded ordinary
file inspection, and deterministic hashing. No large dynamically constructed
PowerShell forensic command was used, no endpoint protection was weakened, and
no preserved V1 evidence artifact was modified.
