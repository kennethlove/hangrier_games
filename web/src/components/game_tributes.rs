use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::tribute_delete::TributeDelete;
use crate::components::tribute_edit::{TributeEdit, EditTributeModal};
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::games::{Game, GAME};
use game::tributes::Tribute;
use shared::EditTribute;

async fn fetch_game_tributes(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Tributes(name)) = keys.first() {
        let response = reqwest::get(
            format!("{}/api/games/{}/tributes", API_HOST.clone(), name)
        ).await.expect("failed to fetch game tributes");

        match response.json::<Vec<Tribute>>().await {
            Ok(tributes) => {
                QueryResult::Ok(QueryValue::Tributes(tributes))
            }
            Err(_) => {
                QueryResult::Err(QueryError::GameNotFound(name.to_string()))
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameTributes(game_name: String) -> Element {
    let tributes_query = use_get_query(
        [QueryKey::Tributes(game_name.clone())],
        fetch_game_tributes
    );

    let edit_tribute_signal: Signal<Option<EditTribute>> = use_signal(|| None);
    use_context_provider(|| edit_tribute_signal);

    match tributes_query.result().value() {
        QueryResult::Ok(QueryValue::Tributes(tributes)) => {
            let tribute_count = &tributes.len();

            rsx! {
                ul {
                    for tribute in tributes {
                        li {
                            "{tribute.name} - {tribute.district}",
                            TributeEdit {
                                name: tribute.clone().name,
                                district: tribute.district,
                                identifier: tribute.clone().identifier,
                            }
                        }
                    }
                }

                EditTributeModal {}
            }
        },
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx!("")
        }
    }
}
