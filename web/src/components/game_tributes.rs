use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::icons::map_pin::MapPinIcon;
use crate::components::item_icon::ItemIcon;
use crate::components::tribute_edit::TributeEdit;
use crate::components::tribute_status_icon::TributeStatusIcon;
use crate::routes::Routes;
use crate::storage::{use_persistent, AppState};
use crate::env::APP_API_HOST as API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::{Game, GameStatus};
use game::items::Item;
use game::messages::GameMessage;
use game::tributes::Tribute;

async fn fetch_tributes(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tributes(identifier)) = keys.first() {
        let client = reqwest::Client::new();

        let request = client.request(
            reqwest::Method::GET,
            format!("{}/api/games/{}/tributes", &*API_HOST, identifier))
            .bearer_auth(token);

        match request.send().await {
            Ok(response) =>  {
                if let Ok(tributes) = response.json::<Vec<Tribute>>().await {
                    QueryResult::Ok(QueryValue::Tributes(tributes))
                } else {
                    QueryResult::Err(QueryError::BadJson)
                }
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

async fn _fetch_tribute_log(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::_TributeDayLog(identifier, day)) = keys.first() {
        if let Some(QueryKey::Game(game_identifier)) = keys.last() {
            let client = reqwest::Client::new();

            let request = client.request(
                reqwest::Method::GET,
                format!("{}/api/games/{}/log/{}/{}", &*API_HOST, game_identifier, day, identifier))
                .bearer_auth(token);

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let messages = response.json::<Vec<GameMessage>>().await;
                        match messages {
                            Ok(messages) => {
                                QueryResult::Ok(QueryValue::Logs(messages))
                            }
                            Err(_) => QueryResult::Err(QueryError::BadJson)
                        }
                    } else {
                        QueryResult::Err(QueryError::TributeNotFound(identifier.to_string()))
                    }
                }
                Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
            }
        } else {
            QueryResult::Err(QueryError::Unknown)
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameTributes(game: Game) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");

    let identifier = game.identifier.clone();

    let tribute_query = use_get_query(
        [
            QueryKey::Tributes(identifier.clone()),
            QueryKey::Game(identifier.clone())
        ],
        move |keys: Vec<QueryKey>| { fetch_tributes(keys, token.clone()) },
    );

    match tribute_query.result().value() {
        QueryResult::Ok(QueryValue::Tributes(tributes)) => {
            rsx! {
                ul {
                    class: "grid gap-2 grid-cols-2",
                    for chunk in tributes.as_slice().chunks(2) {
                        li {
                            class: r#"
                            col-span-2
                            pb-4
                            theme1:not-last-of-type:border-b-2
                            theme1:border-amber-900
                            theme2:not-last-of-type:border-b-1
                            theme2:border-dotted
                            theme2:border-green-200
                            theme3:not-last-of-type:border-b-2
                            theme3:border-gold-rich
                            "#,
                            h3 {
                                class: r#"
                                text-xl
                                text-center
                                mb-2
                                theme1:text-stone-200
                                theme1:font-[Cinzel]
                                theme2:text-green-200
                                theme2:font-[Playfair_Display]
                                theme3:text-yellow-700
                                theme3:drop-shadow-sm
                                theme3:font-[Orbitron]
                                "#,
                                "District {chunk.first().unwrap().district}"
                            }
                            ul {
                                class: "grid subgrid gap-2 grid-cols-2",
                                GameTributeListMember {
                                    tribute: chunk.first().unwrap().clone(),
                                    game_identifier: identifier.clone(),
                                    game_status: game.status.clone(),
                                }
                                GameTributeListMember {
                                    tribute: chunk.last().unwrap().clone(),
                                    game_identifier: identifier.clone(),
                                    game_status: game.status.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
        QueryResult::Err(_) => {
            rsx! { p { "Something went wrong" } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! {} }
    }
}

#[component]
pub fn GameTributeListMember(tribute: Tribute, game_identifier: String, game_status: GameStatus) -> Element {
    let fist_item = Item::new_weapon("basic fist");

    rsx! {
        li {
            "data-alive": tribute.is_alive(),
            class: r#"
            border
            p-2
            self-start
            overflow-hidden

            theme1:border-1
            theme1:text-stone-200
            theme1:data-[alive=false]:border-red-500/50
            theme1:motion-safe:data-[alive=true]:border-tracer
            theme1:motion-safe:data-[alive=true]:border-2
            theme1:motion-reduce:data-[alive=true]:border-green-500

            theme2:data-[alive=true]:border-green-400
            theme2:data-[alive=false]:border-red-400
            theme2:data-[alive=false]:bg-linear-to-r
            theme2:data-[alive=false]:from-green-900
            theme2:data-[alive=false]:to-red-900/75
            theme2:text-green-200
            theme2:rounded-md

            theme3:border-2
            theme3:data-[alive=true]:border-gold-rich
            theme3:data-[alive=false]:opacity-50
            "#,

            div {
                class: r#"
                flex
                flex-row
                gap-2
                place-content-between
                "#,

                h4 {
                    class: r#"
                    mb-2

                    theme1:font-[Cinzel]
                    theme1:text-lg

                    theme2:font-[Playfair_Display]
                    theme2:text-xl
                    theme2:text-green-200
                    theme2:hover:underline
                    theme2:hover:decoration-2
                    theme2:hover:decoration-wavy

                    theme3:font-semibold
                    "#,

                    Link {
                        class: r#"
                        theme1:font-semibold
                        theme1:text-xl
                        theme1:text-amber-500
                        theme1:hover:text-amber-200
                        theme1:hover:border-b-2
                        theme1:hover:border-amber-500

                        theme2:text-green-200
                        theme2:hover:text-green-200

                        theme3:hover:border-b-2
                        theme3:hover:border-yellow-500
                        theme3:hover:text-yellow-500
                        "#,
                        to: Routes::TributeDetail {
                            game_identifier: game_identifier.clone(),
                            tribute_identifier: tribute.identifier.clone()
                        },
                        "{tribute.name}"
                    }
                }

                if game_status == GameStatus::NotStarted {
                    div {
                        TributeEdit {
                            identifier: tribute.clone().identifier,
                            district: tribute.district,
                            name: tribute.clone().name,
                        }
                    }
                } else {
                    div {
                        class: r#"
                        "#,

                        TributeStatusIcon {
                            status: tribute.status.clone(),
                            css_class: r#"
                            inline-block
                            size-5
                            theme1:fill-stone-200
                            theme2:fill-green-200
                            "#
                        }
                    }
                }
            }

            div {
                class: r#"
                text-sm
                flex
                flex-row
                gap-2
                place-items-center
                mb-2
                "#,
                MapPinIcon {
                    class: r#"
                    size-6
                    ml-1
                    theme1:fill-amber-500
                    theme2:fill-green-200
                    "#,
                }
                span {
                    class: "",
                    "{tribute.area}"
                }
            }

            ul {
                class: "flex flex-row gap-2 flex-wrap",
                if tribute.clone().items.is_empty() {
                    li {
                        class: "flex flex-row gap-2 flex-wrap place-items-center",
                        ItemIcon {
                            item: fist_item,
                            css_class: r#"
                            size-8
                            theme1:fill-amber-500
                            theme2:fill-green-200
                            "#,
                        }
                        span {
                            class: "text-sm",
                            "Fist"
                        }
                    }
                } else {
                    for item in tribute.clone().items {
                        li {
                            class: "flex flex-row gap-2 flex-wrap place-items-center",
                            ItemIcon {
                                item: item.clone(),
                                css_class: r#"
                                size-8
                                theme1:fill-amber-500
                                theme2:fill-green-200
                                "#,
                            }
                            span {
                                class: "text-sm capitalize",
                                "{item.to_string()}"
                            }
                        }
                    }
                }
            }
        }
    }
}
