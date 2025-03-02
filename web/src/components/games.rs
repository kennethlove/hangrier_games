use crate::routes::Routes;
use dioxus::prelude::*;

#[component]
pub fn Games() -> Element {
    rsx! {
        div {
            id: "games",
            Outlet::<Routes> {}
        }
    }
}

