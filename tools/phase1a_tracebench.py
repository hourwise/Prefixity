#!/usr/bin/env python3
"""Offline Phase 1A adapter for the pinned Tracebench mini-SWE slice.

This is deliberately a thin, standard-library-only adapter. It selects rows
from the verified manifest using metadata only, converts the mini-SWE-agent
message stream into hash-only Prefixity traces, and writes evaluation labels
to a separate file. The labels are never present in the decision-input trace.

The raw ``.tar.zst`` archives are intentionally not copied into the
repository. Extract them locally before running ``import``. The importer
accepts either the manifest's preserved ``source_relpath`` layout or a short
indexed layout (``raw-root/000/*.traj.json``, ``raw-root/001/*.traj.json``,
...) in selection-record order; the latter avoids Windows path-length limits.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import shutil
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Iterable


CORPUS = "Contextbench/Tracebench"
CORPUS_REVISION = "7da2e4f45b330be8b6e8f1cff835247723cb3341"
SPLIT = "verified"
TRACE_FORMAT_VERSION = 2


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            if not line.strip():
                continue
            try:
                value = json.loads(line)
            except json.JSONDecodeError as exc:
                raise SystemExit(f"{path}:{line_number}: invalid JSON: {exc}") from exc
            if not isinstance(value, dict):
                raise SystemExit(f"{path}:{line_number}: manifest row is not an object")
            rows.append(value)
    return rows


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        json.dump(value, handle, ensure_ascii=False, indent=2, sort_keys=True)
        handle.write("\n")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def length_band(index: int, population_count: int) -> str:
    """Return a deterministic short/medium/long rank band."""

    band = min(2, (index * 3) // population_count)
    return ("short", "medium", "long")[band]


def select_rows(
    rows: Iterable[dict[str, Any]], count_per_cell: int, excluded_ids: set[str] | None = None
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    excluded_ids = excluded_ids or set()
    population = [
        row
        for row in rows
        if row.get("agent") == "mini-SWE-agent"
        and row.get("artifact_path")
        and row.get("source_relpath")
        and row.get("traj_id") not in excluded_ids
    ]
    population.sort(key=lambda row: (int(row.get("step_count", 0)), str(row["traj_id"])))
    if not population:
        raise SystemExit("manifest has no artifact-bearing mini-SWE-agent rows")

    grouped: dict[tuple[bool, str], list[dict[str, Any]]] = defaultdict(list)
    for index, row in enumerate(population):
        solved = bool(row.get("solved"))
        grouped[(solved, length_band(index, len(population)))].append(row)

    selected: list[dict[str, Any]] = []
    cells: list[dict[str, Any]] = []
    for solved in (False, True):
        for band in ("short", "medium", "long"):
            cell = sorted(grouped.get((solved, band), []), key=lambda row: str(row["traj_id"]))
            if len(cell) < count_per_cell:
                raise SystemExit(
                    f"selection cell solved={solved} band={band} has {len(cell)} rows; "
                    f"need {count_per_cell}"
                )
            chosen = [cell[math.floor((offset + 0.5) * len(cell) / count_per_cell)] for offset in range(count_per_cell)]
            selected.extend(chosen)
            cells.append(
                {
                    "solved": solved,
                    "length_band": band,
                    "population_count": len(cell),
                    "selected_traj_ids": [row["traj_id"] for row in chosen],
                }
            )

    selected.sort(key=lambda row: str(row["traj_id"]))
    output = {
        "schema_version": 1,
        "corpus": CORPUS,
        "corpus_revision": CORPUS_REVISION,
        "split": SPLIT,
        "selection_method": {
            "description": (
                "Filter to artifact-bearing mini-SWE-agent rows, sort by "
                "(step_count, traj_id), assign equal-ranked short/medium/long "
                "bands, then choose count_per_cell evenly spaced rows in each "
                "solved x length-band cell. No trajectory contents or observer "
                "outputs are used."
            ),
            "agent_filter": "mini-SWE-agent",
            "count_per_cell": count_per_cell,
            "excluded_traj_ids": sorted(excluded_ids),
            "cell_order": [
                {"solved": solved, "length_band": band}
                for solved in (False, True)
                for band in ("short", "medium", "long")
            ],
        },
        "population_count": len(population),
        "selected_count": len(selected),
        "cells": cells,
        "records": [
            {
                "traj_id": row["traj_id"],
                "agent": row.get("agent"),
                "model": row.get("model"),
                "task_name": row.get("task_name"),
                "task_slug": row.get("task_slug"),
                "difficulty": row.get("difficulty"),
                "category": row.get("category"),
                "solved": bool(row.get("solved")),
                "step_count": row.get("step_count"),
                "stage_count": row.get("stage_count"),
                "source_relpath": row.get("source_relpath"),
                "artifact_path": row.get("artifact_path"),
                "length_band": next(
                    cell["length_band"]
                    for cell in cells
                    if row["traj_id"] in cell["selected_traj_ids"]
                ),
            }
            for row in selected
        ],
    }
    return selected, output


def find_trajectory_file(raw_root: Path, source_relpath: str, ordinal: int) -> Path:
    source_dir = raw_root.joinpath(*Path(source_relpath).parts)
    if source_dir.is_dir():
        candidates = sorted(source_dir.rglob("*.traj.json"))
    else:
        indexed_dir = raw_root / f"{ordinal:03d}"
        candidates = sorted(indexed_dir.rglob("*.traj.json")) if indexed_dir.is_dir() else []
    if len(candidates) != 1:
        raise SystemExit(
            f"expected exactly one mini-SWE .traj.json for selection ordinal {ordinal}, "
            f"found {len(candidates)} (checked {source_dir} and indexed layout)"
        )
    return candidates[0]


def archive_hash(archive_root: Path | None, artifact_path: str) -> str | None:
    if archive_root is None:
        return None
    archive = archive_root / Path(artifact_path).name
    if not archive.is_file():
        raise SystemExit(f"missing local archive for {artifact_path}: {archive}")
    return sha256_file(archive)


def json_text(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def classify_message(message: dict[str, Any], index: int) -> tuple[str, str]:
    role = str(message.get("role", "unknown"))
    content = message.get("content")
    content_text = content if isinstance(content, str) else json_text(content)
    if role == "system":
        return "system_policy", "system"
    if role == "assistant":
        return "conversation", "messages"
    if role == "user":
        # mini-SWE-agent records shell observations as user messages. The
        # markers are source-format syntax, not evaluation labels.
        if index > 0 and ("<returncode>" in content_text or "<output>" in content_text):
            return "tool_result", "messages"
        return "user_request", "messages"
    return "unknown", "other"


def evaluation_stage_summary(stages: Any) -> list[dict[str, Any]]:
    """Keep evaluation IDs/labels without copying action or observation text."""

    if not isinstance(stages, list):
        return []
    summaries: list[dict[str, Any]] = []
    for stage in stages:
        if not isinstance(stage, dict):
            continue
        step_summaries: list[dict[str, Any]] = []
        for step in stage.get("steps", []):
            if not isinstance(step, dict):
                continue
            step_summaries.append(
                {
                    "step_id": step.get("step_id"),
                    "labels": sorted(str(label) for label in step.get("labels", [])),
                }
            )
        summaries.append(
            {
                "stage_id": stage.get("stage_id"),
                "incorrect_step_ids": stage.get("incorrect_step_ids", []),
                "unuseful_step_ids": stage.get("unuseful_step_ids", []),
                "steps": step_summaries,
            }
        )
    return summaries


def make_trace(
    row: dict[str, Any],
    messages: list[dict[str, Any]],
    source_indices: list[int],
    trajectory_path: Path,
    raw_root: Path,
    archive_sha256: str | None,
    turn_index: int | None = None,
) -> tuple[dict[str, Any], list[dict[str, Any]], dict[str, Any]]:
    if not messages or len(messages) != len(source_indices):
        raise SystemExit(f"{trajectory_path}: expected a non-empty messages/source-index pair")

    blocks: list[dict[str, Any]] = []
    source_events: list[dict[str, Any]] = []
    source_file_sha256 = sha256_file(trajectory_path)
    role_counts: Counter[str] = Counter()
    source_counts: Counter[str] = Counter()
    content_hashes: Counter[str] = Counter()

    for position, (index, message) in enumerate(zip(source_indices, messages)):
        if not isinstance(message, dict):
            raise SystemExit(f"{trajectory_path}: messages[{index}] is not an object")
        source, zone = classify_message(message, index)
        role = str(message.get("role", "unknown"))
        event_text = json_text(message)
        event_bytes = event_text.encode("utf-8")
        content_hash = hashlib.sha256(event_bytes).hexdigest()
        token_estimate = max(1, math.ceil(len(event_text) / 4))
        event_id = f"message-{index:04d}"
        role_counts[role] += 1
        source_counts[source] += 1
        content_hashes[content_hash] += 1

        blocks.append(
            {
                "id": event_id,
                "source": source,
                "position": position,
                "content_hash": content_hash,
                "token_count": token_estimate,
                "byte_count": len(event_bytes),
                "semantic_zone": zone,
                "structural_path": f"messages[{index}]",
                "role": role,
                "dependencies": [],
                "optional": False,
                "required": False,
                "stale": False,
                "metadata": {
                    "corpus": CORPUS,
                    "corpus_revision": CORPUS_REVISION,
                    "trajectory_id": row["traj_id"],
                    "source_event_index": index,
                    "source_event_id": event_id,
                    "source_event_content_hash": content_hash,
                    "source_event_path": str(trajectory_path.relative_to(raw_root)).replace("\\", "/"),
                    "content_retained": False,
                    "evaluation_labels_excluded": True,
                    "token_count_method": "surrogate ceil(canonical_event_chars / 4); not provider tokens",
                },
            }
        )
        source_events.append(
            {
                "trajectory_id": row["traj_id"],
                "source_event_index": index,
                "source_event_id": event_id,
                "role": role,
                "source": source,
                "semantic_zone": zone,
                "structural_path": f"messages[{index}]",
                "content_hash": content_hash,
                "byte_count": len(event_bytes),
                "content_retained": False,
                "source_file_sha256": source_file_sha256,
            }
        )

    repeated_event_count = sum(count - 1 for count in content_hashes.values() if count > 1)
    trace = {
        "format_version": TRACE_FORMAT_VERSION,
        "request_id": (
            f"phase1a-{row['traj_id']}-turn-{turn_index:04d}"
            if turn_index is not None
            else f"phase1a-{row['traj_id']}"
        ),
        "session_id": row["traj_id"],
        "provider": "recorded-corpus",
        "model": row.get("model") or "unknown-recorded-model",
        "blocks": blocks,
        "metadata": {
            "corpus": CORPUS,
            "corpus_revision": CORPUS_REVISION,
            "split": SPLIT,
            "trajectory_id": row["traj_id"],
            "turn_index": turn_index,
            "request_context_end_event_index": source_indices[-1],
            "task_name": row.get("task_name"),
            "task_slug": row.get("task_slug"),
            "agent": row.get("agent"),
            "source_relpath": row.get("source_relpath"),
            "source_event_file": str(trajectory_path.relative_to(raw_root)).replace("\\", "/"),
            "source_file_sha256": source_file_sha256,
            "source_archive_sha256": archive_sha256,
            "transformation": "mini-SWE messages -> ordered Prefixity blocks; content hashed and omitted",
            "evaluation_labels_excluded": True,
            "evaluation_labels_location": "../evaluation/labels.json",
            "token_count_method": "surrogate ceil(canonical_event_chars / 4); not provider tokens",
        },
    }
    summary = {
        "trajectory_id": row["traj_id"],
        "task_name": row.get("task_name"),
        "agent": row.get("agent"),
        "model": row.get("model"),
        "step_count_manifest": row.get("step_count"),
        "message_count": len(messages),
        "turn_index": turn_index,
        "role_counts": dict(sorted(role_counts.items())),
        "source_counts": dict(sorted(source_counts.items())),
        "repeated_event_count": repeated_event_count,
        "source_file_sha256": source_file_sha256,
        "source_archive_sha256": archive_sha256,
        "evaluation_labels_excluded": True,
    }
    return trace, source_events, summary


def import_rows(args: argparse.Namespace) -> None:
    selection = json.loads(args.selection.read_text(encoding="utf-8"))
    if selection.get("corpus_revision") != CORPUS_REVISION:
        raise SystemExit("selection corpus revision does not match adapter pin")
    raw_root = args.raw_root.resolve()
    out_dir = args.out_dir
    if out_dir.exists():
        if not args.replace:
            raise SystemExit(f"output exists; pass --replace to replace: {out_dir}")
        shutil.rmtree(out_dir)
    (out_dir / "traces").mkdir(parents=True)
    (out_dir / "provenance").mkdir()
    (out_dir / "evaluation").mkdir()

    manifest_rows = {row["traj_id"]: row for row in read_jsonl(args.manifest)}
    summaries: list[dict[str, Any]] = []
    labels: list[dict[str, Any]] = []
    all_events: list[dict[str, Any]] = []
    for ordinal, record in enumerate(selection["records"]):
        row = manifest_rows[record["traj_id"]]
        trajectory_path = find_trajectory_file(raw_root, row["source_relpath"], ordinal)
        trajectory = json.loads(trajectory_path.read_text(encoding="utf-8"))
        archive_sha = archive_hash(args.archive_root, row["artifact_path"])
        messages = trajectory.get("messages")
        if not isinstance(messages, list) or not messages:
            raise SystemExit(f"{trajectory_path}: expected a non-empty messages array")
        if any(not isinstance(message, dict) for message in messages):
            raise SystemExit(f"{trajectory_path}: every messages entry must be an object")

        # Keep the complete source-event ledger for provenance, but give the
        # observer one request context per recorded assistant turn. The
        # assistant response itself is excluded from its request context.
        _, events, summary = make_trace(
            row, messages, list(range(len(messages))), trajectory_path, raw_root, archive_sha
        )
        turn_summaries: list[dict[str, Any]] = []
        assistant_turn = 0
        for response_index, message in enumerate(messages):
            if message.get("role") != "assistant" or response_index == 0:
                continue
            context_indices = list(range(response_index))
            trace, _, turn_summary = make_trace(
                row,
                [messages[index] for index in context_indices],
                context_indices,
                trajectory_path,
                raw_root,
                archive_sha,
                turn_index=assistant_turn,
            )
            write_json(
                out_dir / "traces" / row["traj_id"] / f"turn-{assistant_turn:04d}.json", trace
            )
            turn_summaries.append(turn_summary)
            assistant_turn += 1
        summary["turn_count"] = len(turn_summaries)
        summary["turn_summaries"] = turn_summaries
        summaries.append(summary)
        all_events.extend(events)
        labels.append(
            {
                "trajectory_id": row["traj_id"],
                "task_name": row.get("task_name"),
                "solved": bool(row.get("solved")),
                "incorrect_stages": evaluation_stage_summary(row.get("incorrect_stages", [])),
                "stage_count": row.get("stage_count"),
                "step_count": row.get("step_count"),
                "label_source": "manifest; evaluation-only; not included in trace input",
            }
        )

    write_json(out_dir / "selection.json", selection)
    write_json(out_dir / "evaluation" / "labels.json", {"schema_version": 1, "records": labels})
    write_json(out_dir / "provenance" / "trajectory-summaries.json", {"records": summaries})
    with (out_dir / "provenance" / "source-events.jsonl").open("w", encoding="utf-8", newline="\n") as handle:
        for event in all_events:
            handle.write(json.dumps(event, ensure_ascii=False, sort_keys=True) + "\n")
    write_json(
        out_dir / "import-report.json",
        {
            "schema_version": 1,
            "corpus": CORPUS,
            "corpus_revision": CORPUS_REVISION,
            "split": SPLIT,
            "trajectory_count": len(summaries),
            "turn_count": sum(summary["turn_count"] for summary in summaries),
            "source_event_count": len(all_events),
            "content_retained": False,
            "evaluation_labels_in_decision_inputs": False,
            "observer_input_glob": "traces/**/*.json",
            "token_counts_are_surrogates": True,
        },
    )
    print(json.dumps({"ok": True, "trajectory_count": len(summaries), "source_event_count": len(all_events)}))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    select = subparsers.add_parser("select", help="select a deterministic verified-split slice")
    select.add_argument("--manifest", type=Path, required=True)
    select.add_argument("--out", type=Path, required=True)
    select.add_argument("--count-per-cell", type=int, default=4)
    select.add_argument(
        "--exclude-traj-id",
        action="append",
        default=[],
        help="exclude a row whose pinned artifact was preflighted as missing a .traj.json",
    )

    importer = subparsers.add_parser("import", help="import extracted mini-SWE trajectories")
    importer.add_argument("--manifest", type=Path, required=True)
    importer.add_argument("--selection", type=Path, required=True)
    importer.add_argument("--raw-root", type=Path, required=True)
    importer.add_argument("--archive-root", type=Path)
    importer.add_argument("--out-dir", type=Path, required=True)
    importer.add_argument("--replace", action="store_true")

    args = parser.parse_args()
    if args.command == "select":
        selected, output = select_rows(
            read_jsonl(args.manifest), args.count_per_cell, set(args.exclude_traj_id)
        )
        if len(selected) < 20 or len(selected) > 50:
            raise SystemExit(f"selected slice has {len(selected)} rows; expected 20-50")
        write_json(args.out, output)
        print(json.dumps({"ok": True, "population_count": output["population_count"], "selected_count": output["selected_count"]}))
    else:
        import_rows(args)


if __name__ == "__main__":
    main()
