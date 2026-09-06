# Phase 1C workflow-identity certification

This record describes the bounded offline certification introduced after the
Attempt-006 `EXPECTED_CHILD_PATH_MISMATCH` stop. It is a non-live identity
certification and does not prepare or execute Attempt 007.

The dedicated registered identity is attempt-style `900` and has the
candidate identity `phase1c-workflow-identity-certification`. It binds the
supervisor handoff to the unchanged 1024 calibration budget, three-case
ceiling, zero retries, the canonical main SHA, and the preserved Attempt-006
supervisor evidence hash. It is intentionally not an Attempt-007 identity.

The real certification invocation is:

```text
prefixity-phase1c-live-supervisor
  --certification-identity docs/phase-1/PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION_V1.json
  --evidence <dedicated certification supervisor result>
  --certification-result <dedicated certification child result>
  -- <real prefixity-phase1c-reasoning-budget-calibration executable>
```

The supervisor uses the same `run_supervised_with_registered_workflow_identity`
path as the future live calibration. The child performs no llama startup,
listener check, readiness request, HTTP request, or inference. It validates the
inherited handoff, registered identity, supervisor and child PIDs, stable
executable identities, parent relationship, evidence-root and candidate
bindings, and native absence of an unexpected competing workflow.

Executable identity is based on the raw path retained for forensics, the
resolved final path, file size, executable SHA-256, and the Windows file
identity returned by `GetFileInformationByHandle`. Path spelling alone is not
accepted. The Attempt-006 representation regression accepts equivalent path
spellings only after these stable identity checks succeed; a different file,
hash, PID, parent, missing handoff, or stale launch identity fails closed.

The deterministic certification record and fingerprint are kept beside this
document. Runtime PIDs and path spellings belong only to the dedicated
certification result, never to a historical experiment evidence root.
