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
//! 3. [`structure`] — structural/wire identity (zones, fingerprints).
//! 4. [`usage`] — offline provider usage normalizers (raw -> normalized).
//! 5. [`prefixity_score`] — the experimental, explainable "prefixity" score.
//! 6. [`analysis`] — single-trace accounting (candidate/heuristic only; a
//!    single trace cannot prove reuse).
//! 7. [`compare`] — observed prefix reuse between two recorded requests.
//! 8. [`cost`] — cost arithmetic over an externally supplied
//!    [`CostProfile`], consuming explicit normalized categories.
//! 9. [`policy`] — offline, zone-constrained policy simulation that never
//!    mutates its input.
//! 10. [`decision`] — conservative Phase 1B intervention-plan contract and
//!     fail-open offline baseline.
//!
//! Three concepts are kept strictly separate (Phase 0A.1):
//!
//! * **prefixity score** (experimental heuristic);
//! * **observed prefix reuse** (trace-to-trace comparison);
//! * **provider-reported cache reuse** (raw usage normalised per schema).
//!
//! See [`model::RequestTrace`] and `docs/phase-0/TRACE_FORMAT.md` for the
//! on-disk format, and `docs/phase-0/PHASE_0_PLAN.md` for the Phase 0 goals.

pub mod analysis;
pub mod compare;
pub mod cost;
pub mod decision;
pub mod error;
pub mod hash;
pub mod limits;
pub mod model;
pub mod observation;
pub mod policy;
pub mod prefixity_score;
pub mod structure;
pub mod terminal;
pub mod tokens;
pub mod usage;
pub mod validation;

pub use error::PrefixityError;
pub use model::{CostProfile, RawUsage, RequestTrace};
pub use observation::{CacheObservation, ContextArtifact, RuntimeCacheCapabilities};
