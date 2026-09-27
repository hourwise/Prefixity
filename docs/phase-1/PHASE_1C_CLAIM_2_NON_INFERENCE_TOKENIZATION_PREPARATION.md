# Phase 1C Claim-2 non-inference tokenization preparation

Status: `PHASE_3_IMPLEMENTATION_DRAFT_PENDING_SUPERVISOR_FREEZE`.
No tokenizer contact, model startup, inference, source-provenance commit,
executable freeze, tokenization identity, CI promotion, or operator-server
readiness is claimed in this local preparation draft.

## Scope and frozen inputs

This preparation follows the accepted canonical advancing-output protocol and
the Phase 2-reviewed request ledger. It leaves the Phase 1 ledger and every
request body unchanged. The ledger SHA-256 is
`739205fb56e4f40bd55245f37d0768b8ca73c891b2f8d28bdb8f284e9f811d45`.
The accepted materialization successor, advancing-output domain ledger, six
fixture manifests, and their exact identities are already bound by that full
ledger hash.

The offline contact plan is
[`fixtures/claim2/tokenization-contact-plan-v1.json`](../../fixtures/claim2/tokenization-contact-plan-v1.json),
SHA-256 `6b7634a3fc7a3d4b76289aea1772c1df686a6641309acc9e307e5f330dfefdf6`.
It fixes 54 logical requests, 22 exact unique bodies, 32 duplicate logical
requests avoided, one maximum readiness contact, 22 maximum input-token
contacts, and zero inference allowance. The 22 entries are ordered by
lowercase request-body SHA-256 ascending. Each entry binds the representative
logical request ID, all mapped logical IDs, exact byte length, and body hash.
The client resolves the representative body from the accepted ledger, verifies
its bytes, and sends those bytes without reserialization.

Plan regeneration is offline:

```powershell
cargo run -p prefixity-controlled-benchmark --example claim2_tokenization_contact_plan --offline --locked -- --write fixtures/claim2/tokenization-contact-plan-v1.json
```

The client is `prefixity-phase1c-claim2-tokenization`, implemented in
[`crates/prefixity-controlled-benchmark/src/phase1c_claim2_tokenization.rs`](../../crates/prefixity-controlled-benchmark/src/phase1c_claim2_tokenization.rs)
and [`crates/prefixity-controlled-benchmark/src/bin/phase1c_claim2_tokenization.rs`](../../crates/prefixity-controlled-benchmark/src/bin/phase1c_claim2_tokenization.rs).
It loads only the fixed ledger and plan paths compiled from the crate manifest.
Before creating a transport or contacting readiness, it checks both file
SHA-256 values, the complete plan value, the 54 logical identities, every
exact body hash and byte length, request semantics, deduplication equality map,
hash ordering, and contact limits. It will not append requests after reading
counts.

## Contact and failure contract

The only production transport methods are readiness and input-token counting.
The static loopback allowlist is:

| Operation | Method | URL |
| --- | --- | --- |
| Readiness | GET | `http://127.0.0.1:8080/health` |
| Input tokens | POST | `http://127.0.0.1:8080/v1/chat/completions/input_tokens` |

The executable has no URL, host, port, or endpoint-path option. It disables
redirects and proxies. It registers no completion or generation method. The
operator must explicitly select `execute`, confirm the accepted model and
llama.cpp runtime identity, and confirm that the server has already been
started. It does not start a process. It also stops before contact if any
`LLAMA_ARG_*` environment variable is present.

There is no retry or fallback. Readiness failure stops before any token-count
request. An input-token transport, HTTP, or response-parse failure stops at
that exact hash; the client preserves the partial evidence and reports
`TOKENIZATION_PASS_INCONCLUSIVE`. Evidence includes attempted contact counts,
hash-to-logical-request mapping, per-hash token counts when usable, response
status and body hashes, and the first failure. A fully counted record remains
pending scientific review and makes no admission decision.

The input-token response parser accepts exactly one integer count field named
`input_tokens` or `tokens`; missing, ambiguous, non-integer, oversized, or
non-200 responses are unusable. This response vocabulary is an implementation
assumption pending a future operator result; any mismatch stops the pass and
does not authorize a different tokenizer.

## Preregistered metric definitions

For CP01–CP04, each count is joined by the frozen request-body hash to its
logical case, arm, and slot. Requests 1 and 2 have exact body identity between
BASELINE and INTERVENTION, so `B1 = I1` and `B2 = I2`. Use the authoritative
records to calculate:

```text
D3   = B3 - I3
Bsum = B1 + B2 + B3
Isum = I1 + I2 + I3
Dsum = Bsum - Isum
R3   = D3 / B3
Rsum = Dsum / Bsum
```

Although the frozen histories imply `Dsum = D3`, calculate `Rsum` from all
three authoritative counts in each arm. Each positive case must independently
meet `D3 >= 800`, `R3 >= 0.20`, and `Rsum >= 0.08`. Every counted request must
also satisfy `input_tokens <= 6000` and `input_tokens + 1024 <= 8192`. Do not
truncate, lower the output ceiling, pad inputs, or choose a different
duplicate.

For CP05 and CP06, each corresponding request must retain exact BASELINE =
NO_OP = INTERVENTION body identity and equal input-token counts. Their legal
reduction is exactly zero; any mutation invalidates control admission.

CPU practicality remains unknown. Once authoritative input totals exist, the
preregistered 9–10 tokens/second rate may be applied to measured input-token
totals for a separate estimate, with that estimate labelled separately from
measured runtime. No current token outcome is inferred from bytes or historical
throughput.

All token counts, context-fit outcomes, positive reductions, threshold
outcomes, control outcomes, and updated CPU-practicality outcomes remain
`null` / `unknown` until an authorized tokenization execution and review.

## Runtime identity and operator boundary

The future operator runtime remains the accepted configuration: Qwen3.5-9B
Q4_K_M at
`D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf`, SHA-256
`cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13`;
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`, build
`b10217-ddd4ec142`, SHA-256
`cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617`;
reasoning off; context 8192; one slot; offline; loopback host and port 8080.

After, and only after, the supervisor has committed source provenance, rebuilt
and frozen the client, created the separate tokenization identity, completed
offline certification and accepted the exact preparation, the operator may
start the server with:

```powershell
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

Before launch, verify that `Get-ChildItem Env:LLAMA_ARG_*` returns nothing.
This preparation does not authorize the operator to launch the server or run
either client mode against it.

The intended offline command is:

```powershell
.\target\release\phase1c_claim2_tokenization.exe preflight
```

The future guarded execution command, unavailable until the separate freeze
and identity steps are complete, is:

```powershell
.\target\release\phase1c_claim2_tokenization.exe execute --operator-started --model-path D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf --model-sha256 cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13 --llama-path C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe --llama-sha256 cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617 --llama-build b10217-ddd4ec142 --context 8192 --slots 1 --reasoning off --offline --host 127.0.0.1 --port 8080 --evidence .\claim2-tokenization-pass-evidence-v1.json
```

The command records the operator's confirmations; it does not independently
inspect a running server's loaded model or runtime settings. Before contact,
the client itself re-hashes the fixed local GGUF and `llama.exe` files and
requires their accepted SHA-256 values; the executable hash binds the accepted
build. The server must remain operator-managed, and the eventual identity
review must bind the accepted runtime artifacts and frozen client executable
before this command is authorized.

## Validation evidence and remaining gates

Offline preflight passed with status
`OFFLINE_PREFLIGHT_PASSED_ZERO_CONTACTS`: it accepted the exact ledger and
plan hashes, verified 54 logical requests and 22 sorted unique bodies, and
reported 0 server contacts and 0 inference requests. It did not create an
HTTP client, read the model file, or start a process.

The focused client test target passed 7/7 tests, covering exact allowlist
paths, generation-path rejection, plan and ledger tamper/substitution,
unchanged 22-entry ordering, exact-body dispatch through a fake transport,
readiness stop, token-response parse stop, no retries, and partial evidence.
The executable confirmation test passed 1/1. The full locked/offline workspace
test suite passed. `cargo fmt --all -- --check`, workspace Clippy with
warnings denied, and the Rust 1.86 locked/offline workspace check passed.
Plan generation reproduced the pinned bytes. All contact accounting in these
tests used fake transports; no test contacted the loopback endpoint.

Interpretation: implementation and offline checks support review of this
draft only. The Phase 2 exact-hash review has been accepted by the supervisor,
but this Phase 3 source has not been source-committed or frozen as an
executable. No separate experiment identity, offline executable certification,
CI result, promotion SHA, operator server startup, readiness request, or token
count exists yet. Supervisor review and those preparation gates remain before
operator use.
