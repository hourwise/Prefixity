# Phase 1C Claim-2 non-inference tokenization domain review

Primary state: `TOKENIZATION_PASS_INCONCLUSIVE`.
Phase 1 stop reason: `TOKENIZATION_BOUND_NOT_FINITE`.

The frozen schema/transition API accepts an unbounded raw assistant language
at both advancing slots in all six cases. It preserves those bytes in later
requests. The required finite, exhaustive schema-level enumeration proof is
therefore absent. The user's Phase 1 stop rule applies: phases 2-4 were not
started. This is not an observed token-admission or context-fit failure.

## Baseline and scope

Before modification, HEAD, local main, origin/main and remote main matched
`4a25e4ca31716b70bec4af6dd5472afccdcae435`; the tracked working tree was clean. The accepted workload
design, materialization review, fixture README and complete materialization
ledger were read. One Luna worker reviewed the domain, followed by supervisor
review and a bounded follow-up for the parser's positional sequence form.

The materialization ledger remains unchanged: SHA-256
`3d9571ddd4e17096b55971040531aac6197b6b9b7bc3d01afefd8992c28f6c65`. No fixture, schema, evaluator,
planner, accepted output ceiling, threshold or historical evidence changed.
All offline diagnostic strings are parser witnesses, not model outputs or
histories selected for tokenization. No BASELINE, NO_OP or INTERVENTION model
trajectory was executed.

## Exact language and constructive proof

For case c and advancing slot s, define L(c,s) as all finite Rust strings x
for which `serde_json::from_str::<Claim2ActionOutput>(x)` succeeds and returns
the expected action ID in the pinned evaluation key, with its action-menu
transition matching the pinned expected state. This parser preimage plus
the intermediate checks characterizes every raw body capable of advancing;
the single canonical JSON spelling is not an exhaustive representation.

The shared Rust response type has one required `action_id: String` with
`deny_unknown_fields`. The implementation also accepts a one-element JSON
sequence such as `["inspect_retry_implementation"]`, because the derived
struct deserializer supports positional sequences. Accepted spellings include
object and sequence forms, JSON escapes in strings, and JSON whitespace
outside strings. Extra fields, duplicate `action_id` fields, unknown actions,
and wrong in-menu actions fail; their raw text can remain in failure evidence
but no subsequent model request is rendered.

For any expected ID A, the family

```text
x_n = '{"action_id":"A"}' + n ASCII spaces, for every integer n >= 0
```

contains distinct raw bodies with the same successful parse and transition.
There is no response byte cap or normalization in `record_output`; planning
byte estimates are expressly not caps. The renderer appends the raw string
unchanged. This proves countably infinitely many schema/API-accepted raw
histories for request 2. Request 3 carries a pair from L(c,1) x L(c,2), also
countably infinite. Fixed receipts do not eliminate this variation. There is
only one advancing semantic action sequence per case. Request 3's final
answer cannot create a fourth request and adds no further carried slot.

Source anchors in the unchanged
[`phase1c_claim2_workload.rs`](../../crates/prefixity-controlled-benchmark/src/phase1c_claim2_workload.rs):
`record_output` lines 334-400; response type lines 610-614; raw carry lines
884-891; existing non-cap regression test lines 2852-2866. The frozen prompt's
preferred object form does not restrict what the actual evaluator accepts.

### Runtime qualification

The claim above concerns the schema/API language, not unlimited generation
by the accepted model. A finite vocabulary and the accepted 1024-token output
ceiling imply a finite set of runtime token sequences. The adapter does not
check that a supplied string is a decoding of such a sequence. This review
neither enumerated that runtime set nor proved an exact tokenizer-backed
bound or exhaustive filtering procedure. Consequently it does not assert
that arbitrarily long whitespace is reachable under the accepted runtime.
No canonical response, planning-byte limit or approximate token bound was
substituted. A different proof approach would require separate review; none
was implemented during this task.

## All transitions

The shared language L(c,s) applies to every row. Both listed action IDs are
public menu options; only the advancing ID satisfies the intermediate gate.
The other ID terminates as FAIL without producing a later carried receipt.

| Case | Transition | Advancing ID | Other menu ID (FAIL) | Receipt event | State after |
| --- | --- | --- | --- | --- | --- |
| CP01 | 1 -> 2 | `inspect_retry_implementation` | `inspect_dispatch_defaults` | `e-result1` | `source_loaded` |
| CP01 | 2 -> 3 | `run_retry_regression` | `check_timeout_units` | `e-result2` | `retry_regression_failed_on_r41` |
| CP02 | 1 -> 2 | `build_symbol_inventory` | `skip_inventory` | `e-result-inventory` | `inventory_ready` |
| CP02 | 2 -> 3 | `inspect_resolution_chain` | `inspect_cli_surface` | `e-result-callsite` | `callsites_opened` |
| CP03 | 1 -> 2 | `collect_dependency_inventory` | `skip_inventory` | `e-result-inventory` | `inventory_ready` |
| CP03 | 2 -> 3 | `verify_snapshot_and_constraints` | `verify_toolchain_only` | `e-result-verify` | `snapshot_verified` |
| CP04 | 1 -> 2 | `run_candidate_guardrail_probe` | `inspect_branch_topology` | `e-result1` | `candidate_failed_origin_restored` |
| CP04 | 2 -> 3 | `verify_restored_state` | `read_failed_candidate_diff` | `e-result2` | `restored_origin_verified` |
| CP05 | 1 -> 2 | `inspect_initial_catalog` | `read_health_only` | `e-inspect-result` | `route_registry_r17_inspected` |
| CP05 | 2 -> 3 | `rebuild_route_index_and_export` | `read_health_summary` | `e-reindex-result` | `route_registry_r18_reindexed` |
| CP06 | 1 -> 2 | `inspect_policy_index` | `read_policy_summary` | `e-policy-inspect-result` | `policy_index_s84_inspected` |
| CP06 | 2 -> 3 | `verify_audit_receipt` | `check_export_signature_only` | `e-audit-result` | `audit_receipt_AUD_2081_verified` |

The advancing receipt files and their exact hashes are:

- CP01 slot 1: [`r-result1`](../../fixtures/claim2/cp01/environment/receipt1.txt), SHA-256 `3eaddfcb86e49ad105f6d11f3a900affae2e5ad3d0a10c54f4078ccff86484a8`.
- CP01 slot 2: [`r-result2`](../../fixtures/claim2/cp01/environment/receipt2.txt), SHA-256 `aa90d37f70fafb34c9409a7e025efe4636afb5ba1d3934d777e8aa88078bd7f2`.
- CP02 slot 1: [`r-inventory`](../../fixtures/claim2/cp02/bodies/receipt1.txt), SHA-256 `e62dea11ce37d189ac0aaa84cb6a1510ebc3845294c10875952ad8c755ee3afa`.
- CP02 slot 2: [`r-callsite`](../../fixtures/claim2/cp02/bodies/receipt2.txt), SHA-256 `4e81fd820ed9740b04bea9111f95317130ceb193b16efb6ba7d8089cf4481777`.
- CP03 slot 1: [`r-inventory`](../../fixtures/claim2/cp03/bodies/receipt1.txt), SHA-256 `df82948814e3851a98709fb122964ba7f3274becb87c2afb0c6c8909411123e9`.
- CP03 slot 2: [`r-verify`](../../fixtures/claim2/cp03/bodies/receipt2.txt), SHA-256 `d832c457e33ba7cc109116f2510b8980b08b09ca3431ad4add0a426d9340ab67`.
- CP04 slot 1: [`r-result1`](../../fixtures/claim2/cp04/environment/receipt1.txt), SHA-256 `6ab3eb54f41ecf9ff1ba4a38b147a16c91f6425b073bd98357c7a98252546b69`.
- CP04 slot 2: [`r-result2`](../../fixtures/claim2/cp04/environment/receipt2.txt), SHA-256 `e68f339375c2fe0c36583226e3e35db64e63407574ac7ec7d44d89b0d34ec874`.
- CP05 slot 1: [`r-route-inspect-017`](../../fixtures/claim2/cp05/bodies/receipt-inspect.txt), SHA-256 `2b424ac040d25e4eed01bff4ab4407b0ddabd95435811f89ef7dd1e8002dd53a`.
- CP05 slot 2: [`r-route-index-064`](../../fixtures/claim2/cp05/bodies/receipt-reindex.txt), SHA-256 `588976b08d47a5b9d899502a41a85a6d231c7aea002ef85354bb0be76010f747`.
- CP06 slot 1: [`r-policy-inspect-084`](../../fixtures/claim2/cp06/bodies/receipt-policy-inspect.txt), SHA-256 `c8500e2bdaea68ec5275f0c8f55e1de332fd4433734b88d1b4135afcecc89694`.
- CP06 slot 2: [`r-audit-AUD-2081`](../../fixtures/claim2/cp06/bodies/receipt-audit-2081.txt), SHA-256 `f1711d4e3232120b0a53306bba96cc3e601ff04d456c6690908b90bd46f476eb`.

## Required request results at the stop boundary

Each row applies separately to BASELINE, NO_OP and INTERVENTION. Counts here
refer to distinct accepted raw histories, not semantic actions. Request 1
has one fully specified history per case/arm. No request was tokenized.

| Case | Request | Schema histories / runtime qualification | Min input tokens | Max input tokens | Min/max request hash | Context hard stop |
| --- | --- | --- | --- | --- | --- | --- |
| CP01 | 1 | 1 | null | null | null | NOT_EVALUATED |
| CP01 | 2 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP01 | 3 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP02 | 1 | 1 | null | null | null | NOT_EVALUATED |
| CP02 | 2 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP02 | 3 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP03 | 1 | 1 | null | null | null | NOT_EVALUATED |
| CP03 | 2 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP03 | 3 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP04 | 1 | 1 | null | null | null | NOT_EVALUATED |
| CP04 | 2 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP04 | 3 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP05 | 1 | 1 | null | null | null | NOT_EVALUATED |
| CP05 | 2 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP05 | 3 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP06 | 1 | 1 | null | null | null | NOT_EVALUATED |
| CP06 | 2 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |
| CP06 | 3 | countably infinite / runtime count unknown | null | null | null | NOT_EVALUATED |

For CP01-CP04, `D3_low`, `B3_high`, `D3_low / B3_high`, `Dsum_low`,
`sum(B)_high`, `Dsum_low / sum(B)_high`, minimum direct `R3(h)` and minimum
direct `Rsum(h)` are all null. The 800-token, 20%, 8%, 6000-input and
input-plus-1024 context gates are NOT_EVALUATED. No measured failure is inferred.

For CP05/CP06, the accepted policy remains DO_NOTHING with legal reduction 0.
Exhaustive reachable-request body/message equality, token equality and context
fit were not evaluated in this stopped pass. The prior materialization's
static byte evidence remains separate; it is not exhaustive token evidence.
The same limitation applies to full-history BASELINE/NO_OP equality for all
six cases. CPU arithmetic was not recomputed because exact input totals are
absent; the previous practicality review and model-capability acceptance
remain unchanged.

## Evidence and validation

The [machine-readable review](PHASE_1C_CLAIM_2_TOKENIZATION_DOMAIN_REVIEW.json) is a Phase 1 review record, not a
token-count ledger. Its [seal](PHASE_1C_CLAIM_2_TOKENIZATION_DOMAIN_REVIEW.sha256) is SHA-256 of canonical JSON
(recursively sorted keys, compact separators, UTF-8, no terminal newline):
`fa6673e283adaeea1616f5a7387932f209b93eedc76a79736425944b26a8620f`. There are zero tokenization records. Missing measurements remain null.

| Contact category | Count |
| --- | --- |
| Readiness contacts | 0 |
| Unique token-count contacts | 0 |
| Duplicate token-count requests avoided | 0 |
| Inference requests | 0 |

No server was started, restarted, reconfigured or terminated. No endpoint
was contacted; no tokenization client or frozen request enumeration exists.
The operator-server readiness boundary was not reached.

The worker and supervisor ran the offline diagnostic below successfully. It
demonstrates exact carry for whitespace/escape and positional sequence forms
in every case and four representative failure classes in CP01. These finite
witnesses corroborate the source proof; they are not an exhaustive enumeration.

Reproduction: place the following manifest and source at
`target/tokenization_domain_review/Cargo.toml` and `src/main.rs`, respectively,
then run from the repository root:

```powershell
cargo run --manifest-path target/tokenization_domain_review/Cargo.toml --offline --locked
```

On a fresh diagnostic directory, first resolve its local path dependencies
with `cargo generate-lockfile --manifest-path target/tokenization_domain_review/Cargo.toml --offline`.
The root workspace dependencies and lockfile are unchanged.

Source-hash detail: `offline_diagnostic.source_sha256` identifies the original
scratch source file, encoded as UTF-8 with a BOM and a terminal LF. The Rust
snippet below omits the invisible BOM. To reproduce that exact file hash,
write the snippet with UTF-8 BOM encoding and one terminal LF. BOM-free source
has the same diagnostic behavior but a different file hash.

```toml
[package]
name = "claim2-domain-probe"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
prefixity-controlled-benchmark = { path = "../../crates/prefixity-controlled-benchmark" }
```

```rust
// OFFLINE DIAGNOSTIC ONLY — synthetic strings, not model outputs or tokenization placeholders.
use prefixity_controlled_benchmark::{
    load_claim2_case, render_claim2_request, Claim2ArmState, Claim2ProjectionMode,
    Claim2SlotStatus,
};
use std::path::PathBuf;

fn unicode_escape(s: &str) -> String {
    s.chars().map(|c| format!("\\u{:04x}", c as u32)).collect()
}
fn spaced_action(id: &str, n: usize) -> String {
    format!(" \t{{ \"action_id\" : \"{id}\" }}\r\n{}", " ".repeat(n))
}
fn escaped_action(id: &str, n: usize) -> String {
    format!("{{\"action_id\":\"{}\"}}{}", unicode_escape(id), "\n".repeat(n))
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/claim2");
    let cases = [
        ("cp01", ["inspect_retry_implementation", "run_retry_regression"]),
        ("cp02", ["build_symbol_inventory", "inspect_resolution_chain"]),
        ("cp03", ["collect_dependency_inventory", "verify_snapshot_and_constraints"]),
        ("cp04", ["run_candidate_guardrail_probe", "verify_restored_state"]),
        ("cp05", ["inspect_initial_catalog", "rebuild_route_index_and_export"]),
        ("cp06", ["inspect_policy_index", "verify_audit_receipt"]),
    ];
    println!("OFFLINE_DIAGNOSTIC_ONLY: synthetic raw strings; zero model/tokenization calls");
    for (case_id, ids) in cases {
        let case = load_claim2_case(&root.join(case_id).join("case.json")).unwrap();
        let mut arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        let request1 = render_claim2_request(&case, &mut arm).unwrap();
        assert_eq!(request1.request_slot, 1);
        let raw1 = spaced_action(ids[0], 64);
        assert!(arm.record_output(&case, raw1.clone()).unwrap().is_none());
        let (receipt1_event, receipt1_state) = {
            let receipt = arm.environment_receipts().last().unwrap();
            (receipt.source_event_id.clone(), receipt.state_after.clone())
        };

        let request2 = render_claim2_request(&case, &mut arm).unwrap();
        let carries1 = request2.messages.iter().any(|m| m.role == "assistant" && m.content == raw1);
        assert!(carries1, "{case_id}: request 2 must carry raw request-1 bytes");
        let raw2 = escaped_action(ids[1], 32);
        assert!(arm.record_output(&case, raw2.clone()).unwrap().is_none());
        let (receipt2_event, receipt2_state) = {
            let receipt = arm.environment_receipts().last().unwrap();
            (receipt.source_event_id.clone(), receipt.state_after.clone())
        };

        let request3 = render_claim2_request(&case, &mut arm).unwrap();
        let assistant_bodies: Vec<_> = request3.messages.iter()
            .filter(|m| m.role == "assistant")
            .map(|m| m.content.as_str()).collect();
        assert_eq!(assistant_bodies, vec![raw1.as_str(), raw2.as_str()]);
        println!(
            "{case_id}: PASS request2/3 advancing; raw1_bytes={} receipt1_event={} state={}; raw2_bytes={} receipt2_event={} state={}; request3_assistant_bodies=exact",
            raw1.len(), receipt1_event, receipt1_state,
            raw2.len(), receipt2_event, receipt2_state,
        );

        // Serde's struct visitor may also accept positional sequences. Probe
        // that raw form independently at both advancing action slots.
        let mut sequence_arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        render_claim2_request(&case, &mut sequence_arm).unwrap();
        let sequence1 = format!("[\"{}\"]", ids[0]);
        assert!(sequence_arm.record_output(&case, sequence1.clone()).unwrap().is_none());
        let request2 = render_claim2_request(&case, &mut sequence_arm).unwrap();
        assert!(request2.messages.iter().any(|m| m.role == "assistant" && m.content == sequence1));
        let sequence2 = format!("[\"{}\"]", ids[1]);
        assert!(sequence_arm.record_output(&case, sequence2.clone()).unwrap().is_none());
        let request3 = render_claim2_request(&case, &mut sequence_arm).unwrap();
        let sequence_bodies: Vec<_> = request3.messages.iter()
            .filter(|m| m.role == "assistant")
            .map(|m| m.content.as_str()).collect();
        assert_eq!(sequence_bodies, vec![sequence1.as_str(), sequence2.as_str()]);
        println!("{case_id}: PASS positional JSON arrays advance at slots 1 and 2; raw arrays carried exactly");
    }

    // Distinguish parser-valid, in-menu alternatives from advancing actions;
    // malformed shapes fail at the same intermediate gate.
    let case = load_claim2_case(&root.join("cp01").join("case.json")).unwrap();
    for (label, raw) in [
        ("in_menu_but_wrong", r#"{"action_id":"inspect_dispatch_defaults"}"#),
        ("unknown_action", r#"{"action_id":"not_in_menu"}"#),
        ("unknown_field", r#"{"action_id":"inspect_retry_implementation","extra":"x"}"#),
        ("duplicate_field", r#"{"action_id":"inspect_retry_implementation","action_id":"inspect_retry_implementation"}"#),
    ] {
        let mut arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        render_claim2_request(&case, &mut arm).unwrap();
        let evaluation = arm.record_output(&case, raw.to_string()).unwrap().unwrap();
        assert_eq!(evaluation.status, Claim2SlotStatus::Fail);
        assert_eq!(arm.slots()[1].status, Claim2SlotStatus::NotExecutedAfterFailure);
        println!("cp01 {label}: rejected as advancing; request 2 NOT_EXECUTED_AFTER_FAILURE");
    }
}
```

Observed diagnostic stdout:

```text
OFFLINE_DIAGNOSTIC_ONLY: synthetic raw strings; zero model/tokenization calls
cp01: PASS request2/3 advancing; raw1_bytes=116 receipt1_event=e-result1 state=source_loaded; raw2_bytes=168 receipt2_event=e-result2 state=retry_regression_failed_on_r41; request3_assistant_bodies=exact
cp01: PASS positional JSON arrays advance at slots 1 and 2; raw arrays carried exactly
cp02: PASS request2/3 advancing; raw1_bytes=110 receipt1_event=e-result-inventory state=inventory_ready; raw2_bytes=192 receipt2_event=e-result-callsite state=callsites_opened; request3_assistant_bodies=exact
cp02: PASS positional JSON arrays advance at slots 1 and 2; raw arrays carried exactly
cp03: PASS request2/3 advancing; raw1_bytes=116 receipt1_event=e-result-inventory state=inventory_ready; raw2_bytes=234 receipt2_event=e-result-verify state=snapshot_verified; request3_assistant_bodies=exact
cp03: PASS positional JSON arrays advance at slots 1 and 2; raw arrays carried exactly
cp04: PASS request2/3 advancing; raw1_bytes=117 receipt1_event=e-result1 state=candidate_failed_origin_restored; raw2_bytes=174 receipt2_event=e-result2 state=restored_origin_verified; request3_assistant_bodies=exact
cp04: PASS positional JSON arrays advance at slots 1 and 2; raw arrays carried exactly
cp05: PASS request2/3 advancing; raw1_bytes=111 receipt1_event=e-inspect-result state=route_registry_r17_inspected; raw2_bytes=228 receipt2_event=e-reindex-result state=route_registry_r18_reindexed; request3_assistant_bodies=exact
cp05: PASS positional JSON arrays advance at slots 1 and 2; raw arrays carried exactly
cp06: PASS request2/3 advancing; raw1_bytes=108 receipt1_event=e-policy-inspect-result state=policy_index_s84_inspected; raw2_bytes=168 receipt2_event=e-audit-result state=audit_receipt_AUD_2081_verified; request3_assistant_bodies=exact
cp06: PASS positional JSON arrays advance at slots 1 and 2; raw arrays carried exactly
cp01 in_menu_but_wrong: rejected as advancing; request 2 NOT_EXECUTED_AFTER_FAILURE
cp01 unknown_action: rejected as advancing; request 2 NOT_EXECUTED_AFTER_FAILURE
cp01 unknown_field: rejected as advancing; request 2 NOT_EXECUTED_AFTER_FAILURE
cp01 duplicate_field: rejected as advancing; request 2 NOT_EXECUTED_AFTER_FAILURE
```

Remaining issue: the frozen response contract does not provide the finite
raw-history domain required by the requested enumeration procedure. No
remediation, later design task, scored pilot preparation or live work began.

Additional validation: the research-state consistency suite passed 12/12;
review JSON/seal, all twelve receipt hashes, local Markdown links and diff
whitespace were checked. Only this review, its JSON/seal, INDEX and ACTIVE
are changed. At the initial Phase 1 closeout the review was local and
uncommitted; no publication or promotion had been performed.

Part A publication review: an independent read-only audit accepted the
scientific finding, all twelve mappings, prose/JSON agreement, canonical
seal, links, whitespace and five-file scope. Contact counts are task
accounting, not independently sealed network telemetry. The later user
instruction authorizes publication of this historical inconclusive result
before a separate offline protocol correction. CI and exact promotion
results belong to the publication closeout; this record does not anticipate
their outcome.
