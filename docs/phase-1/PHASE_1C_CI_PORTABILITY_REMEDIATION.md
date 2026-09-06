# Phase 1C CI Portability Remediation

This record covers only the clean-checkout CI remediation authorized before
Attempt 007. It does not authorize Qwen startup, localhost contact,
calibration inference, Attempt 007 preparation, or changes to preserved
experimental evidence and historical Attempt 006 identities.

## Accepted CI failures

The diagnosed failures were present before Attempt 006:

- Ubuntu and macOS Clippy compiled `ProbeFailureKind::ExecutablePath`, even
  though the variant was constructed only by Windows-native code.
- Windows h001 validation compared raw checkout bytes for `phase1b9.rs` with
  an LF-derived hash, making the check sensitive to checkout representation.
- Windows tests in the default workspace suite read ignored Stage 1 and
  reasoning-budget evidence roots that do not exist in a clean CI checkout.

The failures affect CI portability and test-boundary design only. They do not
alter any model request, runtime setting, inference count, or preserved result.

## Remediation boundary

`ProbeFailureKind::ExecutablePath`, its classification match arm, its test,
and the associated `ExclusivityOutcome::ExecutablePathFailed` variant and
stringification arm are now `cfg(windows)`-gated. No dead-code allowance is
used. The Windows implementation and its fail-closed outcomes are unchanged.

Source identity hashing is centralized in `hashing::canonicalize_source_bytes`:

1. input must be valid UTF-8;
2. a UTF-8 BOM is rejected;
3. CRLF and lone CR are normalized to LF;
4. the canonical bytes are hashed with SHA-256.

The h001 expected source hash remains
`2f1dae56606815034530b9a7114170eea08b9269eed57b18a4fb5d9747830a33`.
Focused tests cover LF, CRLF, lone CR, content mutation, BOM, and invalid
UTF-8 behavior.

Default tests now validate tracked contracts, manifests, schema, request
identity, historical identity sidecars, and classification logic without
requiring ignored evidence. Attempt 004 has a portable dry-run path that
does not read local evidence. Stage 1 contract/fixture preflight is likewise
portable and binds the accepted preserved hashes as lineage constants without
reading the files.

Actual ignored evidence remains a separate fail-closed operation:

```text
cargo run --offline --locked -p prefixity-controlled-benchmark --bin prefixity-phase1c-local-evidence-certification
```

That command reads but never writes preserved artifacts. It validates the
Stage 1 Smoke 01 response and forensic review, the completed Smoke 02 result,
and the preserved Attempt 003/004 calibration evidence. Missing evidence is
an error. The command is not part of the default clean-checkout test suite.

The accepted Attempt 004/006 identity files and sidecars remain unchanged.
Because the remediation necessarily changes the current Windows source
implementation, strict live/preflight identity checks continue to fail closed
until the separately authorized offline workflow-identity certification
reconciles the current source identity. This is deliberate and does not
rewrite prior classifications or evidence.

The first post-remediation Actions run (`34038819063`) confirmed the Windows
and MSRV jobs, then exposed the corresponding ungated outcome variant on
Ubuntu and macOS. That variant and its stringification arm were gated without
changing the frozen identities. The current-source fingerprint unit test was
updated only for the resulting source change; it is not an experimental
evidence hash or a historical identity sidecar.

That same run then exposed a POSIX-only test-helper quoting defect in the
supervisor handoff test: single quotes prevented expansion of the inherited
handoff environment variable. The helper now uses double quotes for POSIX
shell expansion. This changes only test portability and does not affect the
supervisor transport or any runtime/evidence path.

## Validation record

Local validation on the remediation branch:

- `cargo fmt --all -- --check`: passed;
- `cargo test --workspace --locked`: passed, 108 controlled-benchmark unit
  tests plus the workspace integration and documentation tests;
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`:
  passed;
- explicit local evidence certification: passed with zero additional network
  calls and zero additional inference requests;
- `git diff --check`: passed.

The clean-checkout worktree and GitHub Actions results are recorded below as
they are completed. No Qwen process or localhost endpoint was contacted by
this remediation.
