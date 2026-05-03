use gloo_storage::Storage;
use shared::messages::MessageKind;
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum FilterMode {
    #[default]
    All,
    Subset(HashSet<MessageKind>),
}

impl FilterMode {
    pub fn matches(&self, kind: MessageKind) -> bool {
        match self {
            FilterMode::All => true,
            FilterMode::Subset(set) => {
                // CombatSwing rides with Combat: the Combat chip toggles both
                // so the engagement card and the typed swing card stay paired
                // until consumers migrate off `CombatEngagement.detail_lines`.
                if kind == MessageKind::CombatSwing {
                    return set.contains(&MessageKind::Combat);
                }
                set.contains(&kind) || kind == MessageKind::State
            }
        }
    }
    pub fn is_all(&self) -> bool {
        matches!(self, FilterMode::All)
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct PeriodFilters {
    pub by_game: HashMap<String, FilterMode>,
    pub generations: HashMap<String, u32>,
    /// Per-game selected tribute filter (tribute identifier). `None` means
    /// "show events for all tributes".
    pub tribute_by_game: HashMap<String, Option<String>>,
}

impl PeriodFilters {
    pub fn filter_for(&self, game_id: &str) -> FilterMode {
        self.by_game.get(game_id).cloned().unwrap_or_default()
    }

    pub fn set_filter(&mut self, game_id: &str, mode: FilterMode) {
        self.by_game.insert(game_id.to_string(), mode.clone());
        let key = format!("period_filters:{game_id}");
        // best-effort persist; ignore failure
        let _ = gloo_storage::LocalStorage::set(&key, SerializableFilter::from(&mode));
    }

    pub fn tribute_filter(&self, game_id: &str) -> Option<String> {
        self.tribute_by_game.get(game_id).cloned().flatten()
    }

    pub fn set_tribute_filter(&mut self, game_id: &str, tribute_id: Option<String>) {
        self.tribute_by_game
            .insert(game_id.to_string(), tribute_id.clone());
        let key = format!("period_tribute_filter:{game_id}");
        match tribute_id {
            Some(id) => {
                let _ = gloo_storage::LocalStorage::set(&key, &id);
            }
            None => {
                gloo_storage::LocalStorage::delete(&key);
            }
        }
    }

    pub fn hydrate(&mut self, game_id: &str) {
        if !self.by_game.contains_key(game_id) {
            let key = format!("period_filters:{game_id}");
            if let Ok(saved) = gloo_storage::LocalStorage::get::<SerializableFilter>(&key) {
                self.by_game.insert(game_id.to_string(), saved.into());
            }
        }
        if !self.tribute_by_game.contains_key(game_id) {
            let key = format!("period_tribute_filter:{game_id}");
            let saved = gloo_storage::LocalStorage::get::<String>(&key).ok();
            self.tribute_by_game.insert(game_id.to_string(), saved);
        }
    }

    pub fn generation(&self, game_id: &str) -> u32 {
        self.generations.get(game_id).copied().unwrap_or(0)
    }

    pub fn bump(&mut self, game_id: &str) {
        let entry = self.generations.entry(game_id.to_string()).or_insert(0);
        *entry += 1;
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializableFilter {
    mode: String,
    kinds: Vec<MessageKind>,
}

impl From<&FilterMode> for SerializableFilter {
    fn from(m: &FilterMode) -> Self {
        match m {
            FilterMode::All => Self {
                mode: "all".into(),
                kinds: vec![],
            },
            FilterMode::Subset(s) => Self {
                mode: "subset".into(),
                kinds: s.iter().copied().collect(),
            },
        }
    }
}

impl From<SerializableFilter> for FilterMode {
    fn from(s: SerializableFilter) -> Self {
        if s.mode == "subset" {
            FilterMode::Subset(s.kinds.into_iter().collect())
        } else {
            FilterMode::All
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CombatSwing rides with the Combat chip so the engagement card and
    /// the typed swing card stay paired in the timeline.
    #[test]
    fn combat_chip_toggles_combat_swing_too() {
        let mut set = HashSet::new();
        set.insert(MessageKind::Combat);
        let mode = FilterMode::Subset(set);
        assert!(mode.matches(MessageKind::Combat));
        assert!(mode.matches(MessageKind::CombatSwing));
        assert!(!mode.matches(MessageKind::Death));
    }

    #[test]
    fn combat_swing_hidden_when_combat_unselected() {
        let mut set = HashSet::new();
        set.insert(MessageKind::Death);
        let mode = FilterMode::Subset(set);
        assert!(!mode.matches(MessageKind::Combat));
        assert!(!mode.matches(MessageKind::CombatSwing));
    }

    #[test]
    fn all_mode_matches_combat_swing() {
        assert!(FilterMode::All.matches(MessageKind::CombatSwing));
    }
}
