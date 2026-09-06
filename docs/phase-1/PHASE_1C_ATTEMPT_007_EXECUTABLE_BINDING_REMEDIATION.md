# Phase 1C Attempt 007 executable-binding forensics and remediation

Status: PHASE_1C_FROZEN_EXECUTABLE_BINDING_REMEDIATION_ACCEPTED
Scope: offline forensic investigation and control-plane remediation only.

This record does not rerun Attempt 007, prepare Attempt 008, start a model
server, contact port 8080, or interpret new model output. The preserved
Attempt-007 evidence remains the historical input to the forensic validator.

## Starting state and immutable evidence

The accepted starting point was main and origin/main at
1f036e1c6f0afb0db23e3031069d37511a401aae, with a clean worktree. The
Attempt-007 identity canonical SHA-256 was
fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8, and
its evidence root was
experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-007/.

The preserved Attempt-006 supervisor artifact remains
c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07.
The Attempt-007 root, JSON files, and sidecars were read-only inputs. No
historical file was rewritten by this remediation.

The raw Attempt-007 signal remains:

    rbcal-001: PASS
    rbcal-002: FAIL / output limit
    rbcal-003: FAIL / output limit
    raw candidate: FAIL
    raw next_budget: 512

It is not admissible calibration evidence. In particular, it is not evidence
that budget 1024 fails and it is not an authorization to select budget 512.

## Executable provenance

The following facts separate recorded evidence from reconstruction:

| Object | PROVEN from durable records | SUPPORTED reconstruction | UNKNOWN |
| --- | --- | --- | --- |
| Preparation supervisor | Attempt-007 identity recorded SHA-256 8427e2ca64fce890e24d8d0474ee9e964e367b7d805caf0dbdbfb3abecc592db, size 1197056, and Windows file ID 0x0000000000000000000100000039821c. The preparation path was target\debug\prefixity-phase1c-live-supervisor.exe. | It was produced from the prepared checkout at commit c3059a1f2933ce0a4c9107ae03ec634b55c3fd69, using the documented locked/offline Cargo build workflow. | Exact compiler version, linker invocation, build timestamp, and the command that last wrote this object were not sealed in the identity. |
| Preparation child | Attempt-007 identity recorded SHA-256 844e02c7aab8d6223c93c4f9e1c94b74877c1ec766066037bf999d9f05745a39, size 8448000, and Windows file ID 0x00000000000000000006000000393382. The preparation path was target\debug\prefixity-phase1c-reasoning-budget-calibration.exe. | It was produced from the same prepared checkout/build directory and workspace profile as the supervisor. | Exact compiler/linker/build timestamp and last-writing operation are not recoverable from the frozen identity. |
| Runtime supervisor | Attempt-007 supervisor.json recorded SHA-256 f5c7e7eed98be6dcb8976f33b2bff6759c05e350d507e5b0914515fa64faadba, size 1187840, and file ID volume=ba2f80f4;index=001700000036a34d. It was launched from the same mutable target\debug pathname. | A later build-like operation replaced or relinked the target object after preparation. The size and hash transition prove an object change; the durable records do not identify one unique command. | Exact replacement command, replacement timestamp, compiler invocation, and whether checkout/promotion participated cannot be proven. |
| Runtime child | Attempt-007 supervisor.json recorded SHA-256 efedfa712758d5be9a0a0f24bce401f69de2153774f86ce33a685990cb2ffad0, size 8445440, and file ID volume=ba2f80f4;index=000600000039337f. | It was likewise resolved from the mutable target pathname after the prepared object had ceased to be the current object. | Exact replacement command and compiler/linker timestamp are unknown. |

Shared build facts are the workspace root D:\Users\fleur\Prefixity, target
directory target, Windows x86_64-pc-windows-msvc, the workspace dev profile
(unoptimized + debuginfo), and the locked dependency graph. The current
local toolchain inspected during remediation was Rust/Cargo 1.97.1; that
proves the present environment only, not the compiler used for either
historical executable pair. The current Cargo.lock SHA-256 was
79f4fb5ea5e2b698c9785ac609294ed44e793dd8674f49cd53001792a2491cf8.
Cargo.lock identity was not included in the Attempt-007 executable records,
so its historical pair-to-binary provenance is UNKNOWN.

The preparation record documents the offline commands
cargo build -p prefixity-controlled-benchmark --bins --locked --offline,
focused tests, formatting, Clippy, and full tests. It does not provide a
durable process log that orders every command relative to executable capture.
Therefore the narrow evidence-supported conclusion is an object replacement
under a mutable target pathname. It is not defensible to name one specific
Cargo command as the sole cause.

The runtime objects did not match the accepted source-bound preparation
objects. They may still have been compiled from accepted repository source;
the binary hashes alone do not establish source correspondence. No embedded
source or compiler provenance was recorded that could prove or disprove that
possibility.

## Validation gap and defect classification

Preparation recorded executable objects under
implementation_fingerprints.supervisor_binary and
implementation_fingerprints.child_binary. Runtime handoff generation
inspected and recorded the supervisor and child objects. The existing native
validator independently checked the supervisor PID, child PID, executable
identity, parent PID, launch identity, and process ownership. It correctly
treated DOS and extended path spelling as a representation boundary.

The missing edge was the comparison between the preparation identity and the
runtime handoff identity. Attempt-007 identity validation checked source,
manifest, request, certification, and handoff metadata, but omitted the two
prepared executable-object fields. Runtime validation was internally
self-consistent, so a changed supervisor and child could pass their own
runtime handoff checks and reach the calibration execution path. The earliest
available rejection point was supervisor metadata construction, before child
spawn; the child-side fallback point was immediately after inherited handoff
validation and before runtime ownership or inference.

This is classified as a fail-closed contract defect: the old implementation
could reach an inference-capable boundary while
runtime_executable_identity != prepared_executable_identity.

## Corrected invariant and lifecycle

The new invariant is:

    The supervisor and child executable objects authorized during preparation
    must be the executable objects used by the live workflow.

The binding compares SHA-256, file size, and the Windows volume/file identity.
On POSIX, the resolved final path remains part of the comparison because there
is no Windows file identity. On Windows, raw/final path spelling is retained
for diagnostics but is not the sole identity; equal file IDs and content
accept DOS versus \\?\ representations. A content, size, or file-object
difference returns FROZEN_EXECUTABLE_IDENTITY_MISMATCH.

The registered identity loader parses both complete executable objects. The
supervisor verifies the prepared supervisor and child before spawning the
child. The inherited handoff carries the frozen binding, and the child
re-inspects both its own image and its supervisor image before the
post-start/inference-capable gate. A registered workflow with no complete
frozen binding is rejected; there is no path-only fallback.

freeze_copy provides the lifecycle primitive for future preparation:
after build/test/Clippy work is complete, copy each executable to a bounded
staging directory, inspect the staged copy, bind the staged identity, and
refuse a later replacement at that destination. A staged Windows copy has a
new file ID by design. The content identity, frozen local file-object identity,
and runtime process-image identity are therefore recorded and compared at
their respective lifecycle boundaries.

## Regression coverage

The focused suite covers:

- equal supervisor and child objects accepted;
- different supervisor content rejected;
- different child content rejected;
- a rebuilt object at the same pathname rejected;
- DOS and extended Windows spellings accepted for the same file object;
- missing/partial preparation binding rejected without path-only fallback;
- missing registered binding rejected before child spawn;
- existing PID, parent, and native process-identity fail-closed tests retained;
- offline runtime evidence mismatch independently classified; and
- the preserved Attempt-007 mismatch fixture classified as
  FROZEN_EXECUTABLE_IDENTITY_MISMATCH without inspecting model output.

The fixture is
fixtures/phase1c/attempt-007-frozen-executable-mismatch.json. It records
only the prepared/runtime executable relationship. The original evidence
root remains the source of historical calibration records and is untouched.

## Workflow certification

The historical certification identity 900 and its canonical SHA-256
824b65a0f93e18a12e917fd49662790bdb2dce945c6e4f8f0c8b72a9c41524a4 remain
immutable. That record predates the frozen-object fields and is not rewritten.
The corrected certification namespace is versioned as 900-v2: it uses freshly
staged supervisor and child objects, a generated complete binding, real
supervisor-to-child handoff, and the same native PID/parent checks.
Substituting either staged object is rejected before any inference-capable
boundary.

The Windows offline certification completed with state
WORKFLOW_IDENTITY_CERTIFIED. The staged supervisor SHA-256 was
The staged supervisor SHA-256 was
72f4a64c9b7d852cad95b8c8811bcb083eefa8281b29cf796ed94395c65ae3f9, size
1259008, file ID volume=ba2f80f4;index=00110000002f2acc. The staged child
SHA-256 was 0f1a21bc97b6edf7737ba270af35f3d55658559587b353a982ffb5dd3773ae1,
size 8525824, file ID volume=ba2f80f4;index=000b0000002f2ae4. The observed
supervisor and child PIDs were 19208 and 20056, and the native process table
validated the child parent PID as 19208. The generated 900-v2 identity
canonical SHA-256 was acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194.

An independent substitution check passed: the staged supervisor was given the
mutable target/debug child path. It failed before spawn with
FROZEN_EXECUTABLE_IDENTITY_MISMATCH, produced no substitution evidence file,
and performed no model or network operation.

The corrected certification is offline-only:

    model_server_startups=0
    port_8080_contacts=0
    tcp_readiness_contacts=0
    http_model_requests=0
    inference_requests=0

## Attempt state

The state model now distinguishes:

    PREPARED
    EXECUTED
    INTEGRITY_ACCEPTED
    CALIBRATION_ADMISSIBLE
    REUSABLE

Attempt 007 is permanently consumed:

    PREPARED = true
    EXECUTED = true
    INTEGRITY_ACCEPTED = false
    CALIBRATION_ADMISSIBLE = false
    REUSABLE = false
    ATTEMPT_007_EXECUTED_ONCE
    ATTEMPT_007_INTEGRITY_REJECTED
    ATTEMPT_007_CALIBRATION_INADMISSIBLE
    ATTEMPT_007_PERMANENTLY_CONSUMED
    NEXT_FRESH_ATTEMPT_ID = 008

The protocol permits a fresh attempt number after this consumed integrity
failure, but no Attempt-008 identity has been created and no next candidate
budget has been inferred. The raw next_budget: 512 is not admissible.

## Remediation accounting

No Attempt-007 execution was added, no Attempt-008 execution occurred, and
no model-facing activity was performed during remediation:

    attempt_007_executions_added=0
    attempt_008_executions=0
