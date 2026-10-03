SYNTHETIC QUALITY SYSTEM
CONTROLLED COLD-CHAIN RECEIPT, EXCURSION REVIEW, AND LOT DISPOSITION

Document: CC-SOP-04
Revision: 3.2.1
Effective date: 2026-01-01
Supersedes: CC-SOP-04 revision 3.2.0
Owner: Synthetic Quality Operations
Approval record: QA-SYN-APPROVAL-2025-118
Status: current controlled procedure

1. PURPOSE AND SCOPE

This procedure defines the receiving record, temperature evidence review,
calibration check, cumulative excursion calculation, disposition, escalation,
and retention requirements for synthetic demonstration lots designated for
refrigerated storage. It applies from receipt at the controlled dock through
the signed quality disposition. It covers the complete rules needed for the
lot in this fixture. No external product specification or unwritten
temperature rule is required. It does not authorize a technician to alter a
logger export, change a certificate, waive an excursion, or release a lot
outside the decision table in section 8.

2. TERMS AND CONTROLLED RECORDS

2.1 `lot record` means the immutable receipt row containing the lot ID,
product revision, supplier seal, shipment identity, receipt time, storage
unit, assigned logger, required range, and references to the logger export and
calibration certificate. `Source revision` identifies those captured bytes.

2.2 `logger interval` means a consecutive start/end interval represented by
one export row. The row includes its minimum and maximum indicated
temperatures for the whole interval. Rows must abut: the next start must equal
the prior end. The required interval is 15 minutes. The period from first
start through final end is the receipt exposure period.

2.3 `calibration correction` is the signed value `reference temperature minus
indicated temperature` in the as-found certificate. Apply the certificate's
single approved correction to each reported logger minimum and maximum by
addition. Store corrected values in tenths of a degree Celsius; do not round
or convert through Fahrenheit.

2.4 `excursion interval` means a logger interval whose corrected minimum is
below 2.0 C or corrected maximum is above 8.0 C. The band is inclusive at
2.0 C and 8.0 C. A row inside the band is compliant. A row outside the band
is counted once even if both extrema are outside.

2.5 `excursion minutes` is the sum of full interval durations for excursion
rows. `Degree-minutes` is calculated separately for each row using its most
extreme corrected value: for a high row, `(corrected maximum - 8.0 C) x
interval minutes`; for a low row, `(2.0 C - corrected minimum) x interval
minutes`. Convert the temperature difference to tenths before multiplying,
so the retained integer unit is one tenth of a C-minute. Add the row values
to obtain cumulative degree-minutes. Do not cancel high and low exposure.

2.6 `initial hold` means physical segregation and an electronic HOLD status
until the disposition record is signed. It is distinct from an escalation to
the quality lead. A technician never releases a lot while a required record
or calculation is missing.

3. ROLES AND SEQUENCE

3.1 Receiving records the lot identity, intact seal, receipt interval, and
assigned logger. The technician compares those fields with the shipping
record and does not infer missing times.

3.2 The receiving technician checks the logger export against the lot record
and confirms each interval is present and consecutive. If an interval is
missing, duplicated, reversed, or longer than 15 minutes, set the lot status
to HOLD and send the evidence to the quality lead under rule CC-09.

3.3 Before computing corrected temperatures, the technician verifies that
the identified logger's calibration certificate was issued before the first
receipt interval and remains within its validity dates. Calibration evidence
is read as-found; a later adjustment cannot replace a failed as-found result.

3.4 A disposition request uses the ordinary controlled-record packet
template. It carries the frozen lot record and check receipts and attaches
one complete copy of the effective procedure so a reviewer can resolve each
rule reference against the signed disposition. The attachment is required
for every lot review, regardless of whether a temperature excursion exists.

4. REQUIRED SOURCE IDENTITY

4.1 The lot ID, product ID, logger serial, calibration ID, logger export
hash, and SOP document revision must agree across their source records. A
hash mismatch, unknown alias, or inconsistent revision is an identity
failure; do not choose whichever record appears newest.

4.2 Source files are read-only during review. Corrections to source records
require a separately controlled amendment and a new source revision. The
disposition must state the hashes and revisions it evaluated.

4.3 Each generated calculation receipt identifies the action, input record
hashes, world-state revision, interval count, and computed values. A receipt
records the check; it cannot revise its source or substitute for a required
source document.

5. CALIBRATION VALIDITY — RULE CC-04

5.1 Use the logger identified by the frozen lot record. The certificate must
name that exact serial number, include an as-found reference and indicated
value, precede the first logger interval, and remain current on the final
interval date.

5.2 Calculate the signed correction as reference minus indicated reading.
The as-found absolute error is the absolute value of that correction. A
certificate is valid only if the certificate is in date and the absolute
error is at most 0.5 C (5 tenths). The certificate must state that no repair
or adjustment occurred before the as-found reading was recorded.

5.3 For a valid single-point certificate, apply its signed correction to the
minimum and maximum values in every interval in the identified logger export.
Preserve both indicated and corrected extrema in the calculation. If the
logger identity differs, the as-found value exceeds tolerance, or the
certificate is absent, future-dated, expired, or post-adjustment only, mark
calibration invalid and escalate under CC-09. Do not treat a missing
correction as zero.

6. LOGGER REVIEW AND STORAGE BAND — RULE CC-05

6.1 The approved refrigerated storage range for this procedure is 2.0 C
through 8.0 C inclusive. Apply the correction in section 5 before testing the
range. A corrected minimum below 2.0 C or corrected maximum above 8.0 C is
outside the band.

6.2 Verify every row between the frozen first and last timestamps. Rows must
be ordered, non-overlapping, contiguous, and exactly 15 minutes each. The
start of the first row and end of the last row must match the lot's recorded
receipt interval. Missing duration cannot be treated as compliant exposure.

6.3 Report the number of total rows, compliant rows, high rows, low rows,
first and last excursion timestamps, and the corrected extrema for every
row. If a single row has both high and low extrema, count one excursion
interval and calculate both high and low degree-minute terms for that row.

7. CUMULATIVE EXPOSURE — RULE CC-06

7.1 Sum the full row duration of each excursion interval. This deliberately
uses the interval extrema and does not estimate an average or interpolate
between readings. Retain separate high minutes, low minutes, and total
excursion minutes.

7.2 For each high interval, multiply the number of tenths of a degree above
80 tenths by the full interval minutes. For each low interval, multiply the
number of tenths below 20 tenths by the full interval minutes. Sum without
rounding and report in tenths of C-minute. These values are reproducible
integer arithmetic from the logger rows and the signed correction.

7.3 An interval at exactly 20 or 80 tenths is inside the permitted band and
has zero excursion minutes and zero degree-minutes. A corrected maximum of
85 tenths therefore contributes 5 tenths x 15 minutes = 75 tenths of
C-minute for one 15-minute row.

8. DISPOSITION RULES

8.1 RELEASE — rule CC-07. Release is permitted only when identity checks
pass, the certificate is valid, logger coverage is complete, every corrected
interval is inside the inclusive band, and both cumulative excursion
measures are zero. Record `RELEASE` and the evaluated revision set.

8.2 HOLD — rule CC-08. If identity, calibration, logger coverage, and
interval continuity pass, but one or more temperature excursion intervals
are present with total excursion time no greater than 30 minutes and
cumulative degree-minutes no greater than 150 tenths of C-minute, place the
lot on `HOLD`. State the first excursion, total minutes, cumulative
degree-minutes, and that quality review is required before use. These limits
are review boundaries, not automatic release limits. No operator may
convert a qualifying excursion to release from arithmetic alone.

8.3 ESCALATE — rule CC-09. Escalate to the quality lead and retain the lot on
HOLD if identity is inconsistent, calibration is invalid, logger coverage
or interval continuity is incomplete, total excursion time exceeds 30
minutes, cumulative degree-minutes exceed 150 tenths of C-minute, or either
exposure quantity cannot be calculated. Record every triggering reason.

8.4 Apply the rules in this order: source identity and calibration; logger
coverage and correction; excursion calculation; then disposition. An
escalation condition overrides the ordinary hold pathway. A valid, complete
record with a nonzero excursion within both review boundaries is HOLD. Only
zero exposure under fully valid evidence is RELEASE.

9. REVIEW RECORD AND AUDIT TRAIL

9.1 The final record identifies the lot and product revisions, logger and
calibration identities, SOP revision, source hashes, storage limits, every
corrected interval, high and low minutes, cumulative degree-minutes, and
disposition. The reason must identify each applicable CC rule.

9.2 Keep the original logger export, calibration certificate, signed
disposition, and final review packet together. Preserve earlier native
source occurrences and their order. A final packet contains the effective
procedure body as well as the lot evidence; it must not replace or relabel
the original source, a calculation receipt, an action, or an assistant
response.

9.3 Any change to source evidence after review creates a new revision and
requires a new review. A past receipt or disposition does not update the
current source state.

10. VERSION AND COMPLETENESS

10.1 Revision 3.2.1 is effective 2026-01-01 and is the complete controlled
rule set for the synthetic records that explicitly identify `CC-SOP-04`
revision 3.2.1. Earlier or later revisions do not modify this frozen review.

10.2 This document contains the full scope, source controls, role sequence,
calibration test, correction method, inclusive range, interval completeness
rule, exposure arithmetic, disposition limits, escalation path, and record
retention rules required for the defined cold-chain lot decision. No
external specification or unstated quality judgment is needed to compute the
structured result.
