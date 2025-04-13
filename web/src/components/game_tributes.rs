use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::tribute_edit::TributeEdit;
use crate::routes::Routes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::{Game, GameStatus};
use game::messages::GameMessage;
use game::tributes::Tribute;

async fn fetch_tributes(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tributes(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}/tributes", API_HOST, identifier))
            .await
            .unwrap();

        match response.json::<Vec<Tribute>>().await {
            Ok(tributes) => {
                QueryResult::Ok(QueryValue::Tributes(tributes))
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

async fn fetch_tribute_log(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::TributeDayLog(identifier, day)) = keys.first() {
        if let Some(QueryKey::Game(game_identifier)) = keys.last() {
            let response = reqwest::get(format!("{}/api/games/{}/log/{}/{}", API_HOST, game_identifier, day, identifier))
                .await
                .unwrap();

            match response.json::<Vec<GameMessage>>().await {
                Ok(messages) => {
                    QueryResult::Ok(QueryValue::Logs(messages))
                }
                Err(_) => QueryResult::Err(QueryError::TributeNotFound(identifier.to_string()))
            }
        } else {
            QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameTributes() -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let identifier = game.identifier.clone();

    let tribute_query = use_get_query(
        [
            QueryKey::Tributes(identifier.clone()),
            QueryKey::Game(identifier.clone())
        ],
        fetch_tributes,
    );

    match tribute_query.result().value() {
        QueryResult::Ok(QueryValue::Tributes(tributes)) => {
            rsx! {
                ul {
                    class: "grid gap-4 grid-cols-2",
                    for tribute in tributes {
                        GameTributeListMember {
                            tribute: tribute.clone()
                        }
                    }
                }
            }
        }
        QueryResult::Err(e) => {
            dioxus_logger::tracing::error!("{:?}", e);
            rsx! { p { "Something went wrong" } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! {} }
    }
}

#[component]
pub fn GameTributeListMember(tribute: Tribute) -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();

    let identifier = tribute.clone().identifier;

    let tribute_logs_query = use_get_query(
        [
            QueryKey::TributeDayLog(identifier.clone(), game.clone().day.unwrap_or_default()),
            QueryKey::Tribute(identifier.clone()),
            QueryKey::Game(game.clone().identifier)
        ],
        fetch_tribute_log,
    );

    let tribute_logs: Vec<GameMessage> = match tribute_logs_query.result().value() {
        QueryResult::Ok(QueryValue::Logs(logs)) => logs.clone(),
        _ => Vec::new()
    };

    rsx! {
        li {
            "data-alive": tribute.is_alive(),
            class: r#"
            border
            p-2
            theme1:data-[alive=true]:border-green-500
            theme1:data-[alive=false]:border-red-500
            theme1:text-stone-200

            theme2:data-[alive=true]:border-green-200
            theme2:data-[alive=false]:border-red-200
            theme2:text-green-200
            theme2:rounded-md

            theme3:data-[alive=true]:border-green-600
            theme3:data-[alive=false]:border-red-600
            theme3:border-2
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
                    flex-grow

                    theme1:font-[Cinzel]
                    theme1:text-lg

                    theme2:font-[Forum]
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

                        theme2:text-green-200/50
                        theme2:hover:text-green-200

                        theme3:hover:border-b-2
                        theme3:hover:border-yellow-500
                        theme3:hover:text-yellow-500
                        "#,
                        to: Routes::TributeDetail {
                            game_identifier: game.identifier.clone(),
                            tribute_identifier: tribute.identifier.clone()
                        },
                        "{tribute.name}"
                    }
                }

                if game.status == GameStatus::NotStarted {
                    TributeEdit {
                        identifier: tribute.clone().identifier,
                        district: tribute.district,
                        name: tribute.clone().name,
                    }
                }
            }

            dl {
                class: "text-sm grid grid-cols-2 gap-2",
                dt {
                    class: "text-sm",
                    "District",
                }
                dd {
                    class: "font-bold",
                    "{tribute.district}"
                }
                dt {
                    class: "text-sm",
                    "In the",
                }
                dd {
                    class: "font-bold",
                    "{tribute.area}"
                }
                dt {
                    class: "text-sm",
                    "Status",
                }
                dd {
                    class: "font-bold",
                    "{tribute.status}"
                }
                dt {
                    class: "text-sm",
                    "Health",
                }
                dd {
                    class: "font-bold",
                    "{tribute.attributes.health}"
                }
            }

            h5 {
                class: r#"
                mt-2
                theme1:text-amber-200

                theme2:text-green-200
                theme2:bg-green-800
                theme2:px-2

                theme3:border-0
                theme3:border-b-2
                theme3:border-gold-rich
                "#,

                "Items"
            }
            if tribute.clone().items.is_empty() {
                p {
                    class: "text-sm",
                    "No items"
                }
            } else {
                ul {
                    class: "text-sm",
                    for item in tribute.clone().items {
                        li {
                            img {
                                src: format!("/assets/icons/{}", item.as_icon()),
                                alt: "{item.name} icon",
                                title: "{item.name}",
                                class: "size-12",
                            }
                        }
                    }
                }
            }

            h5 {
                class: r#"
                mt-2
                theme1:text-amber-200

                theme2:text-green-200
                theme2:bg-green-800
                theme2:px-2

                theme3:border-0
                theme3:border-b-2
                theme3:border-gold-rich
                "#,

                "Log"
            }
            if tribute_logs.is_empty() {
                p {
                    class: "text-sm",
                    "No logs"
                }
            } else {
                ul {
                    class: "text-sm mt-2",
                    for log in tribute_logs {
                        li {
                            class: "mb-2",
                            "{log.content}"
                        }
                    }
                }
            }

        }
    }
}
