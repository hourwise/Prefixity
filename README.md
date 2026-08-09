# Prefixity

> **Prefixity is experimental research software.**
> **Phase 0 does not modify live LLM requests.**

Prefixity is a provider-neutral, **offline** context-efficiency profiler for
LLM/agent workloads. It analyses recorded or synthetic request traces to
explain where context cost is incurred, where reusable prefixes diverge, and
what alternative context policies *might* have done — before any live request
is touched.

Phase 0 is a small, deterministic research harness. It exists to confirm or
invalidate the core thesis:

> For an observed LLM/agent workload, can a deterministic tool explain where
> context cost is being incurred, identify prefix divergence and unnecessary
> context, model provider-specific cache economics, and simulate alternative
> context policies before modifying live prompts?

A perfectly acceptable Phase 0 result is:

> "Do nothing. Your existing client is already close to optimal."

## Status

- **Phase 0A** complete.
- **Phase 0A.1** complete (semantic corrections; CI green on Linux/macOS/Windows).
- **Phase 0B** DeepSeek live validation complete: **PASS WITH RECORDED
  LIMITATIONS** (see `docs/phase-0/PHASE_0B_DEEPSEEK_CLOSEOUT.md`).
  OpenAI/Anthropic adapters remain offline-tested, not live-validated.
  - **Phase 1A** complete and **Phase 1B.0** conservative offline planning
    implemented; Phase 1C replay remains unauthorized (see
    `docs/phase-1/PHASE_1B_DECISION_CONTRACT.md`).
- No daemon, no localhost proxy, no telemetry.
- No semantic response caching, no KV-cache storage, no RAG, no automatic
  context mutation of live requests.
- Repository: <https://github.com/hourwise/Prefixity>. Phase 0B makes paid
  provider calls **only** with `--execute-live`, an explicit request limit
  (default 4, hard ceiling 10), and a local input-token ceiling.

## Requirements

- Rust 1.86+ (workspace source remains Edition 2021).
- The offline analysis harness needs no network access.
- The experimental `prefixity-live` harness needs network access **only**
  when `--execute-live` is passed, and reads credentials only from
  environment variables (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`,
  `DEEPSEEK_API_KEY`).

## Quick start

```sh
# Validate a trace file's structure
cargo run -p prefixity-cli -- validate fixtures/traces/01-stable-prefix.json

# Analyse one request: accounting, prefixity scores, cost
cargo run -p prefixity-cli -- analyse fixtures/traces/04-large-tool-output.json \
  --provider-profile provider-profiles/synthetic-example.json

# Compare two consecutive requests: divergence, reusable prefix
cargo run -p prefixity-cli -- compare fixtures/traces/03-tool-order-break.json \
  fixtures/traces/03-tool-order-break-turn2.json

# Simulate an alternative context policy (offline; never mutates the trace)
cargo run -p prefixity-cli -- simulate fixtures/traces/06-context-reduction-wins.json \
  --policy combined --provider-profile provider-profiles/synthetic-example.json

# Produce a conservative Phase 1B intervention plan (offline/hypothetical)
cargo run -p prefixity-cli -- plan fixtures/traces/06-context-reduction-wins.json --json
```

Add `--json` to any command for stable, machine-readable output.

## Commands

| Command | Purpose |
| --- | --- |
| `prefixity validate <trace>` | Structural validation only. |
| `prefixity analyse <trace>` | Single-trace accounting, prefixity scores, cost. |
| `prefixity compare <a> <b>` | Divergence detection and reusable-prefix estimation. |
| `prefixity simulate <trace> --policy <policy>` | Offline policy simulation. |
| `prefixity plan <trace>` | Conservative Phase 1B intervention planning; never mutates the trace. |

Policies (research hypotheses, not production recommendations):
`baseline`, `stable-prefix`, `defer-volatile`, `prune-stale-tool-output`,
`combined`. The name `compression` is reserved for future work.

## Live validation harness (Phase 0B, experimental)

The separate `prefixity-live` binary sends **tightly controlled synthetic
requests** to OpenAI, Anthropic, or DeepSeek to test whether the offline
model matches real provider usage reports. It makes paid calls **only** with
`--execute-live`; `dry-run` and `run` without the flag make zero network
requests.

```sh
# Dry run: prints exactly what would be sent. Zero network, no credential needed.
cargo run -p prefixity-live -- dry-run --provider openai --model <model> --scenario schema-smoke

# Live run: explicit opt-in required.
cargo run -p prefixity-live -- run --provider openai --model <model> --scenario stable-prefix --execute-live
```

Scenarios: `schema-smoke`, `stable-prefix`, `early-divergence`,
`late-divergence`. See `docs/phase-0/PHASE_0B_LIVE_VALIDATION.md`.

## Three distinct concepts

Prefixity keeps three concepts strictly separate (Phase 0A.1):

- **Prefixity score** — an experimental heuristic estimate of whether a
  block looks suitable for stable-prefix placement (single trace).
- **Observed prefix reuse** — exact structural prefix match between two
  recorded requests (`prefixity compare`). A single trace can never prove
  reuse; single-trace figures are called *stable-prefix candidates* and are
  heuristic only.
- **Provider-reported cache reuse** — tokens the provider explicitly reports,
  captured as raw usage and normalized per schema. Per source-of-truth
  principle 7, provider-reported values outrank Prefixity's estimates when
  determining what actually happened.

Phase 0B live validation adds an explicit distinction between **structural
potential** and **realized cache** (see `docs/phase-0/PHASE_0B_FINDINGS.md`):

- **`structural_reuse_ratio`** — the observed reusable-prefix **POTENTIAL**
  in Prefixity's structural model. It is *not* a prediction of the exact
  provider cache-hit ratio.
- **`provider_cache_reuse_ratio`** — the **REALIZED** provider cache reuse
  reported for that request (best-effort, asynchronous persistence may lag
  or exceed structural potential).
- **`reuse_ratio_difference`** — the realization/alignment gap between those
  observations. PARTIAL_MATCH / NO_MATCH do not by themselves prove that
  Prefixity's structural analysis is wrong; provider cache availability and
  state can differ.

Cost never bills candidates at cache-read prices unless provider-normalized
usage supports it.

## Repository layout

```
Prefixity/
├── Cargo.toml                 # workspace manifest
├── crates/
│   ├── prefixity-core/        # deterministic analysis logic (authoritative)
│   ├── prefixity-cli/         # thin CLI over the core logic
│   └── prefixity-live/        # Phase 0B live validation harness (experimental)
├── docs/                      # charter, threat model, prior art, phase-0 docs
├── fixtures/traces/           # synthetic trace fixtures (no real secrets)
├── provider-profiles/         # SYNTHETIC cost profiles (not real pricing)
└── experiments/               # live run artifacts (runs/ gitignored)
```

## Source-of-truth principles

1. Prefixity's cache/index state will never be authoritative.
2. Original source/provider state wins over derived Prefixity state.
3. Future Prefixity storage must be disposable and rebuildable.
4. Optimisation must eventually be fail-open: if Prefixity fails, the
   original request must remain usable.
5. Observation precedes transformation.
6. Simulation precedes automatic optimisation.
7. Provider-reported cache usage outranks Prefixity's theoretical estimate
   when determining what actually happened.
8. A lower token count is not automatically a better result if correctness
   degrades.

## Safety and privacy

- Phase 0 fixtures contain **no real credentials and no private source code**.
- Traces may omit prompt content entirely (hashes + metadata suffice).
- Untrusted content is sanitised before terminal display.
- Input files are size-bounded; see `docs/THREAT_MODEL.md`.

## Documentation

- `docs/PROJECT_CHARTER.md` — purpose, boundaries, non-goals.
- `docs/THREAT_MODEL.md` — security/privacy considerations for trace data.
- `docs/research/PRIOR_ART.md` — prior art and why it shaped the hypothesis.
- `docs/phase-0/PHASE_0_PLAN.md` — the Phase 0 plan.
- `docs/phase-0/TRACE_FORMAT.md` — the versioned trace format.
- `docs/phase-0/EXPERIMENTS.md` — the three future experimental groups.
- `docs/phase-0/PHASE_0B_LIVE_VALIDATION.md` — controlled live validation
  (purpose, guardrails, scenarios A–D, procedure, stop conditions).
- `docs/phase-0/PHASE_0B_DEEPSEEK_CLOSEOUT.md` — DeepSeek Phase 0B closeout:
  evidence matrix, decision (PASS WITH RECORDED LIMITATIONS), stopping rule.
- `docs/phase-0/SUCCESS_CRITERIA.md` — success/failure criteria and stop
  conditions.
- `docs/phase-1/PHASE_1_PLAN.md` — the Phase 1 plan: real-workload
  observation and quality-gated context decisions (1A/1B/1C, design gate).
- `docs/phase-1/QUALITY_GATE.md` — Phase 1 quality gate, evidence tiers and
  intervention rules.
- `docs/phase-1/SUCCESS_CRITERIA.md` — Phase 1 success/failure and pivot
  criteria.
- `docs/phase-1/WORKLOAD_CORPUS.md` — Phase 1 workload corpus and provenance
  requirements.
- `docs/phase-1/PRIOR_ART_DECISIONS.md` — Phase 1 prior-art decisions
  (reuse/integrate/differentiate).

## License

MIT — see `LICENSE`.
