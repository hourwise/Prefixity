# P0-L12 — Candidate Evaluation and Evidence Gate

P0-L12 evaluates what Prefixity can currently justify about a P0-L11 layout
candidate. It is an offline, proposal-only evidence gate. It does not execute
the candidate, rewrite a request, run inference, predict cache behavior, or
authorize automatic application.

## Four separate questions

| Layer | Question | Source |
| --- | --- | --- |
| P0-L10 | What is structurally stable or volatile? | `ContextStabilityAnalysis` |
| P0-L11 | What safer structural candidate can be described under declared constraints? | `ContextLayoutPlan` / `LayoutCandidate` |
| P0-L12 | What evidence state is currently justified for that candidate? | `CandidateEvaluation` |
| P0-L6 and future experiments | What happens when a candidate actually runs? | Environment and real observations |

Structural merit and empirical merit are independent. A candidate may be
structurally strong while empirical evidence is unknown, or structurally
modest while real observations support a narrow association.

## Evidence ladder

The evaluator uses a conservative, versioned ladder:

- `structural_only`: a safe candidate exists, but no runtime capability or
  observation has been supplied.
- `capability_compatible`: the selected profile establishes the relevant
  capability, without showing that this candidate improves anything.
- `ready_for_experiment`: the candidate is safe and the capability is known
  sufficiently for a controlled experiment. The implementation uses this
  state for a documented prefix-reuse profile with no observations.
- `observationally_supported`: relevant experimentally observed diagnostics
  are directionally consistent with the narrow hypothesis. Causality remains
  `not_established`.
- `mixed_evidence`: relevant observations disagree across metrics or cases.
- `unsupported_by_current_evidence`: relevant experimental observations show
  no cache benefit or a contrary cache signal.
- `blocked`: safety, capability, identity, or evidence relationships cannot be
  established.

The ladder does not infer empirical support from a cleaner layout. Documented
capability and synthetic protocol/test evidence remain visibly separate from
experimentally observed runtime evidence.

## Bounded evaluation record

`CandidateEvaluation` keeps references and bounded summaries rather than
copying prompts, artifact contents, raw telemetry, or full capability
profiles. It contains candidate and request-diff fingerprints, a fixed
conditional structural hypothesis, P0-L9 capability state, relevant P0-L8
diagnostic references, structural assessment, evidence state, blockers, claim
permissions, next action, separate design/environment/execution readiness,
and deterministic provenance without an automatic timestamp.

P0-L7 `RequestDiff` remains the description of what changed and retains
`cache_impact: unknown`. P0-L8 remains the owner of metric deltas,
known/unknown/not-observed handling, comparability, synthetic boundaries and
causality. P0-L9 remains the owner of capability evidence. P0-L12 only gates
and combines those existing records.

## Claim and readiness boundaries

Structural claims are allowed only when P0-L11 marks the candidate
`ordering_safe_under_declared_constraints`. Capability claims are allowed only
when backed by a selected P0-L9 profile. Observation claims require real
experimental observation. Performance claims, causal claims and automatic
application are always disallowed by this slice.

Logical experiment design readiness is separate from machine readiness. The
current llama.cpp state is therefore representable as:

```text
candidate safety: established
prefix reuse capability: supported_documented
evidence state: ready_for_experiment
design readiness: ready
environment readiness: blocked
execution readiness: blocked
next action: resolve_environment
```

P0-L6 remains `environment-blocked` because no existing usable `llama-server`
or suitable GGUF model is available. P0-L12 does not inspect, install or
download either one and does not fabricate observations.

## Observation relevance and contrary evidence

Before a diagnostic can influence an evaluation, the evaluator checks the
runtime/model/protocol/runtime-version relationship, profile identity where
present, request fingerprints, candidate mutation relationship, envelope
alignment and P0-L8 comparability. Unrelated observations are retained as
rejected evidence and do not influence the state.

Mixed signals remain mixed. For example, increased reused tokens combined with
worse timing is not flattened into a validation result. A contrary observation
can produce `unsupported_by_current_evidence`; Prefixity is allowed to
disagree with its own structural hypothesis.

One observation pair supports an association only:

```text
this candidate was associated with a changed reused-token signal
causality: not_established
```

The evaluator implements no confidence intervals, significance tests,
regression thresholds, repeated-trial aggregation, benchmark score, provider
heuristic, cache simulation, or performance claim. ContextBench remains
pending and P0-L13 is outside this slice.
