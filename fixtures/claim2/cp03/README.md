# CP03: dependency and build metadata reconciliation

This case freezes a small, self-authored Rust workspace under `snapshot/` at
revision `dep-audit-r1`. The complete local project contains seven functional
workspace packages, their path dependencies, build metadata, the toolchain
pin, a CI toolchain matrix, a compatibility declaration, and a lockfile.
`snapshot/tools/dependency_inventory.py` reads those exact files with Python's
standard-library TOML parser and writes the deterministic native inventory in
`bodies/inventory.txt`; the inventory is therefore an export of project data,
not a hand-authored table.

Request 1 presents the inventory action. After its receipt, request 2 retains
request 1's messages unchanged, appends the native inventory, then presents
the snapshot and dependency verification menu. After its receipt, request 3
retains all prior messages unchanged and appends the byte-identical inventory
reemission beside the final question. The two inventory Message events share
revision `dep-audit-r1`, tree digest, and producer query identity. The
verification remains its own Result event.

The hidden evaluation key records required versions, compatibility edges,
toolchain facts, unchanged-state identity, and the final reconciliation. Tests
recompute the workspace facts from the pinned files, check the inventory
against them, exercise exact policy selection and arm evaluation, and confirm
that the Result receipt stays distinct. No model outputs are stored here.

Text files use UTF-8 and LF. `snapshot/SHA256SUMS` pins every project input
file, including the inventory producer. The case manifest separately pins
all prompt, attachment, receipt, and native source bodies.
