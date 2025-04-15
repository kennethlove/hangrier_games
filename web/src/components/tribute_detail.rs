use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::game_detail::InfoDetail;
use crate::components::icons::uturn::UTurnIcon;
use crate::components::tribute_edit::{EditTributeModal, TributeEdit};
use crate::routes::Routes;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::Game;
use game::messages::GameMessage;
use game::tributes::{Attributes, Tribute};
use shared::EditTribute;
use std::collections::HashMap;
use crate::components::icons::game_icons_net::HeartsIcon;
use crate::components::tribute_status_icon::TributeStatusIcon;

async fn fetch_tribute(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tribute(identifier)) = keys.first() {
        if let Some(QueryKey::Game(game_identifier)) = keys.last() {
            let response = reqwest::get(
                format!(
                    "{}/api/games/{}/tributes/{}",
                    API_HOST,
                    game_identifier,
                    identifier
                ))
                .await
                .unwrap();

            match response.json::<Option<Tribute>>().await {
                Ok(Some(tribute)) => {
                    QueryResult::Ok(QueryValue::Tribute(Box::new(tribute)))
                }
                _ => QueryResult::Err(QueryError::TributeNotFound(identifier.to_string()))
            }
        } else {
            QueryResult::Err(QueryError::Unknown)
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

async fn fetch_tribute_log(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::TributeLog(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/tributes/{}/log", API_HOST, identifier))
            .await
            .unwrap();

        match response.json::<Vec<GameMessage>>().await {
            Ok(logs) => {
                QueryResult::Ok(QueryValue::Logs(logs))
            }
            Err(_err) => {
                QueryResult::Err(QueryError::TributeNotFound(identifier.to_string()))
            },
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}


#[component]
pub fn TributeDetail(game_identifier: String, tribute_identifier: String) -> Element {
    let tribute_query = use_get_query(
        [
            QueryKey::Tribute(tribute_identifier.clone()),
            QueryKey::Tributes(game_identifier.clone()),
            QueryKey::Game(game_identifier.clone()),
        ],
        fetch_tribute,
    );

    match tribute_query.result().value() {
        QueryResult::Ok(QueryValue::Tribute(tribute)) => {
            rsx! {
                div {
                    class: "flex flex-row gap-2 place-content-between mb-4",
                    h2 {
                        class: r#"
                        theme1:text-2xl
                        theme1:font-[Cinzel]
                        theme1:text-amber-300

                        theme2:font-[Forum]
                        theme2:text-3xl
                        theme2:text-green-200

                        theme3:font-[Orbitron]
                        theme3:text-2xl
                        theme3:text-stone-700
                        "#,

                        "{tribute.name}"
                    }

                    span {
                        Link {
                            to: Routes::GamePage {
                                identifier: game_identifier.clone()
                            },
                            UTurnIcon {
                                class: r#"
                                size-4
                                theme1:fill-amber-500
                                theme1:hover:fill-amber-200

                                theme2:fill-green-200/50
                                theme2:hover:fill-green-200

                                theme3:fill-amber-600/50
                                theme3:hover:fill-amber-600
                                "#,
                            }
                        }
                    }
                }

                div {
                    class: r#"
                    pr-2
                    grid
                    gap-8
                    grid-cols-none
                    sm:grid-cols-2
                    lg:grid-cols-3
                    2xl:grid-cols-4

                    theme1:text-stone-200
                    theme2:text-green-200
                    "#,

                    InfoDetail {
                        title: "Overview",
                        open: true,
                        dl {
                            class: "grid grid-cols-2 gap-4",
                            dt { "District" }
                            dd { "{tribute.district}" }
                            dt { "Current location" }
                            dd { "{tribute.area}" }
                            dt { "Status" }
                            dd {
                                TributeStatusIcon {
                                    status: tribute.status.clone(),
                                    css_class: "size-8",
                                }
                            }
                            dt { "Outlook" }
                            dd { "TODO" }
                        }
                    }

                    InfoDetail {
                        title: "Inventory",
                        open: false,
                        ul {
                            for item in tribute.clone().items {
                                li {
                                    img {
                                        src: format!("/assets/icons/{}", item.as_icon()),
                                        alt: "{item.name} icon",
                                        title: "{item.name}",
                                        class: "size-16",
                                    }
                                }
                            }
                        }
                    }

                    InfoDetail {
                        title: "Attributes",
                        open: false,
                        TributeAttributes { attributes: tribute.attributes.clone() }
                    }

                    if !tribute.clone().editable {
                        InfoDetail {
                            title: "Log",
                            open: false,
                            TributeLog {
                                identifier: tribute.clone().identifier,
                            }
                        }
                    }
                }
            }
        }
        QueryResult::Err(QueryError::TributeNotFound(identifier)) => {
            rsx! { p { "{identifier} not found." } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! { } }
    }
}

#[component]
fn TributeLog(identifier: String) -> Element {
    let log_query = use_get_query(
        [
            QueryKey::TributeLog(identifier.clone()),
            QueryKey::Tribute(identifier.clone()),
        ],
        fetch_tribute_log,
    );

    match log_query.result().value() {
        QueryResult::Ok(QueryValue::Logs(logs)) => {
            rsx! {
                ul {
                    class: "theme1:text-stone-200 theme2:text-green-200 theme3:text-stone-800",
                    for log in logs {
                        li {
                            p {
                                class: "text-sm",
                                "Day {log.game_day}"
                            }
                            "{log.content}"
                        }
                    }
                }
            }
        }
        QueryResult::Err(_) => { rsx! { p { "Failed to load." }  } }
        QueryResult::Loading(_) => { rsx! { p { "Loading..." }  } }
        _ => { rsx! {} }
    }
}

#[component]
fn TributeAttributes(attributes: Attributes) -> Element {
    rsx! {
        dl {
            class: "grid grid-cols-2 gap-4",
            dt { "Health" }
            dd { "{attributes.health}"}
            dt { "Sanity" }
            dd { "{attributes.sanity}"}
            dt { "Movement" }
            dd { "{attributes.movement}"}
            dt { "Strength" }
            dd { "{attributes.strength}"}
            dt { "Defense" }
            dd { "{attributes.defense}"}
            dt { "Bravery" }
            dd { "{attributes.bravery}"}
            dt { "Loyalty" }
            dd { "{attributes.loyalty}"}
            dt { "Speed" }
            dd { "{attributes.speed}"}
            dt { "Dexterity" }
            dd { "{attributes.dexterity}"}
            dt { "Intelligence" }
            dd { "{attributes.intelligence}"}
            dt { "Persuasion" }
            dd { "{attributes.persuasion}"}
            dt { "Luck" }
            dd { "{attributes.luck}"}
        }
    }
}
