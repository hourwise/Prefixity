# dep-audit workspace

This local Rust workspace verifies that internal package constraints, a
protocol producer version, and declared Rust toolchains agree before a
release. The workspace contains seven functional local packages connected by
path dependencies. Its inventory command reads project manifests, the
compatibility contract, the lockfile, the exact toolchain pin, and the checked-
in CI matrix.

Run `python tools/dependency_inventory.py --root . --revision dep-audit-r1`
from this directory to produce the inventory carried by CP03. Run
`cargo test --workspace --offline --locked` after generating `Cargo.lock` to
exercise all seven local workspace packages without network access.
