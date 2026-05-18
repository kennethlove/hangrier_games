mod affliction;
mod fixation;
mod kind;
mod mechanics;
mod phobia;
mod severity;
mod source;
mod trauma;

pub use affliction::Affliction;
pub use fixation::{FixationAction, FixationMetadata, FixationOrigin, ThwartReason};
pub use kind::{AfflictionKind, BodyPart, FixationTarget, PhobiaTrigger};
pub use mechanics::{
    DecayOutcome, ReinforcementOutcome, apply_traumatic_reinforcement, tick_decay,
};
pub use phobia::{PhobiaMetadata, PhobiaOrigin};
pub use severity::Severity;
pub use source::{
    AfflictionKey, AfflictionSource, BeastKind, CauseClass, DeathCause, HazardKind, TraumaSource,
};
pub use trauma::TraumaMetadata;
