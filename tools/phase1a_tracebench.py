#!/usr/bin/env python3
"""Offline Phase 1A adapter for a pinned mini-SWE trajectory slice.

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


CORPUS = "NJU-LINK/CodeTraceBench"
CORPUS_REVISION = "aa213b84ffb6690fc37ca15766d6ca174ec36d4d"
SPLIT = "verified"
TRACE_FORMAT_VERSION = 2
EVIDENCE_SCHEMA_VERSION = 1

PROVIDER_USAGE_SCHEMAS = {
    "anthropic": "codetracebench-anthropic-chat-completions-v1",
    "deepseek": "codetracebench-deepseek-chat-completions-v1",
    "openai": "openai-chat-completions-v1",
}


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
    rows: Iterable[dict[str, Any]],
    count_per_cell: int,
    excluded_ids: set[str] | None = None,
    corpus: str = CORPUS,
    corpus_revision: str = CORPUS_REVISION,
    split: str = SPLIT,
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
        "corpus": corpus,
        "corpus_revision": corpus_revision,
        "split": split,
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


def provider_identity(model: Any) -> str:
    prefix = str(model or "").split("/", 1)[0].strip().lower()
    return prefix if prefix in PROVIDER_USAGE_SCHEMAS else "recorded-corpus"


def provider_usage_schema(provider: str) -> str:
    return PROVIDER_USAGE_SCHEMAS.get(
        provider, "codetracebench-recorded-response-v1"
    )


def finite_timestamp(message: dict[str, Any], index: int) -> int | float:
    value = message.get("timestamp")
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise SystemExit(f"messages[{index}].timestamp must be a finite number")
    return value


def source_locator(
    trajectory_id: str,
    source_file_sha256: str,
    source_event_index: int | None,
    source_event_id: str | None,
    upstream_field_path: str | None = None,
) -> dict[str, Any]:
    return {
        "trajectory_id": trajectory_id,
        "source_file_sha256": source_file_sha256,
        "source_event_index": source_event_index,
        "source_event_id": source_event_id,
        "upstream_field_path": upstream_field_path,
    }


def evidence_provenance(
    origin: str,
    locator: dict[str, Any] | None,
    derivation_rule: str | None = None,
    evaluation_only: bool = False,
) -> dict[str, Any]:
    value: dict[str, Any] = {"origin": origin}
    if locator is not None:
        value["source_locator"] = locator
    if derivation_rule is not None:
        value["derivation_rule"] = derivation_rule
    if evaluation_only:
        value["evaluation_only"] = True
    return value


def response_field_states(response_message: dict[str, Any]) -> dict[str, str]:
    states: dict[str, str] = {}
    for field in ("reasoning_content", "tool_calls", "function_call", "annotations", "refusal"):
        if field not in response_message:
            states[field] = "absent"
        elif response_message[field] is None:
            states[field] = "null"
        else:
            states[field] = "present"
    return states


def provider_response_metadata(response: dict[str, Any]) -> dict[str, Any]:
    choices = response.get("choices")
    choice = choices[0] if isinstance(choices, list) and choices and isinstance(choices[0], dict) else {}
    response_message = choice.get("message") if isinstance(choice.get("message"), dict) else {}
    created = response.get("created")
    if created is not None and (isinstance(created, bool) or not isinstance(created, int)):
        raise SystemExit("provider response created metadata must be an integer")
    choice_index = choice.get("index")
    if choice_index is not None and (isinstance(choice_index, bool) or not isinstance(choice_index, int)):
        raise SystemExit("provider response choice index must be an integer")
    response_id = response.get("id")
    response_model = response.get("model")
    if not isinstance(response_id, str) or not response_id:
        raise SystemExit("assistant response is missing explicit response.id")
    if not isinstance(response_model, str) or not response_model:
        raise SystemExit("assistant response is missing explicit response.model")
    return {
        "id": response_id,
        "model": response_model,
        "created": created,
        "object": response.get("object"),
        "choice_index": choice_index,
        "finish_reason": choice.get("finish_reason"),
        "response_message_role": response_message.get("role"),
        "field_states": response_field_states(response_message),
    }


def message_line_spans(path: Path) -> dict[int, tuple[int, int]]:
    """Return line spans for raw messages without retaining their content."""

    text = path.read_text(encoding="utf-8")
    decoder = json.JSONDecoder()

    def skip_whitespace(position: int) -> int:
        while position < len(text) and text[position] in " \t\r\n":
            position += 1
        return position

    def line_number(position: int) -> int:
        return text.count("\n", 0, position) + 1

    position = skip_whitespace(0)
    if position >= len(text) or text[position] != "{":
        raise SystemExit(f"{path}: raw trajectory root is not an object")
    position += 1
    while True:
        position = skip_whitespace(position)
        if text[position] == "}":
            break
        key, position = decoder.raw_decode(text, position)
        if not isinstance(key, str):
            raise SystemExit(f"{path}: raw trajectory object key is not a string")
        position = skip_whitespace(position)
        if position >= len(text) or text[position] != ":":
            raise SystemExit(f"{path}: malformed raw trajectory object")
        position = skip_whitespace(position + 1)
        if key == "messages":
            if position >= len(text) or text[position] != "[":
                raise SystemExit(f"{path}: messages is not an array")
            position += 1
            spans: dict[int, tuple[int, int]] = {}
            index = 0
            while True:
                position = skip_whitespace(position)
                if text[position] == "]":
                    return spans
                start = position
                _, end = decoder.raw_decode(text, position)
                spans[index] = (line_number(start), line_number(end - 1))
                index += 1
                position = skip_whitespace(end)
                if text[position] == ",":
                    position += 1
                elif text[position] != "]":
                    raise SystemExit(f"{path}: malformed messages array")
        _, position = decoder.raw_decode(text, position)
        position = skip_whitespace(position)
        if text[position] == ",":
            position += 1


def classify_message(message: dict[str, Any], index: int) -> tuple[str, str]:
    role = str(message.get("role", "unknown"))
    if role == "system":
        return "system_policy", "system"
    if role == "assistant":
        return "conversation", "messages"
    if role == "user":
        return "user_request", "messages"
    return "unknown", "other"


def evaluation_stage_summary(
    stages: Any,
    *,
    trajectory_id: str | None = None,
    source_file_sha256: str | None = None,
    source_file_name: str | None = None,
    message_spans: dict[int, tuple[int, int]] | None = None,
) -> list[dict[str, Any]]:
    """Keep evaluation labels and bounded explicit source locators only.

    The upstream reference ``content`` field is deliberately dropped. A
    locator joins to a raw message only when its explicit path and line range
    identify exactly one message object; no positional or adjacency fallback
    is permitted.
    """

    def sanitized_ref(ref: Any, field_name: str) -> dict[str, Any] | None:
        if not isinstance(ref, dict):
            return None
        path = ref.get("path")
        line_start = ref.get("line_start")
        line_end = ref.get("line_end")
        if (
            not isinstance(path, str)
            or not isinstance(line_start, int)
            or not isinstance(line_end, int)
            or isinstance(line_start, bool)
            or isinstance(line_end, bool)
        ):
            return None
        result: dict[str, Any] = {
            "path": path.replace("\\", "/"),
            "line_start": line_start,
            "line_end": line_end,
            "provenance": evidence_provenance(
                "source_explicit",
                source_locator(
                    trajectory_id or "",
                    source_file_sha256 or "",
                    None,
                    None,
                    f"evaluation.{field_name}_ref",
                ),
                evaluation_only=True,
            ),
        }
        if (
            message_spans is not None
            and source_file_name is not None
            and path.replace("\\", "/").rsplit("/", 1)[-1] == source_file_name
        ):
            matches = [
                index
                for index, (start, end) in message_spans.items()
                if line_start >= start and line_end <= end
            ]
            if len(matches) == 1:
                index = matches[0]
                event_id = f"message-{index:04d}"
                result["source_event_join"] = {
                    "status": "exact",
                    "source_event_index": index,
                    "source_event_id": event_id,
                    "provenance": evidence_provenance(
                        "derived_structural",
                        source_locator(
                            trajectory_id or "",
                            source_file_sha256 or "",
                            index,
                            event_id,
                            f"messages[{index}]",
                        ),
                        derivation_rule="codetracebench.evaluation_locator_to_message_span_v1",
                        evaluation_only=True,
                    ),
                }
            else:
                result["source_event_join"] = {
                    "status": "unresolved_no_unique_message_span",
                    "provenance": evidence_provenance(
                        "unknown",
                        None,
                        evaluation_only=True,
                    ),
                }
        else:
            result["source_event_join"] = {
                "status": "unresolved_no_explicit_source_match",
                "provenance": evidence_provenance(
                    "unknown",
                    None,
                    evaluation_only=True,
                ),
            }
        return result

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
            step_summaries[-1]["source_locators"] = {
                name: locator
                for name, locator in (
                    ("action", sanitized_ref(step.get("action_ref"), "action")),
                    ("observation", sanitized_ref(step.get("observation_ref"), "observation")),
                )
                if locator is not None
            }
            joins = [
                locator.get("source_event_join")
                for locator in step_summaries[-1]["source_locators"].values()
                if locator.get("source_event_join", {}).get("status") == "exact"
            ]
            step_summaries[-1]["source_event_join"] = {
                "status": "exact" if joins else "unresolved",
                "source_event_indices": sorted(
                    join["source_event_index"] for join in joins
                ),
                "provenance": evidence_provenance(
                    "derived_structural" if joins else "unknown",
                    None,
                    derivation_rule=(
                        "codetracebench.evaluation_locator_to_message_span_v1"
                        if joins
                        else None
                    ),
                    evaluation_only=True,
                ),
            }
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
    response_message: dict[str, Any] | None = None,
    response_index: int | None = None,
    corpus: str = CORPUS,
    corpus_revision: str = CORPUS_REVISION,
    split: str = SPLIT,
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
        timestamp = finite_timestamp(message, index)
        locator = source_locator(
            row["traj_id"],
            source_file_sha256,
            index,
            event_id,
            f"messages[{index}]",
        )
        block_provenance = {
            "id": evidence_provenance(
                "derived_structural", locator, "codetracebench.message_id_v1"
            ),
            "role": evidence_provenance(
                "source_explicit",
                {**locator, "upstream_field_path": f"messages[{index}].role"},
            ),
            "source": evidence_provenance(
                "derived_structural",
                locator,
                "codetracebench.role_to_source_v1",
            ),
            "semantic_zone": evidence_provenance(
                "derived_structural",
                locator,
                "codetracebench.role_to_zone_v1",
            ),
            "structural_path": evidence_provenance(
                "derived_structural",
                locator,
                "codetracebench.message_array_path_v1",
            ),
            "timestamp": evidence_provenance(
                "source_explicit",
                {**locator, "upstream_field_path": f"messages[{index}].timestamp"},
            ),
        }
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
                "timestamp": timestamp,
                "semantic_zone": zone,
                "structural_path": f"messages[{index}]",
                "role": role,
                "provenance": block_provenance,
                "metadata": {
                    "corpus": corpus,
                    "corpus_revision": corpus_revision,
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
                "timestamp": timestamp,
                "content_hash": content_hash,
                "byte_count": len(event_bytes),
                "content_retained": False,
                "source_file_sha256": source_file_sha256,
                "provenance": block_provenance,
            }
        )
        extra = message.get("extra")
        response = extra.get("response") if isinstance(extra, dict) else None
        if isinstance(response, dict):
            response_info = provider_response_metadata(response)
            source_events[-1]["provider_response_id"] = response_info["id"]
            source_events[-1]["provider_response_model"] = response_info["model"]
            source_events[-1]["provider_usage_schema"] = provider_usage_schema(
                provider_identity(row.get("model"))
            )

    repeated_event_count = sum(count - 1 for count in content_hashes.values() if count > 1)
    provider = provider_identity(row.get("model"))
    captured_response: dict[str, Any] | None = None
    captured_usage: dict[str, Any] | None = None
    trace_provenance: dict[str, Any] = {}
    if response_message is not None:
        extra = response_message.get("extra")
        response = extra.get("response") if isinstance(extra, dict) else None
        if not isinstance(response, dict):
            raise SystemExit(
                f"{trajectory_path}: assistant message {response_index} has no response envelope"
            )
        if not isinstance(response.get("usage"), dict):
            raise SystemExit(
                f"{trajectory_path}: assistant message {response_index} has no response usage object"
            )
        captured_response = provider_response_metadata(response)
        captured_usage = {
            "provider_schema": provider_usage_schema(provider),
            "raw": response["usage"],
        }
        response_event_id = f"message-{response_index:04d}"
        response_locator = source_locator(
            row["traj_id"],
            source_file_sha256,
            response_index,
            response_event_id,
            f"messages[{response_index}].extra.response",
        )
        trace_provenance = {
            "provider_response": evidence_provenance(
                "source_explicit", response_locator
            ),
            "usage": evidence_provenance(
                "source_explicit",
                {**response_locator, "upstream_field_path": f"messages[{response_index}].extra.response.usage"},
            ),
        }
    trace = {
        "format_version": TRACE_FORMAT_VERSION,
        "request_id": (
            f"phase1a-{row['traj_id']}-turn-{turn_index:04d}"
            if turn_index is not None
            else f"phase1a-{row['traj_id']}"
        ),
        "session_id": row["traj_id"],
        "provider": provider,
        "model": row.get("model") or "unknown-recorded-model",
        "evidence_schema_version": EVIDENCE_SCHEMA_VERSION,
        "blocks": blocks,
        "usage": captured_usage,
        "provider_response": captured_response,
        "provenance": trace_provenance,
        "metadata": {
            "corpus": corpus,
            "corpus_revision": corpus_revision,
            "split": split,
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
            "evidence_schema_version": EVIDENCE_SCHEMA_VERSION,
            "provider_usage_schema": captured_usage["provider_schema"] if captured_usage else None,
            "provider_response_id": captured_response["id"] if captured_response else None,
            "source_response_event_index": response_index,
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
    corpus = selection.get("corpus")
    corpus_revision = selection.get("corpus_revision")
    split = selection.get("split", SPLIT)
    if not corpus or not corpus_revision:
        raise SystemExit("selection must identify corpus and corpus revision")
    if args.corpus and args.corpus != corpus:
        raise SystemExit("requested corpus does not match selection corpus")
    if args.corpus_revision and args.corpus_revision != corpus_revision:
        raise SystemExit("requested corpus revision does not match selection revision")
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
        message_spans = message_line_spans(trajectory_path)
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
            row,
            messages,
            list(range(len(messages))),
            trajectory_path,
            raw_root,
            archive_sha,
            corpus=corpus,
            corpus_revision=corpus_revision,
            split=split,
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
                response_message=message,
                response_index=response_index,
                corpus=corpus,
                corpus_revision=corpus_revision,
                split=split,
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
                "incorrect_stages": evaluation_stage_summary(
                    row.get("incorrect_stages", []),
                    trajectory_id=row["traj_id"],
                    source_file_sha256=summary["source_file_sha256"],
                    source_file_name=trajectory_path.name,
                    message_spans=message_spans,
                ),
                "stage_count": row.get("stage_count"),
                "step_count": row.get("step_count"),
                "label_source": "manifest; evaluation-only; not included in trace input",
            }
        )

    write_json(out_dir / "selection.json", selection)
    labelled_steps = [
        step
        for record in labels
        for stage in record.get("incorrect_stages", [])
        for step in stage.get("steps", [])
    ]
    exact_join_steps = [
        step for step in labelled_steps if step.get("source_event_join", {}).get("status") == "exact"
    ]
    explicit_locator_count = sum(
        len(step.get("source_locators", {})) for step in labelled_steps
    )
    write_json(
        out_dir / "evaluation" / "labels.json",
        {
            "schema_version": 2,
            "records": labels,
            "planner_input": False,
            "source_locator_join": {
                "labelled_step_count": len(labelled_steps),
                "steps_with_exact_source_event_join": len(exact_join_steps),
                "steps_without_exact_source_event_join": len(labelled_steps) - len(exact_join_steps),
                "explicit_source_locator_count": explicit_locator_count,
                "positional_fallback_used": False,
            },
        },
    )
    write_json(out_dir / "provenance" / "trajectory-summaries.json", {"records": summaries})
    with (out_dir / "provenance" / "source-events.jsonl").open("w", encoding="utf-8", newline="\n") as handle:
        for event in all_events:
            handle.write(json.dumps(event, ensure_ascii=False, sort_keys=True) + "\n")
    write_json(
        out_dir / "import-report.json",
        {
            "schema_version": 2,
            "corpus": corpus,
            "corpus_revision": corpus_revision,
            "split": split,
            "trajectory_count": len(summaries),
            "turn_count": sum(summary["turn_count"] for summary in summaries),
            "source_event_count": len(all_events),
            "content_retained": False,
            "evaluation_labels_in_decision_inputs": False,
            "observer_input_glob": "traces/**/*.json",
            "token_counts_are_surrogates": True,
            "evidence_adapter": {
                "schema_version": EVIDENCE_SCHEMA_VERSION,
                "source_explicit": [
                    "messages[].timestamp",
                    "messages[].role",
                    "messages[].extra.response.id",
                    "messages[].extra.response.model",
                    "messages[].extra.response.created",
                    "messages[].extra.response.object",
                    "messages[].extra.response.choices[].finish_reason",
                    "messages[].extra.response.usage",
                ],
                "derived_structural": [
                    "messages[n] source event index/path",
                    "generated source event ID",
                    "role-only source and semantic-zone projection",
                    "explicit evaluation locator to unique message span",
                ],
                "unknown_or_absent": [
                    "tool-call/result IDs and links",
                    "dependency edges",
                    "required",
                    "optional",
                    "stale",
                    "invalidation",
                    "supersession",
                    "removability",
                ],
            },
            "evaluation_source_locator_join": {
                "labelled_step_count": len(labelled_steps),
                "exact_step_join_count": len(exact_join_steps),
                "unresolved_step_count": len(labelled_steps) - len(exact_join_steps),
                "explicit_locator_count": explicit_locator_count,
                "positional_fallback_used": False,
            },
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
    select.add_argument("--corpus", default=CORPUS)
    select.add_argument("--corpus-revision", default=CORPUS_REVISION)
    select.add_argument("--split", default=SPLIT)
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
    importer.add_argument("--corpus")
    importer.add_argument("--corpus-revision")
    importer.add_argument("--replace", action="store_true")

    args = parser.parse_args()
    if args.command == "select":
        selected, output = select_rows(
            read_jsonl(args.manifest),
            args.count_per_cell,
            set(args.exclude_traj_id),
            corpus=args.corpus,
            corpus_revision=args.corpus_revision,
            split=args.split,
        )
        if len(selected) < 20 or len(selected) > 50:
            raise SystemExit(f"selected slice has {len(selected)} rows; expected 20-50")
        write_json(args.out, output)
        print(json.dumps({"ok": True, "population_count": output["population_count"], "selected_count": output["selected_count"]}))
    else:
        import_rows(args)


if __name__ == "__main__":
    main()
