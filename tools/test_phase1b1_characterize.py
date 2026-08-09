import copy
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("phase1b1_characterize.py")
SPEC = importlib.util.spec_from_file_location("phase1b1_characterize", MODULE_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class CharacterizationSchemaTests(unittest.TestCase):
    def test_canonical_hash_is_stable(self):
        value = {"b": 2, "a": [True, "x"]}
        self.assertEqual(MODULE.sha256_canonical(value), MODULE.sha256_canonical(value))
        self.assertEqual(MODULE.canonical_json(value), '{"a":[true,"x"],"b":2}')

    def test_all_six_classes_are_zero_safe(self):
        self.assertEqual(
            list(MODULE.class_counts()),
            [
                "KEEP",
                "DEFER",
                "PRUNE",
                "RELOCATE_CANDIDATE",
                "COMPRESS_CANDIDATE",
                "DO_NOTHING",
            ],
        )
        self.assertEqual(set(MODULE.class_counts()), set(MODULE.CONTRACT_CLASSES))

    def test_planner_command_has_no_label_input(self):
        command = MODULE.planner_command(Path("prefixity"), Path("trace.json"))
        self.assertNotIn("--labels", command)
        self.assertNotIn("labels.json", " ".join(command))
        self.assertNotIn("--provider-profile", command)

    def test_decision_counts_reconcile(self):
        record = {
            "class_counts": MODULE.class_counts(),
            "target_counts": MODULE.class_counts(),
            "recommendation_count": 2,
            "actual_intervention_candidate_count": 0,
        }
        record["class_counts"]["DO_NOTHING"] = 1
        record["class_counts"]["KEEP"] = 1
        distribution = MODULE.decision_distribution([record])
        self.assertEqual(distribution["recommendation_record_total"], 2)
        self.assertTrue(distribution["count_totals_reconcile"])

    def test_label_overlay_does_not_mutate_planner_record(self):
        record = {
            "class_counts": MODULE.class_counts(),
            "target_counts": MODULE.class_counts(),
            "actual_intervention_candidate_count": 0,
            "trajectory_id": "trajectory",
            "request_id": "request",
        }
        before = copy.deepcopy(record)
        self.assertEqual(record, before)
        self.assertEqual(
            MODULE.sha256_canonical(record), MODULE.sha256_canonical(before)
        )

    def test_posthoc_label_join_is_separate_from_planner_record(self):
        record = {
            "class_counts": MODULE.class_counts(),
            "target_counts": MODULE.class_counts(),
            "recommendation_count": 1,
            "actual_intervention_candidate_count": 0,
            "trajectory_id": "trajectory",
            "request_id": "request",
            "plan": {"recommendations": []},
            "trace": {"blocks": []},
        }
        record["class_counts"]["DO_NOTHING"] = 1
        before = MODULE.canonical_json(record)
        with tempfile.TemporaryDirectory() as directory:
            labels = Path(directory) / "labels.json"
            labels.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "records": [
                            {
                                "trajectory_id": "trajectory",
                                "solved": True,
                                "incorrect_stages": [],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            overlay = MODULE.load_posthoc_labels(labels, [record])
        self.assertTrue(overlay["performed"])
        self.assertEqual(MODULE.canonical_json(record), before)

    def test_safety_field_vocab_is_explicit(self):
        self.assertIn(
            "source_trace_hash_changes_before_after_planning",
            MODULE.SAFETY_FAILURE_FIELDS,
        )
        self.assertIn(
            "do_nothing_coexisting_with_actual_intervention_recommendations",
            MODULE.SAFETY_FAILURE_FIELDS,
        )


if __name__ == "__main__":
    unittest.main()
