mod cache;
mod components;
mod routes;
mod storage;

use components::App;
use std::sync::LazyLock;

use dioxus::prelude::*;

static API_HOST: LazyLock<String> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    std::env::var("API_HOST").unwrap_or("http://localhost:3000".to_string())
});

fn main() {
    launch(App);
}
