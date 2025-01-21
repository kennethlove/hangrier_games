mod cache;
mod components;
mod routes;

use components::App;

use dioxus::prelude::*;

fn main() {
    launch(App);
}
