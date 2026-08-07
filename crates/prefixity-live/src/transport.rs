//! HTTP transport abstraction.
//!
//! The real implementation uses `reqwest` (blocking) with the `rustls` TLS
//! stack, certificate verification enabled, no redirects, and an explicit
//! per-request timeout. There is **no automatic retry** anywhere.
//!
//! A [`MockTransport`] is provided so the entire live pipeline can be tested
//! offline (no internet, no credentials) in CI.

use crate::error::LiveError;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::io::Read;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A sanitized HTTP response.
///
/// Only a fixed allowlist of safe header names is captured — never
/// authorization headers or full header dumps.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response body (as text).
    pub body: String,
    /// Allowlisted safe response headers.
    pub safe_headers: BTreeMap<String, String>,
    /// Time until the response headers were received.
    pub time_to_headers_ms: u64,
    /// Time until the first body byte arrived (approximate with the blocking
    /// client), if measurable.
    pub time_to_first_body_byte_ms: Option<u64>,
    /// Total time until the body was fully read.
    pub total_ms: u64,
}

/// Header names that are safe to capture from a provider response.
const SAFE_RESPONSE_HEADERS: &[&str] = &[
    "x-request-id",
    "request-id",
    "content-type",
    "date",
    "openai-organization",
];

/// A minimal POST-JSON transport. The mock implementation lets CI exercise
/// the full pipeline without network access.
pub trait LiveHttpTransport: Send + Sync {
    /// POST `body` (a JSON string) to `url` with `headers`, with an explicit
    /// timeout. Never retries. Returns an error on timeout, transport
    /// failure, or a non-success HTTP status.
    fn post_json(
        &self,
        url: &str,
        headers: &BTreeMap<String, String>,
        body: &str,
        timeout: Duration,
    ) -> Result<HttpResponse, LiveError>;
}

/// Real transport over `reqwest` (blocking) with `rustls`.
pub struct ReqwestTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestTransport {
    /// Build a client with TLS verification enabled and redirects disabled.
    pub fn new() -> Result<ReqwestTransport, LiveError> {
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| LiveError::Network {
                message: format!("failed to build HTTP client: {e}"),
            })?;
        Ok(ReqwestTransport { client })
    }
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        // `new()` only fails on TLS/backend init, which is effectively
        // unreachable on supported platforms; fall back to unwrap for
        // ergonomic construction where a client is required.
        ReqwestTransport::new().expect("reqwest client construction cannot fail")
    }
}

impl LiveHttpTransport for ReqwestTransport {
    fn post_json(
        &self,
        url: &str,
        headers: &BTreeMap<String, String>,
        body: &str,
        timeout: Duration,
    ) -> Result<HttpResponse, LiveError> {
        let start = Instant::now();
        let mut request = self.client.post(url);
        for (name, value) in headers {
            request = request.header(name.as_str(), value.as_str());
        }
        request = request.header(reqwest::header::CONTENT_TYPE, "application/json");
        request = request.timeout(timeout);

        let mut response = request.body(body.to_string()).send().map_err(|e| {
            if e.is_timeout() {
                LiveError::Timeout
            } else {
                LiveError::Network {
                    message: format!("request failed: {e}"),
                }
            }
        })?;
        let time_to_headers_ms = start.elapsed().as_millis() as u64;
        let status = response.status().as_u16();

        let mut safe_headers = BTreeMap::new();
        for name in SAFE_RESPONSE_HEADERS {
            if let Some(value) = response.headers().get(*name) {
                if let Ok(value) = value.to_str() {
                    safe_headers.insert((*name).to_string(), value.to_string());
                }
            }
        }

        // Read the body in two steps to approximate time-to-first-byte.
        let mut first = [0u8; 64];
        let n = response.read(&mut first).map_err(|e| LiveError::Network {
            message: format!("failed to read response body: {e}"),
        })?;
        let time_to_first_body_byte_ms = Some(start.elapsed().as_millis() as u64);
        let mut rest = Vec::new();
        response
            .read_to_end(&mut rest)
            .map_err(|e| LiveError::Network {
                message: format!("failed to read response body: {e}"),
            })?;
        let total_ms = start.elapsed().as_millis() as u64;

        let mut body_bytes = Vec::with_capacity(n + rest.len());
        body_bytes.extend_from_slice(&first[..n]);
        body_bytes.extend_from_slice(&rest);
        let body = String::from_utf8_lossy(&body_bytes).into_owned();

        if status >= 400 {
            // STOP: no automatic retry. Body is deliberately not included.
            return Err(LiveError::HttpStatus { status });
        }

        Ok(HttpResponse {
            status,
            body,
            safe_headers,
            time_to_headers_ms,
            time_to_first_body_byte_ms,
            total_ms,
        })
    }
}

/// A recorded request made through a [`MockTransport`].
///
/// Headers are intentionally **not** recorded, so tests can prove no
/// credential ever reaches a recorded call log.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedCall {
    /// The URL the request was sent to.
    pub url: String,
    /// The JSON body that was sent.
    pub body: String,
    /// The timeout used.
    pub timeout: Duration,
}

/// Offline mock transport for CI testing. Returns canned responses (or
/// errors) in order, and records every call (without headers).
pub struct MockTransport {
    responses: Mutex<VecDeque<Result<HttpResponse, LiveError>>>,
    calls: Mutex<Vec<RecordedCall>>,
}

impl MockTransport {
    /// Create a mock with the given canned responses (served in order).
    pub fn new(responses: Vec<Result<HttpResponse, LiveError>>) -> MockTransport {
        MockTransport {
            responses: Mutex::new(responses.into()),
            calls: Mutex::new(Vec::new()),
        }
    }

    /// Convenience: a mock that always returns one canned success response.
    pub fn single(status: u16, body: &str) -> MockTransport {
        MockTransport::new(vec![ok_response(status, body)])
    }

    /// The calls that were made so far (URL, body, timeout — never headers).
    pub fn calls(&self) -> Vec<RecordedCall> {
        self.calls.lock().unwrap().clone()
    }

    /// Number of calls made.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

impl LiveHttpTransport for MockTransport {
    fn post_json(
        &self,
        url: &str,
        headers: &BTreeMap<String, String>,
        body: &str,
        timeout: Duration,
    ) -> Result<HttpResponse, LiveError> {
        let _ = headers; // headers are intentionally not recorded
        self.calls.lock().unwrap().push(RecordedCall {
            url: url.to_string(),
            body: body.to_string(),
            timeout,
        });
        match self.responses.lock().unwrap().pop_front() {
            Some(result) => result,
            None => Err(LiveError::Network {
                message: "mock transport exhausted its canned responses".to_string(),
            }),
        }
    }
}

/// Build a successful canned response (zero timing).
pub fn ok_response(status: u16, body: &str) -> Result<HttpResponse, LiveError> {
    Ok(HttpResponse {
        status,
        body: body.to_string(),
        safe_headers: BTreeMap::new(),
        time_to_headers_ms: 1,
        time_to_first_body_byte_ms: Some(1),
        total_ms: 2,
    })
}

/// Build a canned error response.
pub fn err_response(error: LiveError) -> Result<HttpResponse, LiveError> {
    Err(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_serves_canned_responses_in_order() {
        let mock = MockTransport::new(vec![
            ok_response(200, r#"{"ok":true}"#),
            err_response(LiveError::Timeout),
        ]);
        let headers = BTreeMap::new();
        let first = mock
            .post_json(
                "https://example.test/a",
                &headers,
                "{}",
                Duration::from_secs(1),
            )
            .unwrap();
        assert_eq!(first.status, 200);
        assert_eq!(first.body, r#"{"ok":true}"#);
        let second = mock
            .post_json(
                "https://example.test/b",
                &headers,
                "{}",
                Duration::from_secs(1),
            )
            .unwrap_err();
        assert!(matches!(second, LiveError::Timeout));
    }

    #[test]
    fn mock_exhaustion_is_an_error() {
        let mock = MockTransport::new(Vec::new());
        let headers = BTreeMap::new();
        let err = mock
            .post_json("u", &headers, "{}", Duration::from_secs(1))
            .unwrap_err();
        assert!(matches!(err, LiveError::Network { .. }));
    }

    #[test]
    fn mock_records_calls_without_headers() {
        let mock = MockTransport::single(200, "ok");
        let mut headers = BTreeMap::new();
        headers.insert(
            "authorization".to_string(),
            "Bearer SUPER-SECRET".to_string(),
        );
        let _ = mock.post_json(
            "https://example.test/v1",
            &headers,
            "{\"a\":1}",
            Duration::from_secs(2),
        );
        let calls = mock.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].url, "https://example.test/v1");
        assert!(calls[0].body.contains("a"));
        // The recorded call must never contain the credential header value.
        assert!(!format!("{:?}", calls).contains("SUPER-SECRET"));
    }
}
