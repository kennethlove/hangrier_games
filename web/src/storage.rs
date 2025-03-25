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

fn get_saved_state(storage: UsePersistent<AppState>) -> AppState {
    let mut state = AppState::default();
    state
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub(crate) dark_mode: bool,
}

impl AppState {
    pub(crate) fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
    }
}
