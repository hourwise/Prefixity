# Research-State Consistency Guard

The `research_state_consistency` integration test is a small deterministic
check for high-impact drift between Prefixity's current source-of-truth
documentation and machine-readable repository state. It is a consistency aid,
not a generic documentation linter.

## Checked invariants

The guard derives workspace members from `Cargo.toml` and compares them with
the bounded current-state marker in `docs/SOURCE_OF_TRUTH.md` and the README's
repository-layout section. It also checks that:

- the semantic current checkpoint is `phase-1c-research-state-v1`;
- Phase 1C Stage 0 is `CERTIFIED`;
- Phase 1C Stage 1 is `BLOCKED`;
- Phase 1C live replay is `NOT_STARTED`;
- the external front-half state is
  `EXTERNAL_TRAJECTORY_PERMISSION_PENDING`;
- `controlled-evidence-policy-v1` remains `CONTROLLED_ONLY`; and
- the implemented `prefixity.external-artifact-admission.v1` contract is
  referenced by current documentation.

The marker is deliberately small and exact. It is a human-readable
consistency aid, while `docs/SOURCE_OF_TRUTH.md` remains the human authority.
The checkpoint identifier is semantic rather than a raw Git SHA so ordinary
commits that preserve these conclusions do not break the guard.

## Authoritative files and historical exclusion

The guard reads only `Cargo.toml`, the bounded current sections of `README.md`,
the explicit marker in `docs/SOURCE_OF_TRUTH.md`, `docs/INDEX.md`, and the
external-artifact contract document. Historical phase evidence and design
documents are intentionally not inputs. They may describe earlier states
without causing a current-state failure.

## Intentional updates

To update current state, first complete the relevant research or authorization
work and revise the human source-of-truth prose and bounded marker together.
Update the guard's expected values and synthetic negative tests only when the
new state is intentionally accepted. Run the normal workspace checks and
review the staged diff. Do not rewrite historical evidence to make it match a
new state.

## Non-goals

This guard does not parse arbitrary prose semantically, evaluate benchmark
results, admit external artifacts, authorize provider calls, inspect
credentials, run replay, or integrate with the production planner. It does
not replace the evidence, licence, provenance, or admission contracts.

It runs as part of the existing offline workspace test command:

```text
cargo test --workspace --offline --locked
```
