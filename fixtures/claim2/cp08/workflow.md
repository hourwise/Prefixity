# CP08 synthetic cold-chain lot disposition workflow, version 1

This fixture is wholly synthetic. `LOT-SYN-CC08-117`, `PRD-SYN-RX-44`,
`LOGGER-SYN-14`, and every supplier, site, certificate, and operator reference
are fictional. The deterministic builder creates a frozen receiving record,
logger intervals, calibration record, ordinary quality-review packet recipe,
receipts, trace, and isolated evaluation key. It performs no network, provider,
or model operation.

## Ordinary receiving and disposition sequence

At receipt of a temperature-controlled lot, the receiving technician opens
the currently effective cold-chain SOP beside the lot record, logger export,
and calibration certificate. The technician checks the lot identity and
continuous logger record, then verifies calibration and calculates total
excursion exposure from the unchanged records. Those checks are read-only and
do not edit, repair, or reclassify any source record.

When a disposition review is opened, the routine controlled-record packet
contains the frozen lot, logger, and calibration evidence plus a copy of the
effective SOP. Quality reviewers receive the governing procedure with the
decision record so that each hold, release, or escalation is traceable to the
rules in force. The packet recipe attaches one native SOP message for every
review; it does not inspect document size or use a benchmark condition. The
final SOP occurrence is generated from the same content-addressed procedure
revision as the receiving occurrence.

## Frozen source state

The SOP is `CC-SOP-04`, revision `3.2.1`, effective 2026-01-01. The frozen lot
record is `source/lot-snapshot-LOT-SYN-CC08-117-v1.json`, revision
`cold-chain-lot@LOT-SYN-CC08-117-v1`. It names the logger and calibration
record and freezes the receiving interval. The logger export is
`source/logger-record-LOGGER-SYN-14-v1.json`, revision
`cold-chain-logger@LOGGER-SYN-14-v1`; the certificate is
`source/calibration-CAL-SYN-14-2026-v1.json`, revision
`logger-calibration@CAL-SYN-14-2026-v1`. All three remain read-only under
world-state revision `cold-chain-disposition@LOT-SYN-CC08-117-v1`.

The original procedure occurrence is `e-sop-original`; the final packet
occurrence is `e-sop-final-reattachment`. Their UTF-8 body bytes, SOP source
revision, and world-state revision are identical. The original remains in the
chronological input. The later native user Message has no consumers and no
protected relation. The ordinary packet recipe records the unconditional
attachment rule and the procedure hash.

## Deterministic checks

Request 1 offers `inspect_lot_logger_state` and
`verify_receiving_record_completeness`. The first checks lot identity, the
logger identity, interval coverage, observed extrema, and the frozen source
hashes. Request 2 offers `calculate_calibration_and_exposure` and
`verify_calibration_certificate`. The expected calculation checks the
calibration correction, inclusive storage band, excursion interval duration,
and cumulative temperature-degree-minutes using the published SOP method.
Each action creates a distinct deterministic receipt tied to its action and
source revision; no receipt mutates the lot, logger, certificate, or SOP.

The final answer is exact JSON: disposition, rule IDs, corrected interval
extrema, interval and cumulative excursion arithmetic, calibration validity,
and the required hold or escalation reason. Its evaluation key is stored
separately and is never a prompt asset or planner-visible event. The public
source documents and check receipts define the answer independently.

## Provenance limits

The records describe a closed synthetic quality system, not a real facility,
product, shipment, logger, certificate, or release decision. The repeated SOP
is part of the ordinary final quality packet defined above. No size-derived
criterion or tokenizer result was used to compose, select, or repeat it.
