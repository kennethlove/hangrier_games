mod addiction;
mod affliction;
mod fixation;
mod kind;
mod mechanics;
mod phobia;
mod severity;
mod source;
mod trapped;
mod trauma;

pub use addiction::{AddictionMetadata, AddictionResistReason};
pub use affliction::Affliction;
pub use fixation::{FixationAction, FixationMetadata, FixationOrigin, ThwartReason};
pub use kind::{AfflictionKind, BodyPart, FixationTarget, PhobiaTrigger, Substance};
pub use mechanics::{
    DecayOutcome, ReinforcementOutcome, apply_traumatic_reinforcement, tick_decay,
};
pub use phobia::{PhobiaMetadata, PhobiaOrigin};
pub use severity::Severity;
pub use source::{
    AfflictionKey, AfflictionSource, BeastKind, CauseClass, DeathCause, HazardKind, TraumaSource,
};
pub use trapped::{
    CYCLES_DECAY_PER_CYCLE, ESCAPE_ROLL_CAP, ESCAPE_STAT_BONUS_MAX, PARTIAL_RESCUE_THRESHOLD,
    RESCUE_BONUS_CAP, SEVERITY_BASE_MILD, SEVERITY_BASE_MODERATE, SEVERITY_BASE_SEVERE, TrapKind,
    TrappedMetadata, escape_threshold,
};
pub use trauma::TraumaMetadata;
