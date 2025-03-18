use dioxus::prelude::*;
use crate::routes::Routes;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            class: "flex flex-row place-content-between mb-4",
            h1 {
                class: "text-3xl",
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
                }
            }
        }
        Outlet::<Routes> {}
    }
}

