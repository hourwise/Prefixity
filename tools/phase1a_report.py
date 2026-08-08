#!/usr/bin/env python3
"""Summarise offline Phase 1A observer results without changing decisions."""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any


def classification(recommendation: str) -> str:
    if "no structural change recommended" in recommendation:
        return "DO_NOTHING"
    if "consider OFFLINE stable-prefix simulation" in recommendation:
        return "INTERVENTION_CANDIDATE"
    return "REVIEW"


def label_counts(record: dict[str, Any]) -> dict[str, int]:
    incorrect = 0
    unuseful = 0
    for stage in record.get("incorrect_stages", []):
        if not isinstance(stage, dict):
            continue
        incorrect += len(stage.get("incorrect_step_ids", []))
        unuseful += len(stage.get("unuseful_step_ids", []))
    return {"incorrect_step_count": incorrect, "unuseful_step_count": unuseful}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--analyses", type=Path, required=True)
    parser.add_argument("--labels", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--corpus", default="verified manifest")
    args = parser.parse_args()

    analyses = json.loads(args.analyses.read_text(encoding="utf-8"))
    labels_document = json.loads(args.labels.read_text(encoding="utf-8"))
    labels = {record["trajectory_id"]: record for record in labels_document["records"]}
    counts: Counter[str] = Counter()
    examples: dict[str, list[dict[str, Any]]] = {
        "positive_intervention_candidates": [],
        "negative_or_nonuseful_candidates": [],
        "do_nothing": [],
    }
    trajectory_counts: Counter[str] = Counter()
    total_tokens = 0
    volatile_tokens = 0
    stable_prefix_tokens = 0

    for item in analyses:
        analysis = item.get("result", {}).get("analysis")
        if not isinstance(analysis, dict):
            counts["ERROR"] += 1
            continue
        recommendation = str(analysis.get("recommendation", ""))
        decision_class = classification(recommendation)
        counts[decision_class] += 1
        trace = analysis.get("trace", {})
        trajectory_id = str(trace.get("session_id", "unknown"))
        trajectory_counts[trajectory_id] += 1
        total_tokens += int(analysis.get("total_estimated_tokens", 0))
        volatile_tokens += int(analysis.get("volatile_tokens", 0))
        stable_prefix_tokens += int(analysis.get("stable_prefix_candidate_tokens", 0))
        label = labels.get(trajectory_id, {})
        label_summary = label_counts(label)
        example = {
            "trace": item.get("trace"),
            "request_id": trace.get("request_id"),
            "trajectory_id": trajectory_id,
            "recommendation": recommendation,
            "total_estimated_tokens": analysis.get("total_estimated_tokens"),
            "volatile_tokens": analysis.get("volatile_tokens"),
            "stable_prefix_candidate_tokens": analysis.get("stable_prefix_candidate_tokens"),
            "evaluation_only_label_summary": label_summary,
        }
        if decision_class == "INTERVENTION_CANDIDATE":
            if len(examples["positive_intervention_candidates"]) < 3:
                examples["positive_intervention_candidates"].append(example)
            if (
                label_summary["incorrect_step_count"]
                or label_summary["unuseful_step_count"]
            ) and len(examples["negative_or_nonuseful_candidates"]) < 3:
                examples["negative_or_nonuseful_candidates"].append(example)
        elif decision_class == "DO_NOTHING" and len(examples["do_nothing"]) < 3:
            examples["do_nothing"].append(example)

    result = {
        "schema_version": 1,
        "observer_decision_classes": dict(sorted(counts.items())),
        "trajectory_count": len(trajectory_counts),
        "request_count": len(analyses),
        "request_counts_by_trajectory": dict(sorted(trajectory_counts.items())),
        "surrogate_token_totals": {
            "total_estimated_tokens": total_tokens,
            "volatile_tokens": volatile_tokens,
            "stable_prefix_candidate_tokens": stable_prefix_tokens,
        },
        "examples": examples,
        "evaluation_labels": {
            "used_in_observer_decisions": False,
            "source": f"{args.corpus}; step IDs and labels only",
            "interpretation": (
                "Negative/non-useful examples are post-hoc correlations only; their labels "
                "were not provided to the observer."
            ),
        },
        "interpretation_limits": {
            "intervention_candidate_meaning": (
                "Deterministic observer/adapter structural candidate only; not a validated "
                "safe intervention, provider cache saving, or quality-preserving reduction."
            )
        },
        "evidence_limits": {
            "provider_cache_evidence": False,
            "realized_provider_savings": False,
            "token_counts_are_provider_tokens": False,
            "live_provider_calls": False,
        },
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(
        json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps({"ok": True, "request_count": len(analyses), "classes": dict(counts)}))


if __name__ == "__main__":
    main()
