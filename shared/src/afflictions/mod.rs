mod affliction;
mod kind;
mod mechanics;
mod phobia;
mod severity;
mod source;
mod trauma;

pub use affliction::Affliction;
pub use kind::{AfflictionKind, BodyPart, PhobiaTrigger};
pub use mechanics::{
    DecayOutcome, ReinforcementOutcome, apply_traumatic_reinforcement, tick_decay,
};
pub use phobia::{PhobiaMetadata, PhobiaOrigin};
pub use severity::Severity;
pub use source::{AfflictionKey, AfflictionSource, CauseClass, DeathCause, TraumaSource};
pub use trauma::TraumaMetadata;
