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
//!
//! See `docs/superpowers/specs/2026-05-03-phobias-design.md`.

pub mod scan;
pub mod spawn;
pub mod triggers;

pub use scan::{PhobiaScanResult, scan_phobias};
pub use spawn::{MAX_PHOBIAS, innate_phobia_metadata, roll_spawn_phobias};
pub use triggers::PhobiaContext;
