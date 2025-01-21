mod cache;
mod components;

use components::App;

use dioxus::prelude::*;


fn main() {
    launch(App);
}
