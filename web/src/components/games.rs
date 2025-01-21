use dioxus::prelude::*;
use crate::components::{CreateGameButton, CreateGameForm, DeleteGameModal};
use crate::routes::Routes;

#[component]
pub fn Games() -> Element {
    rsx! {
        div {
            id: "games",
            Outlet::<Routes> {}
        }
    }
}

