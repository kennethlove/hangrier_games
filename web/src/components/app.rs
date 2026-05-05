use crate::LoadingState;
use crate::components::game_edit::EditGameModal;
use crate::components::loading_modal::LoadingModal;
use crate::components::server_version::ServerVersion;
use crate::components::tribute_edit::EditTributeModal;
use crate::icons::SPRITE;
use crate::routes::Routes;
use crate::storage::{AppState, use_persistent};
use crate::theme::Theme;
use dioxus::prelude::*;
use game::games::Game;
use shared::{DeleteGame, EditGame, EditTribute};

#[component]
pub fn App() -> Element {
    let storage = use_persistent("hangry-games", AppState::default);

    let loading_signal: Signal<LoadingState> = use_signal(LoadingState::default);
    use_context_provider(|| loading_signal);

    let theme_signal: Signal<Theme> = use_signal(|| storage.get().theme);
    use_context_provider(|| theme_signal);

    let game_signal: Signal<Option<Game>> = use_signal(|| None);
    use_context_provider(|| game_signal);

    let delete_game_signal: Signal<Option<DeleteGame>> = use_signal(|| None);
    use_context_provider(|| delete_game_signal);

    let edit_game_signal: Signal<Option<EditGame>> = use_signal(|| None);
    use_context_provider(|| edit_game_signal);

    let edit_tribute_signal: Signal<Option<EditTribute>> = use_signal(|| None);
    use_context_provider(|| edit_tribute_signal);

    let client_version = env!("CARGO_PKG_VERSION");

    let favicon = match *theme_signal.read() {
        Theme::Dark => asset!("/assets/favicons/dark.png"),
        Theme::Light => asset!("/assets/favicons/light.png"),
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
            href: "https://fonts.googleapis.com/css2?family=Bebas+Neue&family=Source+Sans+3:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500;700&display=swap",
            rel: "stylesheet"
        }

        document::Stylesheet {
            href: asset!("/assets/dist/main.css")
        }

        // Inline the icon sprite once at the root so <use href="#..."/> resolves
        // without an async fetch.
        div {
            id: "icon-sprite",
            style: "display:none",
            dangerous_inner_html: SPRITE,
        }

        div {
            class: "{theme_signal.read()}",
            div {
                class: "grid min-h-screen frame transition duration-500 font-text bg-bg text-text",

                div {
                    class: r#"
                    py-4
                    px-2
                    "#,
                    Router::<Routes> {}
                }

                footer {
                    class: "mt-4 pb-4 text-xs text-center text-text-muted",

                    p {
                        "Made with 💜 by ",
                        a {
                            class: "underline text-primary",
                            href: "https://thekennethlove.com",
                            "klove"
                        },
                        ". ",
                        a {
                            class: "underline text-primary",
                            href: "/credits",
                            "Credits"
                        }
                    }
                    p {
                        ServerVersion {},
                        "; Client: v{client_version}",
                    }
                }
            }

            EditGameModal {}
            EditTributeModal {}
            LoadingModal {}
        }
    }
}
