# CP10 synthetic observation publication workflow, version 1

This closed benchmark world contains twelve generated readings from two fictional
laboratory instruments. It contains no real people, facilities, or locations.
All source records are immutable and all actions are read-only.

## Governing specification and completeness

`source/field-dictionary-aggregation-v1.json` is the complete versioned
publication specification. It defines all input fields, units, null handling,
quality flags, timestamp normalization, UTC grouping, output columns,
integer rounding, and quality dispositions. It identifies the three and only
three source shards in this frozen publication collection. The specification
is complete because the declared schema accounts for every field in each
shard and the output recipe resolves every valid observation using only those
fields and rules. No additional source, inferred field, or narrative policy is
needed.

## Ordinary publication sequence

The publication desk first attaches the governing dictionary and the complete
source collection. A catalog inspection records all shard IDs, content hashes,
and row counts. The second desk step evaluates nulls, source quality flags,
timestamp offsets, and coverage against the same immutable collection. Neither
step changes any source record.

When the standard publication packet is assembled, its cover sheet carries
the catalog and quality receipts and attaches the effective field dictionary
again so that the publication record travels with the exact schema used to
interpret its rows. The packet recipe emits one native specification message
for every publication, regardless of document size, arm, or benchmark state.
The later occurrence resolves to the same content-addressed specification
revision as the original. It does not transform or summarize that object.

## Deterministic checks and publication

The finite first-slot action menu offers catalog inspection or a shard-count
cross-check. The second-slot menu offers quality/gap inspection or timezone
and unit verification. Each action has a fixed receipt bound to its input
hashes and the unchanged publication world revision.

The final rollup normalizes RFC3339 offsets to UTC, excludes a null measurement
under `QF-01`, excludes every non-`VALID` quality flag under `QF-02`, and groups
remaining integer tenths of Celsius by instrument and UTC date. Means use
round-half-up integer arithmetic. Excluded observations remain visible in the
flag list. The hidden evaluator checks the exact rows, flags, source IDs,
quality rules, publication status, and reason. Its key is stored outside every
prompt and planner-visible asset.

## Provenance limits

The generator uses fixed synthetic inputs and performs no network, tokenizer,
model, or inference operation. This workflow defines its bounded records and
ordinary packet behavior independently of any byte or token count. No size
derived admission claim is made.
