use std::env;
use std::collections::HashMap;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use game::games::Game;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dotenvy::dotenv().expect("Failed to initialize dotenvy.");
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    use_resource(move || async move {
        dioxus_logger::tracing::info!("starting up");
        match get_games().await {
            Ok(games) => {
                dioxus_logger::tracing::debug!("component {:?}", games);
                rsx! {
                    for game in games {
                        "{game}"
                    }
                }
            }
            Err(e) => {
                rsx! {}
            }
        }
    });

    rsx! {
        Hero {}
        Echo {}
    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            // Navigation links
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[server(EchoServer)]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[derive(Debug, Deserialize, Serialize)]
struct GameRecord {
    id: String,
}

#[server]
async fn get_games() -> Result<Vec<String>, ServerFnError> {
    dioxus_logger::tracing::debug!("get_games");
    let surreal: Surreal<Client> = Surreal::init();
    surreal.connect::<Wss>("http://surrealdb.eyeheartzombies.com").await?;
    surreal.signin(Root {
        username: &env::var("SURREAL_USER")?,
        password: &env::var("SURREAL_PASS")?,
    }).await.unwrap();
    surreal.use_ns("hangry-games").use_db("games").await?;
    dioxus_logger::tracing::debug!("{:?}", surreal);

    match surreal.select("game").await {
        Ok(games) => {
            dioxus_logger::tracing::debug!("{:?}", games);
            Ok(games.into_iter().map(|g: GameRecord| g.id).collect())
        },
        Err(e) => {
            dioxus_logger::tracing::debug!("{:?}", e);
            Err(ServerFnError::Request("failed to get games".to_string()))
        },
    }
}
