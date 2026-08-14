# P0-L6E Fresh-Arm Paired-Mutation Experiment Preparation

P0-L6E is an offline design and implementation slice following the P0-L6D
reconciliation of Attempt 006. It prepares a deconfounded live experiment but
does not execute it. No listener is checked, no localhost request is sent, no
llama.cpp process is started or stopped, and no Attempt 007 evidence is
created by this preparation.

## Purpose and boundary

The five-case P0-L6B runner interleaves control, interference, and treatment in
one runtime epoch. Attempt 006 completed that bounded sequence, but its one-slot
runtime accounting could not separate fresh-server state from layout effects.
P0-L6E therefore prepares two independently executable arms:

| Arm | Cases | Runtime epoch |
| --- | --- | --- |
| Control | `A0` → `A1` | `control_epoch_id` |
| Treatment | `C0` → `C1` | `treatment_epoch_id` |

`B1` is intentionally absent. A fresh-server boundary replaces the in-sequence
interference case for this design. The two epoch IDs are caller-supplied and
must be distinct. Epoch IDs identify execution instances; they are not part of
the semantic experiment identity.

The implementation does not orchestrate a restart. The caller executes and
finalizes the control arm, persists its record, obtains operator confirmation
of a fresh listener, and only then requests treatment execution. The two arms
cannot be silently combined when an epoch, runtime profile, configuration
fingerprint, request identity, or freshness assertion differs.

## Reused contracts and exact runtime bounds

The design reuses the existing P0-L7 request diffs, P0-L10 structural records,
P0-L11 layout candidate, P0-L12 evaluator, and P0-L13 materialization and
safety-certificate identities. C0 is the independently certified materialized
form of A0; C1 is independently certified from A1. The candidate comparison is
the existing A1-to-C1 relationship, not the within-arm C0-to-C1 mutation.

Each projected request is bounded with `generation_limit=1` and wire
`max_tokens=1`. The retained Attempt 006 runtime contract is:

- llama.cpp/OpenAI-compatible chat protocol and the same Q4_0 model family;
- context size `8192`, one parallel slot, and metrics enabled;
- `connect_timeout_ms=1000` and `request_timeout_ms=600000`;
- caller-supplied runtime identity and unknown-preserving configuration fields.

Preparation rejects a missing `fresh_server_for_run` assertion, non-distinct
epoch IDs, duplicate or incomplete arm identity, mismatched model/protocol/
runtime identity, and any projected request without the generation bound.
Preflight returns a versioned readiness record with `network_calls=0` and
explicit A0/A1/C0/C1 step IDs.

## Arm-local durability

`FreshArmRunRecord` carries an explicit evidence state, epoch identity,
transport-attempt and complete-response accounting, normalized-case count,
request fingerprints, raw evidence, normalized result, failure, and provenance.
It can be finalized and persisted with the control arm alone before treatment
is attempted. A partial or failed arm cannot carry a complete normalized
result. A complete result must contain exactly its two certified cases and
matching request/runtime identity.

This is a narrow P0-L6E arm boundary. It does not redesign the existing
generic streaming persistence path or rewrite historical Attempt 001–006
artifacts.

## Semantic identity and offline aggregation

The fresh-arm semantic experiment ID is deterministic and distinct from the
P0-L6B parent ID. It covers the fresh-arm schema/design version, parent paired
experiment, exact control/treatment and A1-to-C1 request diffs, P0-L13
candidate-pair identity, runtime profile, and exact runtime configuration
fingerprint. Caller epoch IDs are excluded so the same scientific design can
be re-armed with new execution epochs without changing semantic identity.
Prompts and request materializations are not perturbed to derive this ID.

After both arm records are independently valid, offline aggregation constructs
three source-aware P0-L8 diagnostics:

1. control mutation `A0` → `A1`;
2. treatment mutation `C0` → `C1`;
3. candidate comparison `A1` → `C1`.

The existing P0-L12 evaluator receives those diagnostics through the corrected
provenance and semantic-envelope paths. There is no composite score and no
automatic direction selection. Positive, equivalent, worse, insufficient,
and identity-mismatch outcomes remain distinguishable. Causality remains
`not_established`; performance and application claims remain disallowed. With
no capability profile supplied, the documented capability gate remains
pending rather than being promoted by live observations.

## Validation and exclusions

The offline test suite covers arm shape and B1 removal, identity and duplicate
epoch rejection, exact generation/runtime bounds, zero-network preflight,
independent fake-transport execution, arm-local persistence, aggregation
identity gates, experimentally observed evidence provenance, noncausal claim
permissions, P0-L13 identity reuse, and deterministic serialization.

This slice does not execute Attempt 007, contact localhost, check listener
readiness, start/stop/restart/configure llama.cpp, perform inference, modify
existing experiment evidence, create an Attempt 007 directory, begin P0-L14,
or integrate ContextBench.
