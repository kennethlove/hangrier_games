use dioxus::dioxus_core::AttributeValue;
use dioxus::document::Script;
use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_edit::EditGameModal;
use crate::components::tribute_edit::EditTributeModal;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::use_init_query_client;
use game::games::Game;
use shared::{DeleteGame, EditGame, EditTribute};
use crate::storage::{use_persistent, AppState, Colorscheme};

#[component]
pub fn App() -> Element {
    use_init_query_client::<QueryValue, QueryError, QueryKey>();

    let mut storage = use_persistent("hangry-games", || AppState::default());

    let theme_signal: Signal<Colorscheme> = use_signal(|| storage.get().colorscheme);
    use_context_provider(|| theme_signal);

    let game_signal: Signal<Option<Game>> = use_signal(|| None);
    use_context_provider(|| game_signal);

    let delete_game_signal: Signal<Option<DeleteGame>> = use_signal(|| None);
    use_context_provider(|| delete_game_signal);

    let edit_game_signal: Signal<Option<EditGame>> = use_signal(|| None);
    use_context_provider(|| edit_game_signal);
    
    let edit_tribute_signal: Signal<Option<EditTribute>> = use_signal(|| None);
    use_context_provider(|| edit_tribute_signal);

    let copyright = "Hangry Games &copy; 2025";

    rsx! {
        document::Link {
            rel: "preconnect",
            href: "https://api.fonts.coollabs.io",
            crossorigin: Some("true".into())
        }
        document::Link {
            href: "https://api.fonts.coollabs.io/css2?family=Cinzel:wght@400..900&display=swap",
            rel: "stylesheet"
        }
        document::Link {
            href: "https://api.fonts.coollabs.io/icon?family=Material+Icons",
            rel: "stylesheet"
        }

        document::Stylesheet {
            href: asset!("/assets/dist/main.css")
        }

        div {
            class: "{theme_signal.read()}",
            div {
                class: r#"
                grid
                grid-flow-row
                min-v-full
                min-h-screen
                frame
                transition
                duration-250
                p-2
                theme1:bg-red-900
                theme2:bg-green-200
                theme3:bg-slate-600
                "#,

                Router::<Routes> {}

                footer {
                    class: "text-xs text-center theme1:text-stone-950 theme2:text-green-900 theme3:text-slate-800",
                    p { dangerous_inner_html: "{copyright}" }
                    p {
                        "Three finger salute icon by "
                        a {
                            href: "https://thenounproject.com/browse/icons/term/three-finger-salute/",
                            "Till Teenck"
                        },
                        " (CC BY 3.0)"
                    }
                    p {
                        "Mockingjay icons from "
                        a {
                            href: "https://www.vecteezy.com/members/inna-marchenko601727",
                            "Inna Marchenko"
                        }
                    }
                }
            }

            EditGameModal {}
            EditTributeModal {}
        }
    }
}

