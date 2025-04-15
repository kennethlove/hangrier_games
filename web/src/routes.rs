use crate::components::{Credits, GamePage, Games, GamesList, Home, IconsPage, Navbar, TributeDetail};
use dioxus::prelude::*;


#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "Page not found" }
        p { "Sorry, don't know what you were looking for" }
        pre { "log\nattempted to navigate to: {route:?}" }
    }
}

#[rustfmt::skip]
#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Routes {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        #[nest("/games")]
            #[layout(Games)]
                #[route("/")]
                GamesList {},
                #[route("/:identifier")]
                GamePage { identifier: String },
                #[route("/:game_identifier/tributes/:tribute_identifier")]
                TributeDetail { game_identifier: String, tribute_identifier: String },
            #[end_layout]
        #[end_nest]
        #[route("/credits")]
        Credits {},
        #[route("/icons")]
        IconsPage {},
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}
