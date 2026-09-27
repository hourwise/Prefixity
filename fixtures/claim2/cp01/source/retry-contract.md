max_attempts is the maximum number of transport calls, including the first
call. A retry follows a 429 or 503 response only while an attempt remains.
backoff_ms, connect_timeout_ms, and request_timeout_ms are milliseconds.
The regression sends three retryable responses followed by success, so reaching
success requires all four configured calls.
