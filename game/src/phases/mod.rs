//! Per-phase pipeline scaffolding (see
//! `docs/superpowers/specs/2026-05-03-four-phase-day-design.md`).
//!
//! PR2b lands the environmental substrate: the per-area `(phase, biome,
//! weather)` roll plus its output container. PR2c will wire the brain and
//! tribute affliction application against the types defined here.

pub mod environment;
