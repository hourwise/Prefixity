# Phase 1C Claim-2 non-inference tokenization preparation

Status: `OFFLINE_CERTIFIED_PENDING_EXACT_SHA_CI_AND_PROMOTION`.
No tokenizer contact, model startup, inference, token admission result, CI
promotion, or operator-server readiness is claimed by this record.

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

The input-token response parser follows the repository's accepted
`phase1c_v3_feasibility::parse_input_tokens` contract: `object` must equal
`response.input_tokens`, and `input_tokens` must be an unsigned integer.
Missing, malformed, oversized, or non-200 responses are unusable. Any future
server-schema mismatch stops the pass and does not authorize another tokenizer.

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

After, and only after, the preparation has passed exact-commit CI and been
promoted, the operator may start the server with:

```powershell
C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe serve -m D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080
```

Before launch, verify that `Get-ChildItem Env:LLAMA_ARG_*` returns nothing.
This preparation does not authorize the operator to launch the server or run
either client mode against it.

The intended offline command is:

```powershell
& 'D:\Users\fleur\Prefixity\target\claim2-tokenization-freeze\2945e3af7f059df643d0f15cca45b3ba9f343249\phase1c_claim2_tokenization.exe' preflight
```

The future guarded execution command, unavailable until exact-commit CI and
promotion succeed, is:

```powershell
& 'D:\Users\fleur\Prefixity\target\claim2-tokenization-freeze\2945e3af7f059df643d0f15cca45b3ba9f343249\phase1c_claim2_tokenization.exe' execute --operator-started --model-path D:\Prefixity-Lab\models\Qwen3.5-9B\Qwen3.5-9B-Q4_K_M.gguf --model-sha256 cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13 --llama-path C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe --llama-sha256 cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617 --llama-build b10217-ddd4ec142 --context 8192 --slots 1 --reasoning off --offline --host 127.0.0.1 --port 8080 --evidence .\claim2-tokenization-pass-evidence-v1.json
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

The focused client test target passed 8/8 tests, covering exact allowlist
paths, generation-path rejection, plan and ledger tamper/substitution,
unchanged 22-entry ordering, exact-body dispatch through a fake transport,
readiness stop, token-response parse stop, no retries, and partial evidence.
The executable confirmation test passed 1/1. The full locked/offline workspace
test suite passed. `cargo fmt --all -- --check`, workspace Clippy with
warnings denied, and the Rust 1.86 locked/offline workspace check passed.
Plan generation reproduced the pinned bytes. All contact accounting in these
tests used fake transports; no test contacted the loopback endpoint.

## Source-provenance freeze and offline certification

The source-provenance commit is
`2945e3af7f059df643d0f15cca45b3ba9f343249`, descended from
the accepted `cabb729ba417a8f42f90848fd28f0d1bbdedfe93` main baseline.
The locked/offline release build from that commit produced a 3,792,896-byte
Windows executable. A read-only copy was frozen at the path used in the two
commands above, with SHA-256
`113ba4d5c9e7b8e01efa451427679572481a6196fb14aac73863e30d36d96e81`
and Windows file ID `0x00000000000000000006000000459c59`.

The distinct [tokenization experiment identity](PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V1.json)
is `claim2-tokenization-v1-a5a6b896555db8296318f38010b7120dad8ad191e1329f030e0f738f31b90b91`.
Its [canonical SHA-256 seal](PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V1.sha256)
is `4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16`.
It binds source blob hashes, the successor and domain ledgers, all six fixture
identities, all 54 logical request hashes and message hashes, the 22-entry
sorted deduplication map, frozen executable path/hash/size/file ID, accepted
GGUF and llama executable identities, fixed runtime and endpoint contract,
contact limits, and zero-inference rule. This identity does not reuse the
historical 9B competence-gate identity.

Before sealing, the frozen executable's offline `preflight` accepted the exact
ledger/plan hashes and reported 54 logical requests, 22 unique bodies, 32
duplicates avoided, and zero server or inference contacts. A read-only
process/port check observed no llama process or TCP listener on port 8080;
the current shell had no `LLAMA_ARG_*` variable. `Get-CimInstance` was denied
by the environment, so `Get-Process` and `netstat` supplied the read-only
absence check. The accepted GGUF and llama executable hashes were verified
from the current files without loading the model. A separate one-byte
substituted client copy had a different SHA-256 and was rejected by the
identity check; the frozen copy remained unchanged. The focused ledger/plan
tests reject tampering and substitution before transport creation.

After promotion and immediately before any future execution, verify that the
frozen executable still has the bound SHA-256. A substituted executable must
not be used. The operator-started server's process, command line, loaded
artifact, listener and exclusivity must be verified separately before any
token-count contact; the CLI confirmation flags alone do not establish those
runtime facts. The exact future execution command above writes to a new file
in the repository root, whose parent already exists. Do not run it during
this preparation.

Supervisor interpretation: request identity and non-inference preparation are
offline-certified. No actual input-token count, threshold result, context-fit
result or updated CPU classification exists. Candidate CI and exact-SHA
promotion remain the only preparation-publication gates. No operator server
startup, readiness request, input-token request or inference occurred.
