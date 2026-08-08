#!/usr/bin/env python3
"""Run Prefixity's existing observer over an imported Phase 1A slice."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


def run(binary: Path, command: str, trace: Path) -> dict[str, Any]:
    completed = subprocess.run(
        [str(binary), "--json", command, str(trace)],
        capture_output=True,
        text=True,
        check=False,
    )
    stdout = completed.stdout.strip()
    try:
        result: Any = json.loads(stdout)
    except json.JSONDecodeError:
        result = {"raw_stdout": stdout}
    return {
        "trace": str(trace).replace("\\", "/"),
        "command": command,
        "exit_code": completed.returncode,
        "stderr": completed.stderr.strip(),
        "result": result,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--trace-root", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()

    traces = sorted(args.trace_root.rglob("*.json"))
    if not traces:
        raise SystemExit(f"no trace JSON files under {args.trace_root}")
    args.out_dir.mkdir(parents=True, exist_ok=True)
    validations = [run(args.binary, "validate", trace) for trace in traces]
    analyses = [run(args.binary, "analyse", trace) for trace in traces]
    (args.out_dir / "validation.json").write_text(
        json.dumps(validations, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (args.out_dir / "analyses.json").write_text(
        json.dumps(analyses, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(
        json.dumps(
            {
                "trace_count": len(traces),
                "validation_ok": sum(item["exit_code"] == 0 for item in validations),
                "analysis_ok": sum(item["exit_code"] == 0 for item in analyses),
            }
        )
    )


if __name__ == "__main__":
    main()
