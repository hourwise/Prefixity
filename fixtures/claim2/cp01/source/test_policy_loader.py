import unittest
from policy_loader import policy_from_settings


class PolicyLoaderTests(unittest.TestCase):
    def test_settings_keep_attempt_count_and_millisecond_unit(self) -> None:
        policy = policy_from_settings({"max_attempts": 4, "retryable_statuses": [429, 503], "backoff_ms": 125})
        self.assertEqual(policy.max_attempts, 4)
        self.assertEqual(policy.retryable_statuses, frozenset({429, 503}))
        self.assertEqual(policy.backoff_ms, 125)
