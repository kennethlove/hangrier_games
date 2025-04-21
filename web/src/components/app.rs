use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_edit::EditGameModal;
use crate::components::tribute_edit::EditTributeModal;
use crate::routes::Routes;
use crate::storage::{use_persistent, AppState, Colorscheme};
use dioxus::prelude::*;
use dioxus_query::prelude::use_init_query_client;
use game::games::Game;
use shared::{DeleteGame, EditGame, EditTribute};

#[component]
pub fn App() -> Element {
    use_init_query_client::<QueryValue, QueryError, QueryKey>();

    let storage = use_persistent("hangry-games", AppState::default);

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

    let server_version = "0.1.8";
    let client_version = env!("CARGO_PKG_VERSION");

    let favicon = match *theme_signal.read() {
        Colorscheme::One => asset!("/assets/favicons/theme1.png"),
        Colorscheme::Two => asset!("/assets/favicons/theme2.png"),
        Colorscheme::Three => asset!("/assets/favicons/theme3.png"),
    };

    rsx! {
        document::Link {
            rel: "icon",
            href: favicon,
            r#type: "image/png"
        }

        document::Link {
            rel: "preconnect",
            href: "https://fonts.googleapis.com",
        }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "anonymous"
        }
        document::Link {
            href: "https://fonts.googleapis.com/css2?family=Cinzel:wght@400..900&family=Work+Sans:ital,wght@0,100..900;1,100..900&family=Orbitron:wght@400..900&family=Playfair+Display:ital,wght@0,400..900;1,400..900&display=swap",
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
                min-v-full
                min-h-screen
                frame
                transition
                duration-500
                font-[Work_Sans]

                theme1:bg-red-900/85

                theme2:bg-green-800/85
                theme2:bg-[url("../assets/images/waves.svg")]
                theme2:bg-no-repeat
                theme2:bg-origin-border
                theme2:bg-bottom
                theme2:bg-size-[3200px_1311px]
                theme2:bg-fixed

                theme3:bg-linear-to-b
                theme3:from-stone-50/80
                theme3:to-stone-900/95
                "#,

                div {
                    class: r#"
                    py-4
                    px-2
                    "#,
                    Router::<Routes> {}
                }

                footer {
                    class: r#"
                    mt-4
                    pb-4
                    text-xs
                    text-center
                    theme1:text-stone-200
                    theme2:text-green-900
                    theme3:text-stone-400
                    "#,

                    p {
                        "Made with 💜 by ",
                        a {
                            class: "theme1:text-amber-300 theme2:text-green-200 theme3:text-yellow-600",
                            href: "https://thekennethlove.com",
                            "klove"
                        },
                        ". ",
                        a {
                            class: "theme1:text-amber-300 theme2:text-green-200 theme3:text-yellow-600",
                            href: "/credits",
                            "Credits"
                        }
                    }
                    p {
                        "Server version: {server_version}; Client version: {client_version}",
                    }
                }
            }

            EditGameModal {}
            EditTributeModal {}
        }
    }
}
