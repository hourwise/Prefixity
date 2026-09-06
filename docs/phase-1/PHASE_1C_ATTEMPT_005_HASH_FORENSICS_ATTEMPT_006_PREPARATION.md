# Phase 1C Attempt 005 Hash Forensics and Attempt 006 Preparation

## Scope and stop boundary

Attempt 005 is permanently frozen as:

`1024 / ATTEMPT 005 — NOT STARTED`

Its pre-live gate failed before Qwen startup because the sealed identity
expected supervisor source SHA-256
`df74a196b79e0407c0d4d83f4a8aede974904a505a6462a72a9e868eda8e8744`, while
the Windows checkout presented
`d18e56fb68c0be8e58a10c94cc8333186614c7ef2f928bc3c252d143207d273f`.

No server startup, localhost contact, readiness check, request, inference, or
retry occurred. Attempt 005's identity and evidence state were not rewritten.

This record covers offline reconciliation only. Attempt 006 is prepared but
has not been executed.

## Hash provenance

The bound source path is:

`crates/prefixity-controlled-benchmark/src/phase1c_live_supervisor.rs`

The historical source bytes were checked out in detached LF-preserving Git
worktrees and hashed with ordinary SHA-256:

| Revision | Source SHA-256 | Interpretation |
| --- | --- | --- |
| `df46280e9078762b81d62f3043177d151b524003` | `b5144b63ed1c25966c8079e7fc7e6fddc1bbcbaa24293af1496329f3fdc1b6c3` | h001 timeout-only V2 predecessor |
| `56cdcc879b0488ac9459ca234bb5c8ccd1dd70cc` | `dfaf43f24189c31590fa41f42a21b0c6cc2af37f0c280e0fff35d19aacc2862b` | Attempt 004 handoff-remediation revision |
| `bab45ca7cf8bb859de8923e6f6097443915425e9` | `df74a196b79e0407c0d4d83f4a8aede974904a505a6462a72a9e868eda8e8744` | Attempt 005 preparation revision and sealed identity source |

The `df74…` source first appears as the reviewed Attempt 005 preparation
revision at `bab45ca…`. The observed `d18…` value is not a separate Git
revision: it is the checked-out Windows representation under
`core.autocrlf=true`.

The source fingerprint path previously hashed checkout bytes directly. It now
uses one explicit canonical definition for both sealing and verification:

`UTF-8 checkout source bytes; CRLF and lone CR normalized to LF; UTF-8 BOM rejected`

The canonicalizer has focused tests for LF/CRLF/mixed line endings and BOM
rejection. This reconciles the sealed `df74…` value without changing Attempt
005's frozen identity.

## Semantic comparison and authoritative revision

The difference between `df74…` and `d18…` is `NON_SEMANTIC` source-byte
representation. It does not affect supervisor-to-child handoff, launch
identity generation, PID/path metadata, timeout behavior, retry behavior,
child invocation, evidence paths, or model-facing request semantics.

Attempt 006 adds one narrowly scoped harness correction: the registered
identity loader binds the zero-padded `attempt_{N}_evidence_root` field and
accepts only the registered 1024 Attempts 005/006. This is classified as
`EXPERIMENT_HARNESS_SEMANTIC`; it changes no model-facing or runtime-contract
semantics.

The authoritative Attempt 006 supervisor source is the reviewed and tested
post-correction implementation at canonical SHA-256:

`76f122f4f6cf096a6a972d5770122b5590767467972aa43c7469d51b1b2656ef`

The native exclusivity source remains unchanged at:

`9dba33fdc0c4c9279e5df4eb12b3be0a0f5f99492f74efa548310ebf4163c37b`

## Attempt 006 identity and evidence boundary

Identity:

`docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_006_IDENTITY_V1.json`

Canonical identity SHA-256:

`1d326578deb5a5280eafbed5d844a754d1468429c4990b25a0ee207652ca28a1`

Evidence root:

`experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-006/`

The root remains absent during preparation. Attempt 006 binds the unchanged
manifest SHA-256
`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b` and the
unchanged request identities:

- `rbcal-001`: `f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461`
- `rbcal-002`: `e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3`
- `rbcal-003`: `2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e`

The runtime candidate remains reasoning on, server-side budget 1024, 8192
context, one slot, metrics enabled, max tokens 2048, temperature 0, top-p 1,
seed 1, request timeout 1200000 ms, supervisor timeout 1320000 ms, and zero
retries. No request-level reasoning-budget field is introduced.

## Offline validation

Completed without Qwen startup or localhost contact:

- Attempt 006 identity self-consistency test: passed.
- Supervisor-generated canonical handoff regression: passed.
- Attempt 006 fingerprint: passed.
- Attempt 006 preflight: passed with `EXCLUSIVE_PRESTART`.
- Attempt 006 dry-run: passed with exactly three ordered cases and zero
  network/listener/inference activity.

The next quality gates are the focused/full offline test suites, strict
Clippy, rustfmt check, and `git diff --check`. Attempt 006 remains preparation
only and requires separate live authorization.
