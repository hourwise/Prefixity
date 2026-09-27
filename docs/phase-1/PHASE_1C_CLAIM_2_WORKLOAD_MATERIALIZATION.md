# Phase 1C Claim-2 workload materialization preparation

Starting main: `6b7e21ddd64b2b11e7e264e42d924d1eedc85dee`.
Branch: `codex/phase1c-claim2-workload-materialization`.
Before changes, HEAD, local main, origin/main, and remote main agreed at that
SHA with a clean tracked tree. The accepted design was read in full.

## Decision and boundary

`CLAIM_2_WORKLOAD_MATERIALIZATION_READY_FOR_TOKENIZATION_REVIEW`

The exact CP01-CP06 cohort is materialized offline. This accepts fixture,
mechanism, and deterministic protocol preparation; it does not admit the cohort
for inference or establish token materiality or context fit. All four token
proof inputs remain absent for every positive case: `D3_low`, `B3_high`,
`Dsum_low`, and `sum(B)_high`. Status: `EXACT_TOKENIZATION_REQUIRED`.

No server was started, model loaded, port 8080 contacted, token-count endpoint
called, inference performed, scored pilot prepared, or live gate frozen.
The old h001/h004/h007/h009/h006/h010 dependency-safety fixtures, consumed
identities, and historical evidence are unchanged.

## Implementation and reproducible evidence

- [Schema/authoring contract](../../fixtures/claim2/README.md), version 1:
  `prefixity.phase1c.claim2-workload-case`.
- [Shared adapter](../../crates/prefixity-controlled-benchmark/src/phase1c_claim2_workload.rs):
  pinned loader, validation, arm-local state machine, common renderer, precheck,
  pure token guard, and deterministic evaluator. No executor/network entry.
- `phase1b9.rs`: narrow internal policy/application/trace access and candidate
  evidence from the same existing predicates. Rule order, eligibility,
  latest-sequence PRUNE selection, first-match DEFER/RELOCATE, and cardinality
  remain unchanged. `lib.rs` exports the offline API.
- Six case directories contain manifests, native bodies, receipts, task and
  action/final prompts, source/state snapshots, provenance, and separate hidden
  keys. `.gitattributes` preserves exact fixture bytes across checkouts.
- Three case integration suites, common unit tests, and
  `claim2_cohort_evidence.rs` validate the cohort and committed evidence.
- [Machine-readable 54-slot ledger](../../fixtures/claim2/materialization-report.json)
  includes manifest/trace/key hashes, actual policy candidate evidence, removed
  events/relations, each message's byte length/hash, every request's exact
  serialized static skeleton/hash, component byte metrics, and unbound slots.

Regenerate the ledger offline with:

```text
cargo run -p prefixity-controlled-benchmark --example claim2_materialization --offline --locked -- --write fixtures/claim2/materialization-report.json
```

The candidate commit is the implementation provenance boundary. CI and the
promotion SHA are reported with that commit in the supervisor closeout; this
pre-commit record does not claim future CI success or freeze a live identity.

## Case evidence

| Case | Pinned task and natural workflow | Actual policy | Legal body omission |
| --- | --- | --- | ---: |
| CP01 | Python retry/config diagnosis: configured four attempts, loop executes three. Inspect implementation, reproduce regression, diagnose exact fix. Normal verification export repeats unchanged `cp01-r41`. | `EXACT_DUPLICATE_PRUNE`, `e-repeat` | 2,954 bytes |
| CP02 | Investigate the 21-source-file `quarry-ledger-r1` Rust snapshot. Generate symbol inventory, follow `Library::find_record` through index/store/cache, carry four direct reads. Final packet reattaches the same generated inventory. | `EXACT_DUPLICATE_PRUNE`, `e-inventory-copy` | 3,366 bytes |
| CP03 | Reconcile seven Rust packages, 12 dependency edges, five protocol consumers, workspace Rust 1.78, pinned toolchain 1.82.0, and CI versions. Reemit the unchanged generated dependency inventory. | `EXACT_DUPLICATE_PRUNE`, `e-inventory-copy` | 2,679 bytes |
| CP04 | Verify candidate `quick-route-v42` fails tenant/account isolation, restore stable R41, then reexport its source/config packet. | `EXACT_DUPLICATE_PRUNE`, `e-recovery-copy` | 3,342 bytes |
| CP05 | Rebuild route index: generation 71 becomes 72 at R17 -> R18; catalog object/body is unchanged. Compare both occurrence/state identities. | `DO_NOTHING`, no target | 0 bytes (4,859-byte candidate-looking body) |
| CP06 | Same policy-index body/state S84 reexported for audit AUD-2081. Receipt names the later occurrence and explicitly depends on it. Both occurrence IDs and receipt are required. | `DO_NOTHING`, no target | 0 bytes (5,282-byte candidate-looking body) |

Case provenance and inventories: [CP01](../../fixtures/claim2/cp01/README.md),
[CP02](../../fixtures/claim2/cp02/README.md),
[CP03](../../fixtures/claim2/cp03/README.md),
[CP04](../../fixtures/claim2/cp04/README.md),
[CP05](../../fixtures/claim2/cp05/workflow.md), and
[CP06](../../fixtures/claim2/cp06/workflow.md).

Each positive has exactly one candidate under the actual frozen predicates,
with equal body bytes and explicit same world-state revision, an earlier
native Message, no consumer, and no protected relation. No DEFER or relocation
candidate exists. Only the selected later Message and its same-state relation
are removed; original required context and all chronological results survive.
CP05 has no same-state eligibility; CP06 has an explicit real protected
consumer. Both controls have empty candidate inventories for all three rules.
No target was selected using size or expected savings.

Closed-world validation checks every endpoint and alias, prior parent/reference
ordering, acyclic dependency/protocol relations, producer/result identity,
revision/body hashes, and action-to-receipt binding. Case tests independently
verify projected endpoint closure and required/critical preservation; synthetic
valid answers pass all three arms. These tests establish machinery behavior,
not model competence or model task outcomes.

## Every request's byte accounting

All values below are exact UTF-8/static serialization bytes except the labelled
planning estimate. `B/N/I` means the corresponding row applies to all arms;
`B/N` means BASELINE and NO_OP. The 54 separate rows and per-message details
are in the linked ledger. Requests 2/3 have empty, explicitly unbound prior
assistant/receipt placeholders: they are skeletons, not exact future requests.
Request 1 is fully specified, but every preview is non-dispatchable.

`F` = fixed prompt content; `A` = included native attachment content;
`W` = JSON structural wrapper; `E` = JSON escaping overhead;
`J = F + A + W + E` = exact skeleton JSON bytes.
Carried assistant and receipt byte counts are zero in these static skeletons.
`P` is the estimated raw prior-output planning bytes, **not an enforced cap or
conservative bound**. `Rmax` sums the largest pinned raw receipt body per
unbound prior action. Receipt alternatives are enumerated in the ledger;
serialization escaping for future carried content is additional.

| Case | Slot | Arms | F | A | W | E | J | Omitted body | P estimate | Rmax |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| CP01 | 1 | B/N/I | 619 | 3,343 | 194 | 117 | 4,273 | 0 | 0 | 0 |
| CP01 | 2 | B/N/I | 846 | 3,343 | 286 | 119 | 4,594 | 0 | 2,048 | 312 |
| CP01 | 3 | B/N | 1,153 | 6,297 | 378 | 228 | 8,056 | 0 | 4,096 | 928 |
| CP01 | 3 | I | 1,153 | 3,343 | 378 | 119 | 4,993 | 2,954 | 4,096 | 928 |
| CP02 | 1 | B/N/I | 698 | 469 | 194 | 14 | 1,375 | 0 | 0 | 0 |
| CP02 | 2 | B/N/I | 984 | 3,835 | 315 | 97 | 5,231 | 0 | 4,096 | 367 |
| CP02 | 3 | B/N | 1,391 | 12,727 | 436 | 364 | 14,918 | 0 | 8,192 | 748 |
| CP02 | 3 | I | 1,391 | 9,361 | 436 | 293 | 11,481 | 3,366 | 8,192 | 748 |
| CP03 | 1 | B/N/I | 741 | 538 | 194 | 14 | 1,487 | 0 | 0 | 0 |
| CP03 | 2 | B/N/I | 1,099 | 3,217 | 315 | 71 | 4,702 | 0 | 4,096 | 296 |
| CP03 | 3 | B/N | 1,662 | 5,896 | 407 | 117 | 8,082 | 0 | 8,192 | 859 |
| CP03 | 3 | I | 1,662 | 3,217 | 407 | 72 | 5,358 | 2,679 | 8,192 | 859 |
| CP04 | 1 | B/N/I | 672 | 3,751 | 194 | 157 | 4,774 | 0 | 0 | 0 |
| CP04 | 2 | B/N/I | 948 | 3,751 | 286 | 159 | 5,144 | 0 | 2,048 | 425 |
| CP04 | 3 | B/N | 1,304 | 7,093 | 378 | 308 | 9,083 | 0 | 4,096 | 900 |
| CP04 | 3 | I | 1,304 | 3,751 | 378 | 159 | 5,592 | 3,342 | 4,096 | 900 |
| CP05 | 1 | B/N/I | 974 | 4,859 | 194 | 632 | 6,659 | 0 | 0 | 0 |
| CP05 | 2 | B/N/I | 1,221 | 4,859 | 286 | 641 | 7,007 | 0 | 4,096 | 408 |
| CP05 | 3 | B/N/I | 1,564 | 9,718 | 407 | 1,263 | 12,952 | 0 | 8,192 | 1,087 |
| CP06 | 1 | B/N/I | 918 | 5,282 | 194 | 630 | 7,024 | 0 | 0 | 0 |
| CP06 | 2 | B/N/I | 1,210 | 5,282 | 286 | 639 | 7,417 | 0 | 4,096 | 333 |
| CP06 | 3 | B/N/I | 1,440 | 10,564 | 407 | 1,259 | 13,670 | 0 | 8,192 | 911 |

Body omissions and serialized request deltas differ because JSON escaping is
also removed. Final serialized deltas are CP01 3,063; CP02 3,437; CP03 2,724;
CP04 3,491 bytes. None is a token-saving measurement.

The earlier worker estimates were 2,048 raw bytes per response for CP01/CP04
and 4,096 for the others. Supervisor review removed an unintended enforcement
check: these are planning scenarios only and cannot fail a valid response or
lower the accepted 1,024-token ceiling. Full raw output is retained without
normalization/truncation. The actual conservative byte bound remains unknown.

## Deterministic trajectories, NO_OP, and evaluation

Exactly three requests and two deterministic environment steps exist in each
arm. Menus are finite; every valid action maps to a pinned result and state.
Each next prompt preserves the entire earlier prompt unchanged and appends
that arm's raw assistant output, actual receipt, and new context. CP02/CP03
initial template placement was corrected before any execution to satisfy this
chronology; no source body, relation, answer fact, or policy outcome changed.

BASELINE preserves natural context. NO_OP traverses the same policy/application
pipeline with forced `DO_NOTHING`; INTERVENTION applies the actual frozen
selection immediately before slot 3. The common renderer preserves message
order, roles, wrappers, and serialization. Static projections prove equal
BASELINE/NO_OP hashes and equal pre-treatment hashes. Future independently
produced outputs are never replayed across arms. Paired checks require complete
three-request trajectories, exact BASELINE/NO_OP outputs and requests, and
matched INTERVENTION pre-treatment history. Divergence is procedural
`INCONCLUSIVE` while any observed model failure remains separately recorded.

Intermediate evaluation rejects malformed schema, extra fields, unknown or
incorrect action identity, and incorrect transition. Final evaluation uses
exact hidden facts, fixed answer shape/types, actual rendered required-event
availability/recall, required relations, critical-event preservation, and
three-output/two-receipt completeness. Availability recall is a separate
preservation metric; exact final facts test answer correctness. Keys never
enter the policy or model-visible renderer. A same-shaped wrong answer is a
complete `FAIL`; malformed/wrong-shaped output is an incomplete `FAIL`.
Reevaluation preserves those terminal distinctions.

`PASS`, `FAIL`, `INCONCLUSIVE`, and `NOT_EXECUTED_AFTER_FAILURE` are explicit.
Skipped slots remain in the denominator. No retry, repair prompt, output
copying, normalization, or hidden fourth turn is provided. The offline state
machine accepts raw responses for tests and later integration; it does not
perform transport or pretend to validate future provider finish metadata.

## Token fit and CPU practicality: unresolved estimates

All cases require `NON_INFERENCE_CONTEXT_TOKENIZATION_PASS`. No conservative
proof presently establishes the 800-token / 20% final and 8% cumulative
positive reductions. CP03's 2,679-byte body in particular must not be presumed
to meet 800 tokens. Do not pad or change thresholds if a case fails later.
The pure guard requires authoritative/bounded input <= 6,000 and
`input_tokens + 1024 <= 8192`; approximately 5,500 remains the design target.
No byte conversion establishes admission. Any future bound failure returns
for offline review without truncation, a smaller output ceiling, or inference.

Measured historical planning rates remain approximately 9-10 prompt tokens/s
and 1.2-2 decode tokens/s. Summed 54 static skeletons contain 366,629 JSON bytes;
adding the raw planning-output scenarios and maximum raw receipt sizes gives
573,671 bytes before unknown dynamic JSON escaping. At the design's rough
3-5 bytes/token conversion, that scenario implies approximately 114,734-191,224
input tokens and 3.2-5.9 hours of cold prefill. These are estimates, not bounds
or predictions from a tokenizer; HTTP JSON is not itself the chat template.

A separate short-answer scenario (80 generated tokens per response, equivalent
80-token carried history per prior slot) estimates about 82,190-134,104 input
tokens including receipt bytes, or approximately 2.9-5.1 hours for prefill and
decode. At the materialized raw-output planning sizes, the corresponding
36,864-61,440 generated-token estimate instead adds about 5.1-14.2 decode
hours: approximately 8.3-20.1 hours total. If all 54 responses reach 1,024
tokens, decode alone is 7.7-12.8 hours. Startup, tokenizer prechecks, I/O, and
cache effects are excluded from these scenarios.

`MODEL_CAPABILITY_ACCEPTED` remains the historical instrument finding.
`CPU_RUNTIME_PRACTICALITY_REVIEW_REQUIRED` is a separate planning concern:
the larger-output scenarios exceed the accepted eight-hour review boundary.
This does not change the model or assert a runtime failure. The later review
must replace size estimates with token evidence before any live preparation.

## Validation and remaining work

Local validation passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings
cargo test --workspace --offline --locked
cargo +1.86.0 check --workspace --offline --locked
```

The complete workspace run passed 661 tests. A final additional consumed-gate
regression test was then added; the complete cohort evidence suite passed 2/2,
and final formatting and Clippy passed. The case suites passed 7/7, 6/6, and
6/6; the shared module passed 13/13. All six manifests' pinned assets and
hidden-key hashes verified; 159 cohort text files passed exact UTF-8/LF checks.
Report/README links and working/staged diff whitespace were checked before
publication. No benchmark outcome criteria or historical evidence was changed.
The reproducible offline ledger comparison includes all 54 projections.

The first full workspace run exposed 20 failures from the old h001 checker
reading evolving `phase1b9.rs` as if it were the manifest's historical commit.
The fix preserves an exact 58,755-byte Git blob at
[historical source](../../fixtures/provenance/phase1b9-748e4673.rs.txt), from
`748e4673e8454d2ac3e27cefabee9259992038aa`, with the unchanged SHA-256
`2f1dae56606815034530b9a7114170eea08b9269eed57b18a4fb5d9747830a33`.
`phase1c_h001.rs` now verifies the historical commit/locator and source hash
against that archive. The archive is not compiled or used as an alternate
policy. No historical manifest, hash, consumed identity, or evidence changed.
A dedicated integration check proves that the consumed local-9B preflight
still rejects current runtime source changes before any runtime inspection or
contact. This is a source-provenance repair, not a changed h001 evaluator. The initial
Clippy pass found two high-arity synthetic test helpers and one boolean
assertion style issue; those were corrected without changing acceptance tests.

The workers additionally verified fixture source behavior: CP01's pinned
retry regression intentionally reproduces 3 attempts instead of 4; its
configuration checks pass. CP04 invariant/recovery checks pass. CP02's local
Rust snapshot tests pass offline/locked; CP03's do too using installed Rust
1.85.0, not its unavailable pinned 1.82.0. That distinction does not change
its pinned metadata. CP05/CP06 state/body/dependency proofs pass offline.

Next substantive task is a separately authorized
`NON_INFERENCE_CONTEXT_TOKENIZATION_PASS`, beginning with tokenization review
of the six frozen fixtures and full 1,024-token output envelopes. No startup,
tokenization, or scored work is authorized by this materialization record.

CLAIM_2_WORKLOAD_MATERIALIZATION_READY_FOR_TOKENIZATION_REVIEW
