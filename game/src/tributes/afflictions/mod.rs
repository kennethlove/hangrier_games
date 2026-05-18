//! Game-layer affliction logic: anatomy resolution, acquisition API,
//! tuning. Storage and wire types live in `shared::afflictions`.
//!
//! PR1 ships only the foundation. Cure / cascade / brain-pipeline
//! integration arrive in PR2 and PR3.
//!
//! See `docs/superpowers/specs/2026-05-03-health-conditions-design.md`.

pub mod anatomy;
pub mod tuning;

pub use anatomy::{AcquireResolution, RejectReason, can_acquire};
pub use tuning::AfflictionTuning;

#[cfg(test)]
mod snapshot_tests;
