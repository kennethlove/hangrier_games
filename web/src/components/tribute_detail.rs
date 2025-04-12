use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::tribute_edit::TributeEdit;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::Game;
use game::tributes::Tribute;
use std::collections::HashMap;

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

#[component]
pub fn TributeDetail(game_identifier: String, tribute_identifier: String) -> Element {
    let mut tribute_game: Game;

    let game_query = use_get_query(
        [QueryKey::Game(game_identifier.clone()), QueryKey::Games],
        crate::components::game_detail::fetch_game,
    );

    match game_query.result().value() {
        QueryResult::Ok(QueryValue::Game(game)) => {
            let game_identifier = game.identifier.clone();

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
                h1 {
                    "{tribute.name}"
                    TributeEdit {
                        identifier: tribute.clone().identifier,
                        district: tribute.district,
                        name: tribute.clone().name,
                    }
                }
                h2 { "District {tribute.district}" }
                h3 { "Current location: {tribute.area}" }
                h3 { "Items" }
                ul {
                    for item in tribute.clone().items {
                        li {
                            onclick: move |_| {},
                            "{item.name}" }
                    }
                }

                h3 { "Attributes" }
                dl {
                    class: "grid grid-cols-2 gap-4",
                    dt { "Health" }
                    dd { "{tribute.attributes.health}"}
                    dt { "Sanity" }
                    dd { "{tribute.attributes.sanity}"}
                    dt { "Movement" }
                    dd { "{tribute.attributes.movement}"}
                    dt { "Strength" }
                    dd { "{tribute.attributes.strength}"}
                    dt { "Defense" }
                    dd { "{tribute.attributes.defense}"}
                    dt { "Bravery" }
                    dd { "{tribute.attributes.bravery}"}
                    dt { "Loyalty" }
                    dd { "{tribute.attributes.loyalty}"}
                    dt { "Speed" }
                    dd { "{tribute.attributes.speed}"}
                    dt { "Dexterity" }
                    dd { "{tribute.attributes.dexterity}"}
                    dt { "Intelligence" }
                    dd { "{tribute.attributes.intelligence}"}
                    dt { "Persuasion" }
                    dd { "{tribute.attributes.persuasion}"}
                    dt { "Luck" }
                    dd { "{tribute.attributes.luck}"}
                    dt { "Hidden?" }
                    dd { "{tribute.attributes.is_hidden}"}
                }

                h3 { "Log" }
            }
                },
                QueryResult::Err(QueryError::TributeNotFound(identifier)) => {
                    rsx! { p { "{identifier} not found." } }
                },
                QueryResult::Loading(_) => {
                    rsx! { p { "Loading..." } }
                },
                _ => { rsx! { } }
            }
        }
        QueryResult::Err(_) => {
            rsx! {
                p { "Game not found." }
            }
        }
        QueryResult::Loading(_) => {
            rsx! {
                p { "Loading..." }
            }
        }
        _ => {
            rsx! {}
        }
    }
}
