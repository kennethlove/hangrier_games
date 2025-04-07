mod cache;
mod components;
mod routes;
mod storage;

use components::App;
use std::sync::LazyLock;

use dioxus::prelude::*;

static API_HOST: &str = env!("API_HOST");

fn main() {
    launch(App);
}
