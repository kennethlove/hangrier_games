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

    /// Serialize to a stable, URL-safe slug list (e.g. `"combat,death"`) for
    /// the `?filter=` query param. `FilterMode::All` (or any empty subset)
    /// returns the empty string so the param can be elided in URLs.
    pub fn to_query_value(&self) -> String {
        match self {
            FilterMode::All => String::new(),
            FilterMode::Subset(set) => {
                let mut slugs: Vec<&'static str> =
                    set.iter().filter_map(|k| message_kind_slug(*k)).collect();
                slugs.sort_unstable();
                slugs.join(",")
            }
        }
    }

    /// Parse a `?filter=` query value back into a `FilterMode`. Empty string,
    /// `"all"`, or only-unknown slugs all collapse to `FilterMode::All`.
    pub fn from_query_value(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
            return FilterMode::All;
        }
        let set: HashSet<MessageKind> = trimmed
            .split(',')
            .filter_map(|s| message_kind_from_slug(s.trim()))
            .collect();
        if set.is_empty() {
            FilterMode::All
        } else {
            FilterMode::Subset(set)
        }
    }
}

/// Stable URL slug for a `MessageKind` variant. `State` and `CombatSwing`
/// are intentionally excluded: `State` always passes the filter regardless
/// of selection, and `CombatSwing` rides with `Combat` (see `matches`).
fn message_kind_slug(kind: MessageKind) -> Option<&'static str> {
    Some(match kind {
        MessageKind::Death => "death",
        MessageKind::Combat => "combat",
        MessageKind::Alliance => "alliance",
        MessageKind::Movement => "movement",
        MessageKind::Item => "item",
        MessageKind::CombatSwing | MessageKind::State => return None,
    })
}

fn message_kind_from_slug(slug: &str) -> Option<MessageKind> {
    Some(match slug {
        "death" => MessageKind::Death,
        "combat" => MessageKind::Combat,
        "alliance" => MessageKind::Alliance,
        "movement" => MessageKind::Movement,
        "item" => MessageKind::Item,
        _ => return None,
    })
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

    #[test]
    fn query_value_roundtrips_subset() {
        let mut s = HashSet::new();
        s.insert(MessageKind::Combat);
        s.insert(MessageKind::Death);
        let mode = FilterMode::Subset(s);
        // Sorted for stability: combat,death.
        assert_eq!(mode.to_query_value(), "combat,death");
        let parsed = FilterMode::from_query_value(&mode.to_query_value());
        match parsed {
            FilterMode::Subset(s) => {
                assert!(s.contains(&MessageKind::Combat));
                assert!(s.contains(&MessageKind::Death));
                assert_eq!(s.len(), 2);
            }
            FilterMode::All => panic!("expected Subset"),
        }
    }

    #[test]
    fn query_value_all_is_empty_and_parses_back_to_all() {
        assert_eq!(FilterMode::All.to_query_value(), "");
        assert!(FilterMode::from_query_value("").is_all());
        assert!(FilterMode::from_query_value("all").is_all());
        // Unknown slugs collapse to All.
        assert!(FilterMode::from_query_value("nonsense,bogus").is_all());
    }

    #[test]
    fn query_value_skips_state_and_combat_swing() {
        let mut s = HashSet::new();
        s.insert(MessageKind::Combat);
        s.insert(MessageKind::CombatSwing);
        s.insert(MessageKind::State);
        let mode = FilterMode::Subset(s);
        assert_eq!(mode.to_query_value(), "combat");
    }
}
