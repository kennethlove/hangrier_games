use dioxus::dioxus_core::AttributeValue;
use dioxus::prelude::*;
use crate::routes::Routes;
use crate::storage::{use_persistent, AppState};

#[component]
pub fn Navbar() -> Element {
    let mut storage = use_persistent("hangry-games", || AppState::default());
    let mut dark_mode_signal: Signal<bool> = use_context();

    let future = use_resource(move || async move {
        let mut eval = document::eval(
            r#"
            "#,
        );
    });

    rsx! {
        div {
            class: "flex flex-row place-content-between mb-4",
            h1 {
                class: "text-3xl cinzel-font",
                "Hangry Games"
            }

            nav {
                ul {
                    class: "flex flex-row gap-2",
                    li {
                        Link { to: Routes::Home {}, "Home" }
                    }
                    li {
                        Link { to: Routes::GamesList {}, "Games" }
                    }
                    li {
                        span {
                            i {
                                class: "ra ra-light-bulb text-gray-800 dark:text-gray-50",
                                onclick: move |_| {
                                    let mut state = storage.get();
                                    state.toggle_dark_mode();
                                    storage.set(state.clone());
                                    dark_mode_signal.set(state.dark_mode);
                                }
                            }
                        }
                    }
                }
            }
        }
        Outlet::<Routes> {}
    }
}

