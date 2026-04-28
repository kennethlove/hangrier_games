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
            FilterMode::Subset(set) => set.contains(&kind) || kind == MessageKind::State,
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

    pub fn hydrate(&mut self, game_id: &str) {
        if self.by_game.contains_key(game_id) {
            return;
        }
        let key = format!("period_filters:{game_id}");
        if let Ok(saved) = gloo_storage::LocalStorage::get::<SerializableFilter>(&key) {
            self.by_game.insert(game_id.to_string(), saved.into());
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
