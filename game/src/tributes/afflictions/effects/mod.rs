//! Stat-effect and brain-bias computation for afflictions.
//!
//! §6 Mechanical Effects — each affliction kind maps to concrete stat
//! penalties and behavioral bias multipliers. Severity tiers scale
//! penalties linearly (Mild = 0.5x, Moderate = 1.0x, Severe = 1.5x).

mod brain_bias;
mod stat_modifiers;
mod trauma_effects;

pub use brain_bias::{BrainBias, compute_brain_bias};
pub use stat_modifiers::{StatModifiers, compute_stat_modifiers};
pub use trauma_effects::{avoidance_hard_veto, flashback_chance, sleep_recovery_multiplier};
