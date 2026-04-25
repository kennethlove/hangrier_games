//! Tribute trait system. Replaces `BrainPersonality`. See spec
//! `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md` §5.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trait {
    // Combat stance
    Aggressive,
    Defensive,
    Cautious,
    Reckless,
    // Social
    Friendly,
    Loyal,
    Paranoid,
    LoneWolf,
    Treacherous,
    // Mental
    Resilient,
    Fragile,
    Cunning,
    Dim,
    // Physical
    Asthmatic,
    Nearsighted,
    Tough,
}

impl Trait {
    pub fn label(&self) -> &'static str {
        match self {
            Trait::Aggressive => "aggressive",
            Trait::Defensive => "defensive",
            Trait::Cautious => "cautious",
            Trait::Reckless => "reckless",
            Trait::Friendly => "friendly",
            Trait::Loyal => "loyal",
            Trait::Paranoid => "paranoid",
            Trait::LoneWolf => "a lone wolf",
            Trait::Treacherous => "treacherous",
            Trait::Resilient => "resilient",
            Trait::Fragile => "fragile",
            Trait::Cunning => "cunning",
            Trait::Dim => "dim",
            Trait::Asthmatic => "asthmatic",
            Trait::Nearsighted => "nearsighted",
            Trait::Tough => "tough",
        }
    }
}
