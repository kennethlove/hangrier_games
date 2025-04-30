use std::fmt::{Display, Formatter};
use std::str::FromStr;
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A persistent storage hook that can be used to store data across application reloads.
pub fn use_persistent<T: Serialize + DeserializeOwned + Default + 'static>(
    // A unique key for the storage entry
    key: impl ToString,
    // A function that returns the initial value if the storage entry is empty
    init: impl FnOnce() -> T,
) -> UsePersistent<T> {
    let state = use_signal(move || {
        let key = key.to_string();
        let value = LocalStorage::get(key.as_str()).ok().unwrap_or_else(init);
        StorageEntry { key, value }
    });

    // Wrap the state in a new struct
    UsePersistent { inner: state }
}

struct StorageEntry<T> {
    key: String,
    value: T,
}

/// Storage that persists across application reloads
pub struct UsePersistent<T: 'static> {
    inner: Signal<StorageEntry<T>>,
}

impl<T> Clone for UsePersistent<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <T> Copy for UsePersistent<T> {}

impl <T: Serialize + DeserializeOwned + Clone + 'static> UsePersistent<T> {
    /// Returns a reference to the value
    pub fn get(&self) -> T {
        self.inner.read().value.clone()
    }

    /// Sets the value
    pub fn set(&mut self, value: T) {
        let mut inner = self.inner.write();
        LocalStorage::set(inner.key.as_str(), &value).expect("Unable to write to LocalStorage");
        inner.value = value;
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Colorscheme {
    #[default]
    One,
    Two,
    Three,
}

impl Display for Colorscheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Colorscheme::One => { write!(f, "theme1") }
            Colorscheme::Two => { write!(f, "theme2") }
            Colorscheme::Three => { write!(f, "theme3") }
        }
    }
}

impl FromStr for Colorscheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "theme1" => Ok(Colorscheme::One),
            "theme2" => Ok(Colorscheme::Two),
            "theme3" => Ok(Colorscheme::Three),
            _ => Err("invalid colorscheme".into())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub(crate) colorscheme: Colorscheme,
    pub(crate) jwt: Option<String>,
    pub(crate) username: Option<String>,
}

impl AppState {
    pub fn switch_to_theme_one(&mut self) {
        self.colorscheme = Colorscheme::One;
    }
    pub fn switch_to_theme_two(&mut self) {
        self.colorscheme = Colorscheme::Two;
    }
    pub fn switch_to_theme_three(&mut self) {
        self.colorscheme = Colorscheme::Three;
    }
}
