from collections.abc import Mapping

from retry import RetryPolicy


def policy_from_settings(values: Mapping[str, object]) -> RetryPolicy:
    return RetryPolicy(
        max_attempts=int(values["max_attempts"]),
        retryable_statuses=frozenset(int(status) for status in values["retryable_statuses"]),
        backoff_ms=int(values["backoff_ms"]),
    )
