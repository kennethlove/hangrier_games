mod cache;
mod components;
mod routes;

use components::App;
use std::sync::LazyLock;

use dioxus::prelude::*;
use dotenvy_macro::dotenv;

static API_HOST: LazyLock<String> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    dotenv!("API_HOST").parse().unwrap_or(String::from("http://localhost"))
});

fn main() {
    launch(App);
}
