# P0-L11 — Safe Context Layout Candidate Planner

P0-L11 consumes a neutral P0-L4 ConformanceRequest and its P0-L10
ContextStabilityAnalysis to propose a small, bounded set of alternative
layouts. It answers:

> Is there a safer alternative ordering candidate under known constraints?

It does not answer whether a candidate improves real cache performance. That
requires later runtime evidence.

## Planner boundary

The planner is proposal-only. It does not mutate or execute the source
request, expose an automatically applicable request, predict cache reuse,
estimate token savings, or report latency, cost, TTFT, or model-quality
effects. A candidate is described as ordering_safe_under_declared_constraints,
not as semantically identical.

The first implementation deliberately uses the existing request shape. Only
already represented context artifacts may be reordered. System instructions,
current-user content, and tool definitions remain in their existing request
slots. Tool schemas, artifact contents, conversation turns, and tool-call or
result relationships are not rewritten or reordered.

## Constraints and safety

Movement permission must be explicit. MovableWithinCompatibleRegion names the
artifact and compatible region. If permission, ordering, semantic dependency,
chronology, or trust is unknown, the candidate is rejected or marked
unknown_not_provable.

The constraint vocabulary represents:

- must_precede and must_follow;
- fixed_position;
- preserve_relative_order with chronology, semantic-dependency, source-
  authority, tool-call/result, continuation, or other reasons;
- movable_within_compatible_region; and
- unknown.

Trust is evaluated independently of stability and lifecycle. A stable
untrusted artifact cannot be promoted ahead of trusted material. Unknown
trust does not become safe by implication. Lifecycle is retained in the
P0-L10 analysis and candidate result; persistent or transient status is not a
placement heuristic.

## Candidate generation

The planner scans P0-L10 boundaries for known stability inversions. For each
inversion it considers an adjacent artifact swap and one region-local move.
It checks movement permission, fixed positions, relative ordering, semantic
and chronology constraints, trust order, and whether the resulting layout has
a neutral structural benefit. It stops at eight proposed candidates and keeps
at most 32 bounded rejection records. It does not search all permutations.

Candidates are sorted deterministically by:

1. lower candidate inversion count;
2. longer stability-aligned leading region;
3. fewer unknown boundaries;
4. fewer moved segments;
5. fewer changed relative relationships; and
6. candidate layout fingerprint.

Equivalent final layouts are emitted once. The fingerprint covers ordered
roles, component identities, and content fingerprints, but not provenance or
timestamps. A candidate is re-analysed through P0-L10 and receives a P0-L7
RequestDiff. Every candidate diff retains cache_impact: unknown.

## Example: safe proposal

CURRENT

system            stable
task metadata     volatile
stable artifact   stable
current user      volatile

FINDING

1 stability inversion

SAFE MOVE ANALYSIS

stable artifact may move before task metadata
declared ordering permits move
trust boundary preserved
chronology preserved

CANDIDATE

system
stable artifact
task metadata
current user

STRUCTURAL RESULT

inversions: 1 -> 0
stable-leading units: 1 -> 2

CACHE IMPACT

unknown

RUNTIME EVIDENCE

none

ACTION

proposal only
request not modified

## Example: refusal

candidate rejected:
would move untrusted content across trusted instruction boundary

Other bounded rejection reasons include ordering constraint, semantic
dependency, unknown move safety, fixed segment, chronology, no structural
benefit, duplicate candidate, unsupported request region, and candidate limit.

## Evidence and future boundary

P0-L11 creates no synthetic telemetry and no ObservationComparison. No
candidate is experimentally validated. A later evaluator may combine:

P0-L11 candidate
  + P0-L9 runtime capabilities
  + P0-L6/P0-L8 observations

to assess empirical value. A still later stage may apply an explicitly
approved candidate. Neither evaluation nor request application is implemented
here. ContextBench remains pending, and P0-L6 remains environment-blocked
because no existing usable llama-server or suitable GGUF is available.
