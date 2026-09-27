import unittest
from retry import RetryPolicy, dispatch_with_retry


class RetryPolicyTests(unittest.TestCase):
    def test_max_attempts_includes_initial_call(self) -> None:
        statuses = iter([503, 503, 503, 200])
        calls: list[int] = []
        sleeps: list[float] = []

        def send() -> int:
            calls.append(len(calls) + 1)
            return next(statuses)

        result = dispatch_with_retry(send, RetryPolicy(4, frozenset({429, 503}), 125), sleeps.append)
        self.assertEqual(len(calls), 4)
        self.assertEqual(result, 200)
        self.assertEqual(sleeps, [0.125, 0.125, 0.125])
