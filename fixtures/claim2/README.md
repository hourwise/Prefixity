# Claim-2 workload fixture authoring

This directory documents the shared offline fixture contract. The only
permitted workload locations are `fixtures/claim2/cp01/` through
`fixtures/claim2/cp06/`, with case IDs `CP01` through `CP06` in that order.
The six directories contain the materialized cohort. Tests also create a
clearly synthetic temporary case at runtime; it is not a seventh workload.

Each case directory contains one public `case.json`, pinned body
files under local subdirectories such as `bodies/` and `prompt/`, and one
separate evaluation-key JSON file. `case.json` references the key path and its
SHA-256 but never embeds the key. The loader canonicalizes every path, rejects
paths or symlinks that escape the case directory, checks exact UTF-8 bytes
against SHA-256, and rejects a key file that is also exposed as a prompt asset.

The public manifest is the Rust serde type `Claim2CaseManifest` and uses
`prefixity.phase1c.claim2-workload-case`, version `1`. Its top-level shape is:

```json
{
  "schema_id": "prefixity.phase1c.claim2-workload-case",
  "schema_version": 1,
  "case_id": "CP01",
  "kind": "positive",
  "planner_input": { "events": [], "relations": [], "provenance": [] },
  "assets": [],
  "action_menu": [],
  "request_templates": [],
  "evaluation_key_path": "evaluation/key.json",
  "evaluation_key_sha256": "<sha256 of exact key-file bytes>",
  "assistant_output_planning_bytes": 4096,
  "token_proof_inputs": null
}
```

The excerpt shows only the envelope, not a valid or proposed workload. The
manifest structs use `deny_unknown_fields`; write fields using the serde
definitions in `phase1c_claim2_workload.rs`. `token_proof_inputs` is
evaluation/preflight metadata and is optional until independently supported
by token evidence. It is never converted from bytes.

Each asset declares an ASCII `asset_id`, case-relative file path, exact SHA-256,
channel kind, optional event ID, and pinned source revision ID. `prompt_text`
assets contain static prompt text. `context_attachment` and
`native_export_body` assets bind to a `Message` event and must match its
content hash. `environment_receipt` assets bind to `Result` events and their
observation hashes. A tool result stays a `Result`; only `EventBody` parts for
native `Message` occurrences can be pruned. Keep origin IDs in the trace and
receipt metadata. Do not convert a tool result, assistant response, receipt,
or chronological message into a context attachment.

`planner_input` is the complete closed-world structural trace. Event sequence
indexes are contiguous. Event, action, result, and context aliases are unique;
all parents/references and relation endpoints resolve. Parent and reference
edges point to earlier events. `Produces` links agree with action IDs,
originating action IDs, result IDs, and pinned result bytes. The directed
dependency/protocol/producer graph must be acyclic. Every
`SameStateRevision` edge joins an earlier and later native `Message` with the
same exact content hash and explicit same `world_state_revision`; equal bytes
at different revisions do not qualify. Provenance must be planner-visible and
must not use evaluation-only or unsafe-inferred evidence. An invalid trace is
rejected; the loader does not delete edges, relabel events, or fix the policy's
decision.

`action_menu` is the finite public menu for request slots 1 and 2. Each entry
binds an action ID and its Action event to the corresponding Result event,
pinned receipt asset, and deterministic resulting state. There are exactly two
action slots followed by one final-answer slot. The separate evaluation file
has exactly two expected action IDs, result-event IDs, and states, an exact
final-answer JSON value, required event IDs and required relation IDs, and
critical event IDs.
Those expected values never enter `PlannerInput`, `select`, prompt templates,
or the serialized chat request. The model sees only fixture-selected
`prompt_text`, event bodies, this arm's raw assistant outputs, and this arm's
actual deterministic receipts.

Declare three `request_templates`, with slots 1, 2, and 3. Each template starts
with a system message and ends with a user message. Use `Asset` for static
prompt text, `EventBody` for an existing native `Message` body,
`PriorAssistantOutput` in an assistant-role message, and
`PriorEnvironmentReceipt` in a user-role message. Slot 2 must carry request 1's
assistant output and environment receipt. Slot 3 must carry both earlier
assistant outputs and both receipts. Keep each receipt in its own message so
its source result-event ID remains available to the offline evaluator.

`Claim2ArmState` is the only dynamic render/record path. Call
`render_claim2_request(case, &mut arm)` to render that arm's next slot, then
record that slot's exact raw output once. The state machine rejects out-of-order
or repeated renders/outputs, accepts exactly three slots, runs the two
intermediate action/state checks before continuing, stops on the first FAIL,
and marks later slots `NOT_EXECUTED_AFTER_FAILURE`. A transport/integrity stop
can be recorded as `INCONCLUSIVE`. It has no retry, repair, provider, executor,
or live dispatch path. Use `Claim2ArmState::preview_request` or
`precheck_claim2_cohort` only for static no-output projections: prior raw output
and receipt slots remain separately listed as unbound, and every static
projection is marked `dispatchable: false`.

For the frozen CP01–CP06 cohort, the advancing-output protocol correction
requires request-1 and request-2 raw outputs to equal the exact canonical JSON
object generated for that slot's expected action ID. A byte mismatch fails
the slot and retains the original bytes; it is never normalized before carry.
The rule applies independently to BASELINE, NO_OP, and INTERVENTION. See the
[protocol correction record](../../docs/phase-1/PHASE_1C_CLAIM_2_ADVANCING_OUTPUT_PROTOCOL_CORRECTION.md),
[12-transition finite-domain ledger](advancing-output-domain-v1.json), and
[version-2 materialization successor](materialization-report-v2.json). The
older version-1 materialization report remains historical and unchanged.

After all three trajectories, `compare_paired_arms` checks exact BASELINE/NO_OP
request and output equality and INTERVENTION request/output equality before
treatment. A mismatch marks the affected arm `INCONCLUSIVE`; this comparison
is separate from task PASS/FAIL and arm completeness.

The projection modes are fixed. BASELINE returns the input trace unchanged.
NO_OP runs the shared selection path and forces `DO_NOTHING` through the shared
application boundary. INTERVENTION uses the exact selected frozen decision;
the renderer applies it only at request 3. Positive admission requires exactly
one eligible `EXACT_DUPLICATE_PRUNE` target selected by the existing
`controlled-evidence-policy-v1` rule order. Controls must naturally select
`DO_NOTHING`. The module also reports PRUNE, DEFER, and
RELOCATE_CANDIDATE evidence returned by those same predicates, but does not
score DEFER/RELOCATE as context reduction and does not use production planner
recommendations.

For each rendered request, metrics separate fixed UTF-8 bytes, carried raw
assistant bytes, carried environment receipt bytes, included attachment bytes,
omitted attachment bytes, wrapper bytes, JSON escaping bytes, full serialized
request bytes, unbound output slots, and the per-response planning byte estimate.
Static projections report the estimated carried raw-output UTF-8 bytes and list
each possible pinned receipt body size for unbound action slots. The estimates
are not enforced output caps or conservative bounds. The later tokenization
review can count the fully rendered request bodies using the canonical
arm-local histories in the version-2 successor; it must still count the actual
serialized request and its JSON escaping. The full 1024-token envelope remains
the accepted runtime output setting, but is not an estimate of the one
canonical advancing response. Byte measurements remain separate from token
counts. The request
keeps the accepted settings:
model label `lmstudio-community/Qwen3.5-9B-GGUF:Q4_K_M`, `max_tokens: 1024`,
temperature `0`, `top_p: 1`, seed `1`, and `stream: false`; reasoning stays
server-level off and no reasoning or `chat_template_kwargs` field is sent.
Context size is 8192. The pure token guard accepts a supplied measured/bounded
input count only when it is at most 6000 and `input_tokens + 1024 <= 8192`.
Missing counts return `EXACT_TOKENIZATION_REQUIRED`. The materiality evaluator
uses token proof inputs only and returns that same status when they are absent;
it never infers token savings from UTF-8 lengths.

The exact offline request-preparation ledger is reproducible with:

```text
cargo run -p prefixity-controlled-benchmark --example claim2_tokenization_requests --offline --locked -- --write fixtures/claim2/tokenization-request-ledger-v1.json
```

It records the 54 canonical-history requests rendered by `Claim2ArmState`,
including ordered message roles/content, exact future input-token request
bodies, SHA-256 identities, pinned arm-local prior outputs and receipts, and
exact-body hash groups. It confirms the expected 22 unique request bodies in
this fixture state; this is offline identity evidence, not a token count or a
live dispatch plan. Every row is prepared for a future input-token counter call
but is marked non-dispatchable, and no tokenization or server contact occurs.

Before adding any CP fixture, verify the case against the accepted
`PHASE_1C_CLAIM_2_CONTEXT_PRESSURE_WORKLOAD_DESIGN.md`. Preserve the earlier
six-case dependency-safety battery and all historical evidence separately.
Do not edit those fixtures, report identities, benchmark criteria, or frozen
source contracts to make a new case eligible. The first four CP cases must be
natural duplicate context attachments; CP05 must keep equal bytes at different
states; CP06 must keep the same-state duplicate when a consumer or protected
protocol relation makes it ineligible. The positive set has one selected
target per case and no size-based ranking.

## Phase 3 tokenization-client preparation draft

The Phase 2-reviewed request ledger remains unchanged at SHA-256
`739205fb56e4f40bd55245f37d0768b8ca73c891b2f8d28bdb8f284e9f811d45`.
The separate offline contact plan is
[`tokenization-contact-plan-v1.json`](tokenization-contact-plan-v1.json),
SHA-256 `6b7634a3fc7a3d4b76289aea1772c1df686a6641309acc9e307e5f330dfefdf6`.
It fixes 54 logical requests to 22 unique exact-body hashes, 32 logical
duplicates avoided, at most one readiness contact, at most 22 token-count
contacts, and zero inference allowance. Hashes are ordered ascending, and each
row binds its representative exact-body identity and all logical request IDs.

Regenerate the plan without contact:

```text
cargo run -p prefixity-controlled-benchmark --example claim2_tokenization_contact_plan --offline --locked -- --write fixtures/claim2/tokenization-contact-plan-v1.json
```

The dedicated `prefixity-phase1c-claim2-tokenization` executable validates the
entire accepted ledger, plan hash and plan contents, request bodies and fixed
settings before its transport can be created. The static production allowlist
is only `GET http://127.0.0.1:8080/health` and
`POST http://127.0.0.1:8080/v1/chat/completions/input_tokens`; redirects and
proxies are disabled, and no completion/generation API is registered. The
future `preflight` command reads only repository files. A separate future
`execute` mode requires explicit operator-started and runtime-identity
confirmations, has no endpoint or runtime override, and preserves partial
evidence when the single readiness or any input-token request fails. It does
not start a process, retry, or fall back to another tokenizer.

Preparation draft:
[`../../docs/phase-1/PHASE_1C_CLAIM_2_NON_INFERENCE_TOKENIZATION_PREPARATION.md`](../../docs/phase-1/PHASE_1C_CLAIM_2_NON_INFERENCE_TOKENIZATION_PREPARATION.md).
The client, executable, plan and draft have not yet been source-committed,
executable-frozen, bound into a separate experiment identity, offline-certified,
CI-promoted, or accepted for operator use. No readiness check or token count
has been performed. Context-fit, positive-reduction, materiality, control and
CPU-practicality outcomes remain unknown.
