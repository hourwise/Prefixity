#!/usr/bin/env python3
"""Emit a deterministic symbol and qualified-call inventory for this snapshot."""

from __future__ import annotations

import argparse
import hashlib
import re
from pathlib import Path

DECLARATION = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?"
    r"(?:fn|struct|enum|trait|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
IMPL = re.compile(r"^\s*impl(?:<[^>]+>)?\s+(?:([A-Za-z_][A-Za-z0-9_:]*)\s+for\s+)?([A-Za-z_][A-Za-z0-9_:]*)")
QUALIFIED_CALL = re.compile(r"\b([A-Z][A-Za-z0-9_]*)::([a-z_][A-Za-z0-9_]*)\s*\(")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--query", default="symbol-map-v1")
    args = parser.parse_args()
    root = args.root.resolve()
    files = sorted((root / "src").rglob("*.rs"), key=lambda path: path.relative_to(root).as_posix())
    digest = hashlib.sha256()
    rows: list[str] = []
    for path in files:
        relative = path.relative_to(root).as_posix()
        data = path.read_bytes()
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update(data)
        digest.update(b"\0")
        text = data.decode("utf-8")
        current_impl: str | None = None
        for line_number, line in enumerate(text.splitlines(), 1):
            implementation = IMPL.match(line)
            if implementation:
                current_impl = implementation.group(2).split("::")[-1]
            declaration = DECLARATION.match(line)
            if declaration:
                name = declaration.group(1)
                if current_impl and re.search(r"\bfn\s+" + re.escape(name) + r"\b", line):
                    name = current_impl + "::" + name
                rows.append(f"{relative}:{line_number}: declaration: {name}")
            for receiver, method in QUALIFIED_CALL.findall(line):
                rows.append(f"{relative}:{line_number}: qualified-call: {receiver}::{method}")
    print("repository=quarry-ledger")
    print(f"revision={args.revision}")
    print(f"query_id={args.query}")
    print(f"source_files={len(files)}")
    print(f"source_sha256={digest.hexdigest()}")
    print("entries:")
    print("\n".join(rows))


if __name__ == "__main__":
    main()
