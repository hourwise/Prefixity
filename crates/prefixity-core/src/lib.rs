//! `prefixity-core`: deterministic, offline analysis logic for Prefixity.
//!
//! Prefixity is **experimental research software**. Phase 0 does not modify
//! live LLM requests. Every function in this crate operates on recorded or
//! synthetic trace files only, and requires no network access.
//!
//! The crate is organised as a small pipeline:
//!
//! 1. [`model`] — the versioned trace format and provider cost profiles.
//! 2. [`validation`] — structural validation of traces.
//! 3. [`prefixity_score`] — the experimental, explainable "prefixity" score.
//! 4. [`analysis`] — accounting, reconciliation and recommendations for a
//!    single trace.
//! 5. [`compare`] — divergence detection and reusable-prefix estimation
//!    between two consecutive requests.
//! 6. [`cost`] — cost arithmetic over an externally supplied [`CostProfile`].
//! 7. [`policy`] — offline policy simulation that never mutates its input.
//!
//! See [`model::RequestTrace`] and `docs/phase-0/TRACE_FORMAT.md` for the
//! on-disk format, and `docs/phase-0/PHASE_0_PLAN.md` for the Phase 0 goals.

pub mod analysis;
pub mod compare;
pub mod cost;
pub mod error;
pub mod hash;
pub mod limits;
pub mod model;
pub mod policy;
pub mod prefixity_score;
pub mod terminal;
pub mod tokens;
pub mod validation;

pub use error::PrefixityError;
pub use model::{CostProfile, ProviderUsage, RequestTrace};
