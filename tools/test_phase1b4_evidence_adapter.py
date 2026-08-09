import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("phase1a_tracebench.py")
SPEC = importlib.util.spec_from_file_location("phase1a_tracebench", MODULE_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class EvidenceAdapterTests(unittest.TestCase):
    def test_user_observation_text_does_not_become_tool_result(self):
        source, zone = MODULE.classify_message(
            {"role": "user", "content": "<returncode>0</returncode>"}, 3
        )
        self.assertEqual((source, zone), ("user_request", "messages"))

    def test_trace_preserves_timestamp_usage_and_provenance_without_safety_defaults(self):
        messages = [
            {"role": "system", "content": "system", "timestamp": 1.0},
            {"role": "user", "content": "request", "timestamp": 2.0},
            {
                "role": "assistant",
                "content": "response",
                "timestamp": 3.0,
                "extra": {
                    "response": {
                        "id": "response-1",
                        "model": "gpt-test",
                        "created": 3,
                        "object": "chat.completion",
                        "choices": [
                            {
                                "index": 0,
                                "finish_reason": "stop",
                                "message": {
                                    "role": "assistant",
                                    "content": "response",
                                    "tool_calls": None,
                                },
                            }
                        ],
                        "usage": {
                            "prompt_tokens": 10,
                            "completion_tokens": 2,
                            "total_tokens": 12,
                            "unsupported_provider_field": {"value": 7},
                        },
                    }
                },
            },
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            raw = root / "raw.traj.json"
            raw.write_text(json.dumps({"messages": messages}, indent=2), encoding="utf-8")
            trace, events, _ = MODULE.make_trace(
                {
                    "traj_id": "trajectory",
                    "model": "OpenAI/GPT-test",
                    "task_name": "task",
                    "task_slug": "task",
                    "agent": "mini-SWE-agent",
                },
                messages[:2],
                [0, 1],
                raw,
                root,
                None,
                turn_index=0,
                response_message=messages[2],
                response_index=2,
            )
        self.assertEqual(trace["blocks"][1]["timestamp"], 2.0)
        self.assertEqual(
            trace["blocks"][1]["provenance"]["timestamp"]["origin"],
            "source_explicit",
        )
        self.assertEqual(trace["provider_response"]["id"], "response-1")
        self.assertEqual(trace["provider_response"]["field_states"]["tool_calls"], "null")
        self.assertEqual(
            trace["usage"]["raw"]["unsupported_provider_field"], {"value": 7}
        )
        self.assertEqual(trace["provenance"]["usage"]["origin"], "source_explicit")
        self.assertFalse(any(key in trace["blocks"][0] for key in ("optional", "required", "stale")))
        self.assertFalse(any("dependencies" in block for block in trace["blocks"]))
        self.assertEqual(events[1]["timestamp"], 2.0)

    def test_evaluation_join_requires_explicit_locator_and_drops_content(self):
        stages = [
            {
                "stage_id": 1,
                "steps": [
                    {
                        "step_id": 10,
                        "labels": ["incorrect"],
                        "action_ref": {
                            "path": "traj.json",
                            "line_start": 2,
                            "line_end": 3,
                            "content": "must not be retained",
                        },
                    },
                    {"step_id": 11, "labels": ["incorrect"]},
                ],
            }
        ]
        summary = MODULE.evaluation_stage_summary(
            stages,
            trajectory_id="trajectory",
            source_file_sha256="a" * 64,
            source_file_name="traj.json",
            message_spans={0: (1, 4)},
        )
        first, second = summary[0]["steps"]
        self.assertEqual(first["source_event_join"]["status"], "exact")
        self.assertEqual(second["source_event_join"]["status"], "unresolved")
        self.assertNotIn("content", json.dumps(summary))
        self.assertNotIn("position", second["source_event_join"])


if __name__ == "__main__":
    unittest.main()
