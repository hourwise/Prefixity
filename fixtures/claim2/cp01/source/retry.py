from dataclasses import dataclass
from typing import Callable


@dataclass(frozen=True)
class RetryPolicy:
    max_attempts: int
    retryable_statuses: frozenset[int]
    backoff_ms: int


def dispatch_with_retry(send: Callable[[], int], policy: RetryPolicy, sleep_seconds: Callable[[float], None]) -> int:
    last_status: int | None = None
    for attempt in range(1, policy.max_attempts):
        last_status = send()
        if last_status not in policy.retryable_statuses:
            return last_status
        if attempt < policy.max_attempts:
            sleep_seconds(policy.backoff_ms / 1000.0)
    if last_status is None:
        raise RuntimeError("retry loop produced no response")
    return last_status
