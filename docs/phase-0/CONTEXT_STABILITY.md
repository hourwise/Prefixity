# P0-L10 — Context Stability and Volatility Analysis

P0-L10 classifies one neutral P0-L4 request using optional P0-L2
ContextArtifact metadata. It describes structural stability
characteristics; it does not choose an optimal layout.

## Stability, lifecycle, and trust

Stability describes how expected a component's content is to change in the
relevant context. P0-L2 supplies the vocabulary immutable, stable,
append_only, volatile, and unknown. Lifecycle is separate:
persistent_versioned, transient, and unknown.

Trust is also separate. A stable artifact can be untrusted, and a volatile
artifact can be trusted. Stable does not mean trusted. Transient does not mean
volatile.

Explicit artifact metadata takes precedence over structural role defaults.
When metadata is absent, P0-L10 uses documented, overridable defaults:
system instruction is stable/persistent-versioned, current user content is
volatile/transient, and tool definitions are unknown. These are request-role
classification rules, not observed facts about every workload.

## Segments and boundaries

The analysis retains bounded segment paths, role, artifact identity,
fingerprint, classification source, trust, sizes, and explicit token metadata
where supplied. It does not copy prompts, source files, tool content, or other
large values.

Adjacent segments produce deterministic boundaries. A boundary records both
stability/lifecycle pairs and whether movement is toward more stable material,
toward more volatile material, unchanged among known classes, or unknown.

A stability inversion means:

> A more stable context segment occurs after a less stable segment in the
> current request order.

For example, stable -> volatile -> stable contains an inversion at the
volatile -> stable boundary. A stability inversion is a structural
observation, not proof of cache loss.

## Leading region and sizes

P0-L10 reports a stability-aligned leading region. It is the leading structural
region before an inversion or an unknown stability boundary limits stronger
classification. It is not called a cached prefix and does not imply provider
cache reuse.

Known byte sizes are reported by stability class. Unknown-size components keep
the aggregate unknown rather than becoming zero. Explicit artifact token
measurements are preserved on their segment, but the aggregate token analysis
is not_observed; P0-L10 introduces no tokenizer and does not estimate tokens
from characters or bytes.

Append-only history is a first-class classification. A stable historical region
and a growing tail can therefore remain distinguishable without claiming that
the history is fully reusable.

## Tools and unknowns

Tool definitions remain in request order. P0-L2 ContextArtifact metadata can
be supplied by tool name to classify an individual tool/schema component.
Without that metadata, dynamic tool material remains unknown; tools are not
canonicalized or reordered.

Unknown stability is propagated. An unknown segment is not converted into
stable or volatile, and an unknown boundary limits the leading-region
conclusion.

## Example

CONTEXT STABILITY

[system]
stable / persistent-versioned
4.2 KB

[source artifact]
stable / persistent-versioned
18.4 KB

[current task]
volatile / transient
0.8 KB

[tool definitions]
unknown
9.1 KB

FINDING

unknown tool stability; no ordering action

CACHE IMPACT

unknown
no runtime/provider evidence applied

ACTION

none
P0-L10 does not reorder context

The intended future sequence is:

P0-L10: classify stability
  -> future planner: consider semantic constraints, trust, runtime capability,
     and observed evidence
  -> only then propose safe optimisation

P0-L10 does not perform live inference, networking, runtime/model
installation, cache-hit/miss prediction, provider-specific simulation,
reordering, canonicalization, tool pruning, semantic compaction, cache
routing, KV quantisation, benchmark scoring, provider ranking, ContextBench
integration, or P0-L11 work. P0-L6 remains environment-blocked because no
existing usable llama-server or suitable GGUF is available.
