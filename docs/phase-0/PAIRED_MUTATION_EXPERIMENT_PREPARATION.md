# P0-L6B — Paired Mutation Control/Treatment Experiment Preparation

P0-L6B adds a second experiment type beside the existing P0-L6A harness. It
prepares a paired vanilla/candidate mutation test and produces no live
evidence. It does not contact localhost, start llama.cpp, run inference, load
GGUF, rerun the exploratory benchmark, add a tokenizer, or begin P0-L14.

## Why this experiment exists

Exploratory native llama.cpp mutation measurements motivate a conditional
hypothesis: moving independent stable material ahead of independently mutable
material may let ordinary native prefix caching retain more reusable prompt
context across that volatile mutation. Those exploratory measurements are
motivation only, not Prefixity improvement evidence. The outcome contract is
deliberately falsifiable and does not require C1 to be better than A1.

## Paired layouts

```text
VANILLA

stable A
volatile V0
stable B

         ↓ V changes

stable A
volatile V1
stable B

native reusable prefix may terminate near V
```

```text
PREFIXITY CANDIDATE

stable A
stable B
volatile V0

         ↓ V changes

stable A
stable B
volatile V1

larger unchanged leading region is available
to native prefix caching
```

The seed is synthetic and semantically independent. P0-L10 receives explicit
stable/volatile metadata; the planner does not infer stability heuristically.
Stable B and the designated volatile artifact have explicit compatible-region
movement permission so P0-L11 can propose the approved order while preserving
trust, provenance, hierarchy, chronology, lifecycle, and independence.

## Certified states and mutation

The definition contains four certified request states and one deterministic
interference request:

| State | Meaning |
| --- | --- |
| A0 | vanilla control with stable A, volatile V0, stable B |
| A1 | same control with only designated volatile V1 content changed |
| C0 | independent P0-L13 materialization of A0 |
| C1 | independent P0-L13 materialization of A1 |
| B1 | deterministic interference request; not a cache-destruction guarantee |

V1 is a small deterministic content/version marker change. System instruction,
stable artifacts, current user, tools, envelope, and artifact membership stay
unchanged. C1 is never produced by editing C0 after certification; A1 is
planned and materialized afresh through the same P0-L11/P0-L12/P0-L13 path.

## Fixed five-request sequence

```text
A0  control_initial
A1  control_mutated       volatile mutation of A0
B1  deterministic interference
C0  treatment_initial     P0-L13-certified layout of A0
C1  treatment_mutated     P0-L13-certified layout of A1
```

There are no repeats, combinatorial trials, automatic retries, cache clearing,
or server startup/restart actions. A later operator must manually start the
llama.cpp server fresh with no prior chat-completion requests and assert
`fresh_server_for_run = true`. Prefixity does not infer that condition by
probing or mutate runtime state.

## Verification and evidence flow

Preflight requires exact P0-L7 rules: A0→A1 and C0→C1 are one designated
volatile content change with no order or envelope change; A0→C0 and A1→C1
are authorized artifact reorders only. It records P0-L10 inversion count and
leading-region changes without turning them into a performance claim. Both C0
and C1 must retain valid independent P0-L13 safety certificates.

The later primary endpoint is compatible native cache/prefill accounting:

```text
A1 reused/cached prompt tokens   versus   C1 reused/cached prompt tokens
A1 fresh/evaluated prompt tokens versus   C1 fresh/evaluated prompt tokens
```

Prompt/prefill time is recorded exactly when exposed, but remains descriptive
and noise-sensitive. A single run cannot establish speedup, percentage
improvement, statistical significance, or causality. Raw llama evidence still
flows through P0-L5 normalization, `CacheObservation`, P0-L8 comparisons, and
P0-L12 evaluation without duplicating those implementations.

## Runtime preparation

The intended later profile is llama.cpp with model identity
`ggml-org/Qwen3.5-0.8B-GGUF`, quantization `Q4_0`, endpoint
`http://127.0.0.1:8080/v1/chat/completions`, context 8192, one parallel slot,
and metrics enabled. Build/version, thread count, batch size, KV precision,
and GPU offload remain caller-supplied or unknown. Generation is bounded to a
small deterministic value and no reasoning/thinking setting is introduced.

The first workload is moderate and byte-bounded, targeting roughly the scale
of the exploratory 2000-token bucket without adding a tokenizer. Projected
requests remain bounded below the configured context limit.

## Current status

P0-L6B prepares this experiment but produces no live evidence. The exploratory
llama.cpp mutation benchmark motivates the hypothesis but does not count as
Prefixity improvement evidence. Live execution remains separately gated and
pending the approved environment and operator assertion.
