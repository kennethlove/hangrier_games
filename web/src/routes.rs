use dioxus::prelude::*;
use dioxus_query::prelude::use_query_client;
use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::{App, Games, GamesList, GameDetail, CreateGameButton, CreateGameForm, DeleteGameModal};


#[component]
fn Home() -> Element {
    rsx! {
        p { "May the odds be ever in your favor!" }
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "Page not found" }
        p { "Sorry, don't know what you were looking for" }
        pre { "log\nattempted to navigate to: {route:?}" }
    }
}

#[component]
fn NavBar() -> Element {
    rsx! {
        nav {
            ul {
                li {
                    Link { to: Routes::Home {}, "Home" }
                }
                li {
                    Link { to: Routes::GamesList {}, "Games" }
                }
            }
        }
        Outlet::<Routes> {}
    }
}

#[rustfmt::skip]
#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Routes {
    #[layout(NavBar)]
        #[route("/")]
        Home {},
        #[nest("/games")]
            #[layout(Games)]
                #[route("/")]
                GamesList {},
                #[route("/:name")]
                GameDetail { name: String },
            #[end_layout]
        #[end_nest]
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}
