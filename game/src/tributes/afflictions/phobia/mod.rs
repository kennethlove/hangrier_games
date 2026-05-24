//! Phobia affliction system.
//!
//! Phobias are triggered, durable, observer-aware fears that sit dormant
//! inside a tribute's affliction set. When the stimulus is detected during
//! the per-cycle scan, the phobia fires, producing severity-tiered reactions.
//!
//! Modules:
//! - `triggers` — `PhobiaTrigger::is_present()` detection rules (spec §4)
//! - `spawn` — spawn-time phobia acquisition with weighted distribution (spec §6)
//! - `scan` — per-cycle scan skeleton, pure detection (spec §4, §7)
//! - `reaction` — severity tiers, trait modifiers, stat penalties, brain override (spec §5)
//!
//! See `docs/superpowers/specs/2026-05-03-phobias-design.md`.

pub mod reaction;
pub mod scan;
pub mod spawn;
pub mod triggers;

pub use reaction::{
    FiringPhobia, MAX_PHOBIA_PENALTY, PhobiaEffect, Reaction, collect_firing_phobias, effect_for,
    effective_severity, reaction_for, strongest_reaction, total_stat_penalty,
};
pub use scan::{PhobiaScanResult, scan_phobias, scan_tribute};
pub use spawn::{MAX_PHOBIAS, innate_phobia_metadata, roll_spawn_phobias};
pub use triggers::PhobiaContext;
