# Phase 0B DeepSeek Closeout

Evidence-led closeout of the controlled DeepSeek live validation sequence.
This is a research closeout, not marketing material, and not a production
validation.

## 1. Objective

Validate whether Prefixity's provider-neutral structural context analysis
can be reconciled meaningfully with real provider cache accounting **without
pretending Prefixity's estimated tokens and provider tokenizer tokens are
the same unit**. Reconciliation is therefore ratio-based, with each
measurement kept in its own unit.

## 2. Provider / model

- **Provider:** DeepSeek
- **Model:** `deepseek-v4-flash`
- **API surface:** DeepSeek Chat Completions (`deepseek-chat-completions-v1`)

The OpenAI and Anthropic adapters remain **offline-tested but NOT
live-validated** in this phase.

## 3. Schema smoke result

| Field | Value |
| --- | --- |
| Prefixity estimate | 563 estimated tokens (chars/4) |
| Provider total input | 1215 provider tokens |
| Cache read | 0 |
| Schema result | **MATCH** |

**Finding:** generic token estimates and provider tokenizer counts are
materially different units (563 vs 1215 for the same request).

**Consequence:** absolute cross-tokenizer token subtraction was abandoned in
favour of ratio-based reconciliation.

## 4. Stable-prefix result

Primary stable pair (A → B):

- structural reuse potential ≈ **99.8%** (8048/8063)
- realized cache reuse ≈ **99.9%** (18048/18062)
- **MATCH**

## 5. Early-divergence negative control

Pair B → C (early header changed):

- structural reuse potential = **0%** (0/8066)
- realized cache reuse = **0%** (0/18064)
- **MATCH** (consistent no-reuse observation)

## 6. First late-divergence observation

Pair B → C (late suffix first changed):

- structural reuse potential ≈ **89.9%** (7245/8063)
- realized cache reuse ≈ **57.9%** (10496/18115)
- **PARTIAL_MATCH**

This observation **must remain visible**. It is not hidden because the later
persistence probe matched: it is the evidence that structural reuse potential
and realized provider cache state are distinct, and that realized cache
availability can lag structural potential.

## 7. Corrected late-prefix persistence observation

Pair C → D (PRIMARY; D used a second, distinct late suffix after a controlled
10-second settle):

- structural reuse potential ≈ **89.9%** (7245/8063)
- realized cache reuse ≈ **90.0%** (16256/18070)
- realization/alignment gap ≈ **0.106 percentage points**
- **MATCH**

Overall conclusion: **MATCH**.

## 8. Evidence matrix

| Scenario | Structural potential | Realized cache | Outcome |
| --- | --- | --- | --- |
| Stable prefix | ~99.8% | ~99.9% | MATCH |
| Early divergence | 0.0% | 0.0% | MATCH |
| First late divergence | ~89.9% | ~57.9% | PARTIAL_MATCH |
| Late persistence probe | ~89.9% | ~90.0% | MATCH |

## 9. What Phase 0B establishes

Evidence supports (with the limitations below):

- raw DeepSeek cache accounting can be captured and normalized;
- structural prefix identity is meaningfully observable independently of
  provider tokenization;
- an early structural break corresponds to loss of realized cache reuse in
  this controlled test;
- a stable structural prefix corresponds extremely closely to realized
  provider cache reuse in this controlled test;
- structural reuse **POTENTIAL** and realized provider cache state are
  distinct;
- a reusable boundary can apparently exist structurally before equivalent
  provider cache reuse is realized;
- ratio-based reconciliation is materially more meaningful than absolute
  cross-tokenizer token subtraction.

## 10. What Phase 0B does NOT establish

Explicitly **not** established:

- not proof across providers;
- not proof across DeepSeek models;
- not proof across natural agent workloads;
- not evidence that chars/4 is an accurate tokenizer;
- not evidence that a universal DeepSeek conversion multiplier exists;
- not proof that 10 seconds is necessary or optimal;
- not proof that structural potential guarantees provider cache realization;
- not a validated cost-saving percentage;
- not evidence that provider cache behaviour is deterministic.

## 11. Remaining uncertainty

- one live observation per controlled scenario;
- deterministic synthetic corpus rather than natural agent traces;
- provider cache state is best-effort/asynchronous;
- serialization/tokenization overhead differs from Prefixity's structural
  blocks;
- cache-unit boundaries remain provider-controlled;
- OpenAI/Anthropic live semantics remain untested.

## 12. Decision

> **DEEPSEEK PHASE 0B VALIDATION: PASS WITH RECORDED LIMITATIONS**

This means the Prefixity architectural hypothesis has survived this
controlled DeepSeek falsification exercise sufficiently to justify continued
development.

It does **NOT** mean Prefixity is production validated.

## Phase 0B DeepSeek stopping rule

The controlled DeepSeek live matrix is complete. No further DeepSeek Phase
0B repetitions are recommended solely to improve confidence or obtain
prettier ratios. Additional provider calls require a **new explicit research
question**. This prevents post-hoc number chasing.
