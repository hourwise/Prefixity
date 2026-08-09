#!/usr/bin/env python3
"""Offline Phase 1B.1 characterization of the frozen Phase 1B.0 planner.

This tool deliberately shells out to the existing CLI. It contains no
planner rules and never passes evaluation labels to the planner.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, Iterable


REPORT_SCHEMA_NAME = "prefixity.phase1b1.characterization"
REPORT_SCHEMA_VERSION = 1
INTERVENTION_CONTRACT_VERSION = 1
FROZEN_PLANNER_CHECKPOINT = "3436e16afcdf359a33a691c15202900d796b25bc"
EXPECTED_CORPUS = "NJU-LINK/CodeTraceBench"
EXPECTED_CORPUS_REVISION = "aa213b84ffb6690fc37ca15766d6ca174ec36d4d"
EXPECTED_SPLIT = "verified"

CONTRACT_CLASSES = [
    "KEEP",
    "DEFER",
    "PRUNE",
    "RELOCATE_CANDIDATE",
    "COMPRESS_CANDIDATE",
    "DO_NOTHING",
]
INTERVENTION_CLASSES = {
    "DEFER",
    "PRUNE",
    "RELOCATE_CANDIDATE",
    "COMPRESS_CANDIDATE",
}
REASON_CODES = [
    "REQUIRED_BLOCK",
    "PROTOCOL_CRITICAL_BLOCK",
    "CURRENT_REQUEST",
    "OPTIONAL_STALE_TOOL_RESULT",
    "OPTIONAL_VOLATILE_TOOL_RESULT",
    "DEPENDENCY_CLOSURE_PROTECTED",
    "UNKNOWN_DEPENDENCY_EVIDENCE",
    "UNKNOWN_SAFETY",
    "CHRONOLOGY_PROTECTED",
    "CROSS_ZONE_RELOCATION_REJECTED",
    "WITHIN_ZONE_RELOCATION",
    "STRUCTURAL_HEURISTIC_NOT_SUFFICIENT",
    "NO_JUSTIFIED_INTERVENTION",
    "NO_PROVIDER_EVIDENCE",
    "PROVIDER_EVIDENCE_NOT_USED_AS_SAFETY_PROOF",
    "NO_ECONOMIC_EVIDENCE",
    "QUALITY_EVIDENCE_ABSENT",
    "COMPRESSION_NOT_ESTABLISHED",
]
EVIDENCE_STRENGTHS = ["UNKNOWN", "WEAK", "MODERATE", "STRONG"]
QUALITY_RISKS = ["NONE_FOR_RETENTION", "UNKNOWN"]
PROVIDER_DEPENDENCE = ["NONE_FOR_RETENTION", "POTENTIALLY_RELEVANT", "UNKNOWN"]
DEPENDENCY_STATES = [
    "UNCERTAIN",
    "RELEVANT_DEPENDENCY",
    "NO_RELEVANT_DEPENDENCY",
]
SAFETY_FAILURE_FIELDS = [
    "destructive_recommendations_targeting_required_blocks",
    "destructive_recommendations_targeting_protocol_critical_blocks",
    "destructive_recommendations_targeting_current_user_requests",
    "destructive_recommendations_violating_known_dependency_closure",
    "destructive_recommendations_with_missing_or_cyclic_dependency_evidence",
    "unsafe_cross_zone_or_chronology_relocation_recommendations",
    "compress_candidate_emissions",
    "do_nothing_coexisting_with_actual_intervention_recommendations",
    "contradictory_destructive_recommendations_for_same_target",
    "source_trace_byte_changes_before_after_planning",
    "source_trace_hash_changes_before_after_planning",
    "non_hypothetical_recommendations",
    "plans_with_hypothetical_only_false",
]


def canonical_json(value: Any) -> str:
    return json.dumps(
        value,
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    )


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_canonical(value: Any) -> str:
    return sha256_bytes(canonical_json(value).encode("utf-8"))


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def relative_path(path: Path, root: Path) -> str:
    return path.resolve().relative_to(root.resolve()).as_posix()


def class_counts() -> dict[str, int]:
    return {name: 0 for name in CONTRACT_CLASSES}


def enum_counts(values: Iterable[str]) -> dict[str, int]:
    return {value: 0 for value in values}


def contains_key(value: Any, key: str) -> bool:
    if isinstance(value, dict):
        return key in value or any(contains_key(item, key) for item in value.values())
    if isinstance(value, list):
        return any(contains_key(item, key) for item in value)
    return False


def fail(message: str) -> None:
    raise RuntimeError(message)


def verify_corpus(
    root: Path,
    provenance_path: Path,
    selection_path: Path,
    import_report_path: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    provenance = read_json(provenance_path)
    selection = read_json(selection_path)
    import_report = read_json(import_report_path)

    if provenance.get("corpus") != EXPECTED_CORPUS:
        fail("corpus provenance name does not match accepted CodeTraceBench corpus")
    if provenance.get("corpus_revision") != EXPECTED_CORPUS_REVISION:
        fail("corpus provenance revision does not match the accepted revision")
    if selection.get("corpus") != EXPECTED_CORPUS or selection.get("corpus_revision") != EXPECTED_CORPUS_REVISION:
        fail("selection identity does not match the accepted corpus revision")
    if selection.get("split") != EXPECTED_SPLIT or import_report.get("split") != EXPECTED_SPLIT:
        fail("accepted Phase 1A split is not verified")
    if import_report.get("evaluation_labels_in_decision_inputs") is not False:
        fail("Phase 1A import report does not prove label exclusion")
    if import_report.get("trajectory_count") != 24 or import_report.get("turn_count") != 719:
        fail("Phase 1A import report does not contain the accepted 24/719 slice")

    selected_ids = {
        trajectory_id
        for cell in selection.get("cells", [])
        for trajectory_id in cell.get("selected_traj_ids", [])
    }
    if len(selected_ids) != 24 or selection.get("selected_count") != 24:
        fail("selection does not contain the accepted 24 trajectories")

    trace_paths = sorted(root.rglob("*.json"), key=lambda item: relative_path(item, root))
    trajectory_dirs = sorted(
        (item.name for item in root.iterdir() if item.is_dir()),
    )
    if len(trace_paths) != 719 or len(trajectory_dirs) != 24:
        fail("local trace root is not the accepted 24-trajectory/719-trace set")
    if set(trajectory_dirs) != selected_ids:
        fail("local trajectory directories differ from pinned selection")

    traces: list[dict[str, Any]] = []
    for path in trace_paths:
        document = read_json(path)
        metadata = document.get("metadata", {})
        trajectory_id = metadata.get("trajectory_id")
        if trajectory_id not in selected_ids or path.parent.name != trajectory_id:
            fail(f"trace trajectory identity mismatch at {relative_path(path, root)}")
        if document.get("format_version") != 2:
            fail(f"trace format mismatch at {relative_path(path, root)}")
        if metadata.get("corpus") != EXPECTED_CORPUS or metadata.get("corpus_revision") != EXPECTED_CORPUS_REVISION:
            fail(f"trace corpus provenance mismatch at {relative_path(path, root)}")
        if metadata.get("split") != EXPECTED_SPLIT or metadata.get("evaluation_labels_excluded") is not True:
            fail(f"trace evaluation boundary mismatch at {relative_path(path, root)}")
        if contains_key(document, "labels"):
            fail(f"evaluation label key found in decision-input trace {relative_path(path, root)}")
        traces.append(
            {
                "path": path,
                "relative_path": relative_path(path, root),
                "document": document,
                "trajectory_id": trajectory_id,
                "request_id": document.get("request_id", ""),
            }
        )

    corpus = {
        "name": EXPECTED_CORPUS,
        "revision": EXPECTED_CORPUS_REVISION,
        "split": EXPECTED_SPLIT,
        "phase_1a_fixture": relative_path(root.parent, Path.cwd()),
        "trajectory_count": len(selected_ids),
        "request_trace_count": len(traces),
        "source_event_count": import_report.get("source_event_count"),
        "excluded_missing_cases": {
            "count": len(selection.get("selection_method", {}).get("excluded_traj_ids", [])),
            "established_by": "Phase 1A selection.json",
            "trajectory_ids": sorted(selection.get("selection_method", {}).get("excluded_traj_ids", [])),
        },
    }
    return traces, corpus


def source_hashes(traces: list[dict[str, Any]]) -> dict[str, str]:
    return {
        item["relative_path"]: sha256_bytes(item["path"].read_bytes())
        for item in traces
    }


def sanitize_failure(text: str) -> str:
    text = re.sub(r"[\r\n\t]+", " ", text).strip()
    return text[:240]


def planner_command(binary: Path, trace_path: Path) -> list[str]:
    # Deliberately no labels path, provider profile, prompt, or trace mutation.
    return [str(binary), "--json", "plan", str(trace_path)]


def dependency_graph(trace: dict[str, Any]) -> tuple[dict[str, int], bool]:
    blocks = trace.get("blocks", [])
    ids = {block.get("id"): index for index, block in enumerate(blocks)}
    missing = any(
        dependency not in ids
        for block in blocks
        for dependency in block.get("dependencies", [])
    )
    state = [0] * len(blocks)

    def visit(index: int) -> bool:
        if state[index] == 1:
            return True
        if state[index] == 2:
            return False
        state[index] = 1
        for dependency in blocks[index].get("dependencies", []):
            if dependency in ids and visit(ids[dependency]):
                return True
        state[index] = 2
        return False

    cyclic = any(visit(index) for index in range(len(blocks)))
    return ids, missing or cyclic


def depends_on(trace: dict[str, Any], ids: dict[str, int], start: int, target: int) -> bool:
    blocks = trace.get("blocks", [])
    stack = [ids[dependency] for dependency in blocks[start].get("dependencies", []) if dependency in ids]
    seen: set[int] = set()
    while stack:
        index = stack.pop()
        if index == target:
            return True
        if index in seen:
            continue
        seen.add(index)
        stack.extend(ids[dependency] for dependency in blocks[index].get("dependencies", []) if dependency in ids)
    return False


def is_protocol_critical(block: dict[str, Any]) -> bool:
    return (
        block.get("semantic_zone") in {"system", "tools"}
        or block.get("role") in {"system", "tool"}
        or block.get("source") in {
            "system",
            "system_policy",
            "system-policy",
            "system_instruction",
            "system-instruction",
            "tool_definition",
            "tool_definitions",
            "tool-definition",
            "tools",
        }
    )


def is_current_request(block: dict[str, Any]) -> bool:
    return block.get("source") in {
        "user_request",
        "user-request",
        "current_request",
        "current-user-request",
    } or block.get("role") == "user"


def parse_relocation_positions(recommendation: dict[str, Any]) -> tuple[int, int] | None:
    match = re.search(
        r"from position (\d+) to (\d+)",
        recommendation.get("expected_structural_effect", ""),
    )
    if not match:
        return None
    return int(match.group(1)), int(match.group(2))


def compact_recommendation(
    trajectory_id: str,
    request_id: str,
    recommendation: dict[str, Any],
    trace: dict[str, Any],
) -> dict[str, Any]:
    reasons = list(recommendation.get("reason_codes", []))
    dependency_state = (
        "UNCERTAIN"
        if "UNKNOWN_DEPENDENCY_EVIDENCE" in reasons
        else "RELEVANT_DEPENDENCY"
        if recommendation.get("relevant_dependencies") or "DEPENDENCY_CLOSURE_PROTECTED" in reasons
        else "NO_RELEVANT_DEPENDENCY"
    )
    quality_evidence_present = any(
        item and "No replay, task-check or semantic-quality evidence" not in item
        for item in recommendation.get("source_evidence", {}).get("quality", [])
    )
    return {
        "trajectory_id": trajectory_id,
        "request_id": request_id,
        "class": recommendation.get("class"),
        "target_block_ids": list(recommendation.get("target_block_ids", [])),
        "reason_codes": reasons,
        "evidence_strength": recommendation.get("evidence_strength"),
        "expected_quality_risk": recommendation.get("expected_quality_risk"),
        "provider_state_dependence": recommendation.get("provider_state_dependence"),
        "provider_evidence_present": bool(recommendation.get("provider_evidence_present")),
        "economic_evidence_present": bool(recommendation.get("economic_evidence_present")),
        "quality_evidence_present": quality_evidence_present,
        "dependency_state": dependency_state,
        "hypothetical_only": recommendation.get("hypothetical_only") is True,
        "block_count": len(trace.get("blocks", [])),
    }


def compact_record(
    trace_info: dict[str, Any],
    plan: dict[str, Any],
    plan_digest: str,
) -> dict[str, Any]:
    counts = class_counts()
    targets = class_counts()
    candidates = 0
    for recommendation in plan.get("recommendations", []):
        decision_class = recommendation.get("class")
        if decision_class not in counts:
            fail(f"planner emitted unknown intervention class {decision_class!r}")
        counts[decision_class] += 1
        targets[decision_class] += len(recommendation.get("target_block_ids", []))
        if decision_class in INTERVENTION_CLASSES:
            candidates += 1
    return {
        "relative_trace": trace_info["relative_path"],
        "trajectory_id": trace_info["trajectory_id"],
        "request_id": trace_info["request_id"],
        "contract_version": plan.get("contract_version"),
        "plan_digest": plan_digest,
        "recommendation_count": len(plan.get("recommendations", [])),
        "class_counts": counts,
        "target_counts": targets,
        "actual_intervention_candidate_count": candidates,
        "plan": plan,
        "trace": trace_info["document"],
    }


def run_pass(
    binary: Path,
    traces: list[dict[str, Any]],
    root: Path,
    local_plan_root: Path | None,
    pass_name: str,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], str]:
    records: list[dict[str, Any]] = []
    failures: list[dict[str, Any]] = []
    if local_plan_root is not None:
        (local_plan_root / pass_name).mkdir(parents=True, exist_ok=True)

    for trace_index, trace_info in enumerate(traces):
        command = planner_command(binary, trace_info["path"])
        completed = subprocess.run(
            command,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
        if completed.returncode != 0:
            failures.append(
                {
                    "relative_trace": trace_info["relative_path"],
                    "request_id": trace_info["request_id"],
                    "failure_kind": "planner_process",
                    "message": sanitize_failure(completed.stderr or completed.stdout),
                }
            )
            continue
        try:
            payload = json.loads(completed.stdout)
            if payload.get("ok") is not True or not isinstance(payload.get("plan"), dict):
                raise ValueError("CLI did not return a successful plan document")
            plan = payload["plan"]
            if plan.get("contract_version") != INTERVENTION_CONTRACT_VERSION:
                raise ValueError("planner contract version mismatch")
            plan_digest = sha256_canonical(plan)
            record = compact_record(trace_info, plan, plan_digest)
            records.append(record)
            if local_plan_root is not None:
                local_name = (
                    f"{trace_index:04d}-"
                    f"{sha256_bytes(trace_info['relative_path'].encode('utf-8'))[:16]}.plan.json"
                )
                local_path = local_plan_root / pass_name / local_name
                local_path.write_text(
                    json.dumps(payload, indent=2, sort_keys=True, ensure_ascii=True) + "\n",
                    encoding="utf-8",
                )
        except (ValueError, TypeError, json.JSONDecodeError) as error:
            failures.append(
                {
                    "relative_trace": trace_info["relative_path"],
                    "request_id": trace_info["request_id"],
                    "failure_kind": "planner_output",
                    "message": sanitize_failure(str(error)),
                }
            )

    entries: list[dict[str, Any]] = [
        {
            "relative_trace": record["relative_trace"],
            "request_id": record["request_id"],
            "plan_digest": record["plan_digest"],
        }
        for record in records
    ]
    entries.extend(
        {
            "relative_trace": failure["relative_trace"],
            "request_id": failure["request_id"],
            "failure_kind": failure["failure_kind"],
            "message": failure["message"],
        }
        for failure in failures
    )
    entries.sort(key=lambda item: (item["relative_trace"], item.get("request_id", "")))
    return records, failures, sha256_canonical(entries)


def decision_distribution(records: list[dict[str, Any]]) -> dict[str, Any]:
    recommendation_counts = class_counts()
    traces_containing = class_counts()
    target_counts = class_counts()
    traces_with_intervention = 0
    traces_with_do_nothing = 0
    traces_with_multiple_candidates = 0
    for record in records:
        present: set[str] = set()
        for decision_class, count in record["class_counts"].items():
            recommendation_counts[decision_class] += count
            target_counts[decision_class] += record["target_counts"][decision_class]
            if count:
                present.add(decision_class)
                traces_containing[decision_class] += 1
        if record["actual_intervention_candidate_count"]:
            traces_with_intervention += 1
        if present == {"DO_NOTHING"}:
            traces_with_do_nothing += 1
        if record["actual_intervention_candidate_count"] >= 2:
            traces_with_multiple_candidates += 1
    recommendation_record_total = sum(recommendation_counts.values())
    expected_record_total = sum(record["recommendation_count"] for record in records)
    if recommendation_record_total != expected_record_total:
        fail("decision distribution recommendation counts do not reconcile")
    return {
        "recommendation_record_total": recommendation_record_total,
        "count_totals_reconcile": recommendation_record_total == expected_record_total,
        "recommendation_counts": recommendation_counts,
        "traces_containing_class": traces_containing,
        "target_block_counts_by_class": target_counts,
        "traces_with_at_least_one_non_noop_intervention": traces_with_intervention,
        "traces_whose_result_is_do_nothing": traces_with_do_nothing,
        "traces_with_multiple_intervention_candidates": traces_with_multiple_candidates,
        "counting_basis": "recommendation records; not savings claims",
    }


def evidence_distribution(records: list[dict[str, Any]]) -> dict[str, Any]:
    reasons = enum_counts(REASON_CODES)
    strengths = enum_counts(EVIDENCE_STRENGTHS)
    quality_risks = enum_counts(QUALITY_RISKS)
    dependence = enum_counts(PROVIDER_DEPENDENCE)
    provider = {"PRESENT": 0, "ABSENT": 0}
    economic = {"PRESENT": 0, "ABSENT": 0}
    quality = {"PRESENT": 0, "ABSENT": 0}
    dependency = enum_counts(DEPENDENCY_STATES)
    recommendation_total = 0
    for record in records:
        for recommendation in record["plan"].get("recommendations", []):
            recommendation_total += 1
            for reason in recommendation.get("reason_codes", []):
                if reason not in reasons:
                    fail(f"planner emitted unknown reason code {reason!r}")
                reasons[reason] += 1
            strengths[recommendation.get("evidence_strength")] += 1
            quality_risks[recommendation.get("expected_quality_risk")] += 1
            dependence[recommendation.get("provider_state_dependence")] += 1
            provider["PRESENT" if recommendation.get("provider_evidence_present") else "ABSENT"] += 1
            economic["PRESENT" if recommendation.get("economic_evidence_present") else "ABSENT"] += 1
            compact = compact_recommendation(
                record["trajectory_id"],
                record["request_id"],
                recommendation,
                record["trace"],
            )
            quality["PRESENT" if compact["quality_evidence_present"] else "ABSENT"] += 1
            dependency[compact["dependency_state"]] += 1
    return {
        "counting_basis": "recommendation records; evidence dimensions remain separate",
        "recommendation_total": recommendation_total,
        "reason_code_counts": reasons,
        "evidence_strength_counts": strengths,
        "expected_quality_risk_counts": quality_risks,
        "provider_state_dependence_counts": dependence,
        "provider_evidence": provider,
        "economic_evidence": economic,
        "quality_evidence": quality,
        "dependency_evidence_states": dependency,
    }


def safety_audit(
    records: list[dict[str, Any]],
    before_hashes: dict[str, str],
    after_hashes: dict[str, str],
) -> dict[str, Any]:
    counts = {field: 0 for field in SAFETY_FAILURE_FIELDS}
    target_classes: dict[str, set[str]] = defaultdict(set)
    for record in records:
        trace = record["trace"]
        blocks = trace.get("blocks", [])
        ids, dependency_uncertain = dependency_graph(trace)
        recommendations = record["plan"].get("recommendations", [])
        if record["plan"].get("hypothetical_only") is not True:
            counts["plans_with_hypothetical_only_false"] += 1
        for recommendation in recommendations:
            decision_class = recommendation.get("class")
            if recommendation.get("hypothetical_only") is not True:
                counts["non_hypothetical_recommendations"] += 1
            if decision_class in {"PRUNE", "DEFER"}:
                for target_id in recommendation.get("target_block_ids", []):
                    target_index = ids.get(target_id)
                    if target_index is None:
                        continue
                    block = blocks[target_index]
                    if block.get("required"):
                        counts["destructive_recommendations_targeting_required_blocks"] += 1
                    if is_protocol_critical(block):
                        counts["destructive_recommendations_targeting_protocol_critical_blocks"] += 1
                    if is_current_request(block):
                        counts["destructive_recommendations_targeting_current_user_requests"] += 1
                    if any(
                        depends_on(trace, ids, index, target_index)
                        for index in range(len(blocks))
                        if index != target_index
                    ):
                        counts["destructive_recommendations_violating_known_dependency_closure"] += 1
                    target_classes[target_id].add(decision_class)
                if dependency_uncertain:
                    counts["destructive_recommendations_with_missing_or_cyclic_dependency_evidence"] += 1
            elif decision_class == "RELOCATE_CANDIDATE":
                positions = parse_relocation_positions(recommendation)
                unsafe = positions is None
                if positions is not None:
                    source_index, destination_index = positions
                    unsafe = (
                        source_index >= len(blocks)
                        or destination_index >= len(blocks)
                        or blocks[source_index].get("semantic_zone")
                        != blocks[destination_index].get("semantic_zone")
                        or blocks[source_index].get("semantic_zone") == "messages"
                    )
                if unsafe:
                    counts["unsafe_cross_zone_or_chronology_relocation_recommendations"] += 1
            elif decision_class == "COMPRESS_CANDIDATE":
                counts["compress_candidate_emissions"] += 1
        if "DO_NOTHING" in {
            recommendation.get("class") for recommendation in recommendations
        } and any(
            recommendation.get("class") in INTERVENTION_CLASSES
            for recommendation in recommendations
        ):
            counts["do_nothing_coexisting_with_actual_intervention_recommendations"] += 1

    for classes in target_classes.values():
        if len(classes) > 1:
            counts["contradictory_destructive_recommendations_for_same_target"] += 1

    changed = sorted(
        path for path, before in before_hashes.items() if after_hashes.get(path) != before
    )
    counts["source_trace_hash_changes_before_after_planning"] = len(changed)
    counts["source_trace_byte_changes_before_after_planning"] = len(changed)
    return {
        **counts,
        "all_failure_counts_zero": all(value == 0 for value in counts.values()),
        "source_trace_integrity": {
            "traces_hashed_before": len(before_hashes),
            "traces_hashed_after": len(after_hashes),
            "changed_relative_traces": changed,
            "unchanged": not changed and before_hashes == after_hashes,
        },
        "counting_basis": "recommendation/target violations, except source-integrity counts",
    }


def deterministic_examples(records: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    candidates: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for record in records:
        for recommendation in record["plan"].get("recommendations", []):
            compact = compact_recommendation(
                record["trajectory_id"],
                record["request_id"],
                recommendation,
                record["trace"],
            )
            candidates[compact["class"]].append(compact)
    examples: dict[str, dict[str, Any]] = {}
    for decision_class in CONTRACT_CLASSES:
        if decision_class not in candidates:
            continue
        examples[decision_class] = min(
            candidates[decision_class],
            key=lambda item: (
                item["trajectory_id"],
                item["request_id"],
                item["target_block_ids"],
            ),
        )
    return examples


def load_posthoc_labels(labels_path: Path, records: list[dict[str, Any]]) -> dict[str, Any]:
    labels_document = read_json(labels_path)
    label_records = labels_document.get("records", [])
    labels = {record.get("trajectory_id"): record for record in label_records}
    trajectory_ids = sorted({record["trajectory_id"] for record in records})
    missing = sorted(set(trajectory_ids) - set(labels))
    if missing:
        fail("post-hoc label join is missing accepted trajectory IDs")

    grouped: dict[str, dict[str, Any]] = {}
    for solved in (False, True):
        group_records = [record for record in records if bool(labels[record["trajectory_id"]].get("solved")) == solved]
        recommendation_counts = class_counts()
        for record in group_records:
            for decision_class, count in record["class_counts"].items():
                recommendation_counts[decision_class] += count
        grouped["solved" if solved else "unsolved"] = {
            "trajectory_count": len({record["trajectory_id"] for record in group_records}),
            "trace_count": len(group_records),
            "recommendation_counts": recommendation_counts,
            "traces_with_at_least_one_non_noop_intervention": sum(
                1 for record in group_records if record["actual_intervention_candidate_count"]
            ),
        }

    incorrect_steps = sum(
        len(stage.get("incorrect_step_ids", []))
        for record in label_records
        for stage in record.get("incorrect_stages", [])
    )
    unuseful_steps = sum(
        len(stage.get("unuseful_step_ids", []))
        for record in label_records
        for stage in record.get("incorrect_stages", [])
    )
    # The trace importer preserves message IDs, while labels preserve source
    # step IDs. The accepted local evidence has no exact message-to-step map.
    overlay = {
        "performed": True,
        "labels_schema_version": labels_document.get("schema_version"),
        "label_record_count": len(label_records),
        "trajectory_join_count": len(trajectory_ids),
        "trajectory_join_missing": missing,
        "solved_trajectory_count": sum(bool(item.get("solved")) for item in label_records),
        "unsolved_trajectory_count": sum(not bool(item.get("solved")) for item in label_records),
        "labelled_incorrect_step_count": incorrect_steps,
        "labelled_unuseful_step_count": unuseful_steps,
        "decision_distribution_by_trajectory_outcome": grouped,
        "recommendation_overlap": {
            "status": "unavailable",
            "reason": "accepted traces expose message IDs and labels expose step IDs; no exact existing join is available",
        },
    }
    return overlay


def git_revision(root: Path, args: list[str]) -> str:
    completed = subprocess.run(
        ["git", "-c", f"safe.directory={root.as_posix()}", *args],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if completed.returncode != 0:
        return "unavailable"
    return completed.stdout.strip()


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    root = Path(args.repo_root).resolve()
    trace_root = (root / args.trace_root).resolve()
    fixture_root = trace_root.parent
    provenance_path = (root / args.provenance).resolve()
    selection_path = (root / args.selection).resolve()
    import_report_path = (root / args.import_report).resolve()
    labels_path = (root / args.labels).resolve()
    binary = (root / args.binary).resolve()
    output = (root / args.out).resolve()
    local_plan_root = (root / args.local_plan_dir).resolve() if args.local_plan_dir else None

    if not binary.exists():
        fail(f"planner binary does not exist: {relative_path(binary, root)}")
    traces, corpus = verify_corpus(
        trace_root,
        provenance_path,
        selection_path,
        import_report_path,
    )
    before_hashes = source_hashes(traces)
    first_records, first_failures, first_hash = run_pass(
        binary, traces, root, local_plan_root, "pass-1"
    )
    second_records, second_failures, second_hash = run_pass(
        binary, traces, root, local_plan_root, "pass-2"
    )
    after_hashes = source_hashes(traces)

    first_summary = [
        {key: value for key, value in record.items() if key not in {"plan", "trace"}}
        for record in first_records
    ]
    second_summary = [
        {key: value for key, value in record.items() if key not in {"plan", "trace"}}
        for record in second_records
    ]
    deterministic_match = (
        first_hash == second_hash
        and canonical_json(first_summary) == canonical_json(second_summary)
        and first_failures == second_failures
    )
    if not deterministic_match:
        fail("second planner pass did not reproduce the first pass")

    labels_overlay: dict[str, Any]
    labels_loaded_after_planning = False
    if labels_path.exists():
        # This is intentionally the first read of labels, after both passes.
        labels_overlay = load_posthoc_labels(labels_path, first_records)
        labels_loaded_after_planning = True
    else:
        labels_overlay = {
            "performed": False,
            "reason": "evaluation labels file was not available after planner execution",
        }
    labels_overlay.update(
        {
            "planner_input_boundary": {
                "labels_loaded_after_both_deterministic_planner_passes": labels_loaded_after_planning,
                "raw_labels_passed_to_planner": False,
                "planner_aggregate_hash_before_overlay": first_hash,
                "planner_output_hash_unchanged_by_overlay": True,
            }
        }
    )

    planner = {
        "intervention_plan_contract_version": INTERVENTION_CONTRACT_VERSION,
        "frozen_phase_1b0_planner_checkpoint": FROZEN_PLANNER_CHECKPOINT,
        "git_base_checkpoint": git_revision(root, ["rev-parse", "HEAD"]),
        "planner_mode": "offline CLI plan --json; no provider profile",
        "planner_binary": relative_path(binary, root),
        "planner_binary_sha256": sha256_bytes(binary.read_bytes()),
        "provider_input_available": False,
        "economic_input_available": False,
        "quality_input_available": False,
        "evaluation_labels_input_available": False,
    }
    execution = {
        "traces_attempted": len(traces),
        "plans_produced_successfully": len(first_records),
        "planning_validation_failures": len(first_failures),
        "second_pass_planning_validation_failures": len(second_failures),
        "failures": first_failures,
        "first_pass_aggregate_hash": first_hash,
        "second_pass_aggregate_hash": second_hash,
        "deterministic_match": deterministic_match,
        "second_pass_failures_match": first_failures == second_failures,
    }
    report = {
        "schema": {
            "name": REPORT_SCHEMA_NAME,
            "version": REPORT_SCHEMA_VERSION,
            "canonicalization": "sorted-key compact UTF-8 JSON; aggregate hash covers ordered trace/request/plan-digest entries",
            "contract_classes": CONTRACT_CLASSES,
            "reason_codes": REASON_CODES,
            "evidence_strengths": EVIDENCE_STRENGTHS,
            "quality_risks": QUALITY_RISKS,
            "provider_state_dependence": PROVIDER_DEPENDENCE,
            "dependency_evidence_states": DEPENDENCY_STATES,
            "safety_audit_fields": SAFETY_FAILURE_FIELDS,
        },
        "corpus": corpus,
        "planner": planner,
        "execution": execution,
        "decision_distribution": decision_distribution(first_records),
        "evidence_distribution": evidence_distribution(first_records),
        "safety_audit": safety_audit(first_records, before_hashes, after_hashes),
        "deterministic_examples": deterministic_examples(first_records),
        "post_hoc_label_audit": labels_overlay,
        "interpretation_limits": {
            "intervention_counts_are_not_savings_claims": True,
            "quality_preservation_not_established": True,
            "realized_provider_savings_not_measured": True,
            "live_provider_calls": False,
            "replay_performed": False,
            "compression_implemented": False,
            "raw_trace_text_or_full_plan_set_in_report": False,
        },
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(report, indent=2, sort_keys=True, ensure_ascii=True) + "\n",
        encoding="utf-8",
    )
    return report


def parser() -> argparse.ArgumentParser:
    current_fixture = "fixtures/phase-1a/codetracebench-mini-swe-v1"
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--repo-root", default=".")
    result.add_argument("--trace-root", default=f"{current_fixture}/traces")
    result.add_argument("--provenance", default=f"{current_fixture}/corpus-provenance.json")
    result.add_argument("--selection", default=f"{current_fixture}/selection.json")
    result.add_argument("--import-report", default=f"{current_fixture}/import-report.json")
    result.add_argument("--labels", default=f"{current_fixture}/evaluation/labels.json")
    result.add_argument("--binary", default="target/debug/prefixity.exe")
    result.add_argument("--out", default=f"{current_fixture}/results/phase1b1-characterization.json")
    result.add_argument(
        "--local-plan-dir",
        default=None,
        help="optional ignored directory for bulky per-trace planner outputs",
    )
    return result


def main(argv: list[str] | None = None) -> int:
    try:
        report = build_report(parser().parse_args(argv))
    except (OSError, RuntimeError, ValueError, json.JSONDecodeError) as error:
        print(f"phase1b1 characterization error: {error}", file=sys.stderr)
        return 1
    print(
        json.dumps(
            {
                "ok": True,
                "report_schema": report["schema"]["name"],
                "report_version": report["schema"]["version"],
                "traces_attempted": report["execution"]["traces_attempted"],
                "plans_produced": report["execution"]["plans_produced_successfully"],
                "deterministic_match": report["execution"]["deterministic_match"],
                "report": parser().parse_args(argv).out,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
