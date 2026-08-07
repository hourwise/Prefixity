//! `prefixity-live`: Phase 0B controlled live validation harness.
//!
//! This crate is **disposable experimental infrastructure**. Its only
//! responsibilities are:
//!
//! * controlled provider HTTP requests (OpenAI / Anthropic / DeepSeek);
//! * safe credential acquisition from environment variables;
//! * capture of request/response metadata;
//! * conversion into Prefixity trace format v2;
//! * writing sanitized experiment artifacts.
//!
//! It contains **no analysis logic**. `prefixity-core` remains authoritative
//! for normalization, structural comparison, analysis, cost modelling and
//! policy simulation. This crate performs experiment-level reconciliation
//! only (comparing observed structural reuse against provider-reported cache
//! reuse), and only using `prefixity-core` primitives.
//!
//! **Safety model (Phase 0B):**
//! * no command makes a paid/network call unless `--execute-live` is passed;
//! * request count is bounded (default 3, hard ceiling 10);
//! * a local input-token ceiling is enforced before any call;
//! * there is no automatic retry of paid requests;
//! * credentials come only from environment variables and are never
//!   persisted, serialized, logged, or printed;
//! * provider base URLs are hard-coded/allowlisted in the adapters;
//! * TLS verification remains enabled; redirects are not followed.

pub mod artifacts;
pub mod cli;
pub mod content;
pub mod credentials;
pub mod error;
pub mod experiment;
pub mod manifest;
pub mod providers;
pub mod result;
pub mod scenario;
pub mod trace;
pub mod transport;
