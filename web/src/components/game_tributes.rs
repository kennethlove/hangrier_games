use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::tribute_edit::TributeEdit;
use crate::routes::Routes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::Game;
use game::messages::GameMessage;
use game::tributes::Tribute;

async fn fetch_tributes(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tributes(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}/tributes", API_HOST.clone(), identifier))
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
            let response = reqwest::get(format!("{}/api/games/{}/log/{}/{}", API_HOST.clone(), game_identifier, day, identifier))
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
            font-[Work_Sans]
            theme2:data-[alive=true]:border-green-200
            theme2:data-[alive=false]:border-red-200
            theme2:text-green-200
            theme2:rounded-md
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
                    text-lg
                    mb-2
                    theme2:font-[Forum]
                    theme2:text-xl
                    theme2:text-green-200
                    theme2:hover:underline
                    theme2:hover:decoration-2
                    theme2:hover:decoration-wavy
                    "#,

                    Link {
                        to: Routes::TributeDetail {
                            game_identifier: game.identifier.clone(),
                            tribute_identifier: tribute.identifier.clone()
                        },
                        "{tribute.name}"
                    }
                }
                TributeEdit {
                    identifier: tribute.clone().identifier,
                    district: tribute.district,
                    name: tribute.clone().name,
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
                theme2:text-green-200
                theme2:bg-green-800
                theme2:px-2
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
                        li { "{item.name}" }
                    }
                }
            }

            h5 {
                class: r#"
                mt-2
                theme2:text-green-200
                theme2:bg-green-800
                theme2:px-2
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
