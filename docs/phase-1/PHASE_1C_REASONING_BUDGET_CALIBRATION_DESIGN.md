# Phase 1C reasoning-budget calibration design

Status: `PREPARATION COMPLETE - NO LIVE CALIBRATION AUTHORIZATION`

This is an independent, non-scored runtime-feasibility calibration created
after the accepted h001 V2 result. It does not reopen h001, create V3, assess
Prefixity capability, or make a correctness claim about Qwen. Its only
question is whether a bounded reasoning-on configuration can produce a
complete, parseable terminal response under the existing total
`max_tokens=2048` ceiling.

## h001 V2 closure

The accepted h001 V2 BASELINE result is recorded as:

`PHASE 1C h001 V2 - INCOMPLETE / NON-COMPARABLE`

BASELINE is `INCONCLUSIVE - GENERATION CEILING EXHAUSTED`. The request was
HTTP 200 with 2048 completion tokens and `finish_reason=length`; reasoning
content was present and terminal final content was absent. The deterministic
evaluator did not run. NO_OP and INTERVENTION were not executed. V1 and V2
evidence remain immutable and no future calibration result may be combined
with either scored lineage.

## Local llama.cpp capability

The installed executable was inspected offline at
`C:\Users\USER\AppData\Local\Microsoft\WindowsApps\llama.exe`.
`--version` reported:

`b10217-ddd4ec142`

The installed `serve --help` reports:

- `--reasoning [on|off|auto]`; `on` explicitly enables chat reasoning;
- `--reasoning-budget N`; `-1` means unrestricted, `0` means immediate end,
  and positive `N` means a token budget for thinking;
- `--reasoning-budget-message MESSAGE`; a message injected before the
  end-of-thinking tag when the reasoning budget is exhausted.

The flags are server launch options, so the installed build exposes them as
server-side configuration. The calibration uses only `--reasoning-budget N`.
It leaves `--reasoning-budget-message` unset because that message would be an
additional runtime intervention. `--reasoning on` remains explicit. No server
was started and no localhost endpoint was contacted while establishing this
support record.

## Frozen runtime and candidate order

Every candidate retains the accepted scored-runtime settings:

- model `ggml-org/Qwen3.5-0.8B-GGUF:Q4_0`, quantization `Q4_0`;
- context `8192`, one parallel slot, metrics enabled;
- reasoning `on`, temperature `0`, top-p `1`, seed `1`;
- total `max_tokens=2048`, stream `false`;
- connect timeout `1000 ms`, request timeout `1200000 ms`, supervisor timeout
  `1320000 ms`;
- automatic retries, fallback requests, and adaptive replicates all `0`.

Exactly these server-side reasoning budgets are pre-registered, in descending
order:

`1024 -> 512 -> 256`

Each candidate uses a fresh llama.cpp process. The three independent synthetic
cases run in fixed order on that fresh process, with no warmup. The cases do
not share h001 material and the feasibility predicate concerns terminal
generation and structural completion rather than cache reuse, so a fresh
server per candidate is sufficient and is the registered freshness policy.

## Independent calibration cases

The machine-readable manifest freezes exactly three cases:

| Case | Structural workload | Request SHA-256 | Bytes |
| --- | --- | --- | ---: |
| `rbcal-001` | duplicate records with an explicit operation reference | `f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461` | 1224 |
| `rbcal-002` | two dependency records with an explicit job dependency | `e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3` | 1206 |
| `rbcal-003` | ordered facts with an explicit task source reference | `2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e` | 1166 |

Each request contains only `model`, `messages`, `temperature`, `top_p`,
`max_tokens`, `stream`, and `seed`. The reasoning budget is deliberately not
an API request field. Each expected response is a trivial exact JSON object
with three string fields. The cases are structural feasibility probes, not
semantic capability benchmarks.

The frozen manifest is
`PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.json` with canonical
SHA-256:

`4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b`

Its sidecar is
`PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.sha256`.

## Pass, stopping, and accounting rules

A case passes only when it receives HTTP 200, the bounded response is valid
JSON, `finish_reason` is not `length`, terminal assistant content is
non-empty, the content is exactly the registered structural JSON object, and
transport is unambiguous. A candidate passes only when all three cases pass.

The runner completes the full three-case set for a candidate. A passing 1024
candidate stops calibration; otherwise the registered next candidate is 512.
A passing 512 candidate stops calibration; otherwise the registered next
candidate is 256. A failing or inconclusive 256 candidate produces the exact
terminal classification:

`REASONING-ON / 2048-TOKEN SCORED CONFIGURATION NOT FEASIBLE`

No candidate is tested twice, and no unregistered value may be supplied. The
maximum is 9 inference requests: three cases for each of the three
candidates. Runtime ambiguity is preserved as `INCONCLUSIVE`; it is never
retried or inferred to be zero.

## Experiment-only runner

The isolated runner is implemented in
`crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs`
and exposed as the
`prefixity-phase1c-reasoning-budget-calibration` binary.

Offline commands are:

```text
prefixity-phase1c-reasoning-budget-calibration fingerprint
prefixity-phase1c-reasoning-budget-calibration preflight
prefixity-phase1c-reasoning-budget-calibration dry-run
prefixity-phase1c-reasoning-budget-calibration summarize --budget 1024
```

The future live candidate command is intentionally strict:

```text
prefixity-phase1c-reasoning-budget-calibration run --budget 1024 --confirm-fresh-runtime
```

The child must be run through the checked-in live supervisor under a later
authorization. The runner validates candidate order, uses the fixed server
launch template, performs one listener check per candidate, sends at most
three sequential requests, and never retries. It persists raw bounded
responses, normalized final-content records, separate reasoning diagnostics,
per-case results, and a candidate aggregate. Reasoning content is never used
as final content, scored, or passed to another case.

## Live boundary

This preparation performs no server startup, listener check, HTTP request,
inference, warmup, or calibration. The calibration requires a later explicit
authorization naming the manifest hash, candidate, fresh-server confirmation,
server launch, supervisor boundary, and evidence retention. This document
does not authorize V3 or any scored h001 replay.
