use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::icons::lock_closed::LockClosedIcon;
use crate::components::icons::lock_open::LockOpenIcon;
use crate::components::map::Map;
use crate::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_get_query, QueryResult};
use game::areas::AreaDetails;
use game::games::Game;
use crate::components::item_icon::ItemIcon;
use crate::storage::{use_persistent, AppState};

async fn fetch_areas(keys: Vec<QueryKey>, token: String) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Areas(identifier)) = keys.first() {
        let client = reqwest::Client::new();

        let request = client.request(
            reqwest::Method::GET,
            format!("{}/api/games/{}/areas", API_HOST, identifier))
            .bearer_auth(token);

        match request.send().await {
            Ok(response) => {
                match response.json::<Vec<AreaDetails>>().await {
                    Ok(areas) => {
                        dioxus_logger::tracing::debug!("Areas: {:?}", areas);
                        QueryResult::Ok(QueryValue::Areas(areas))
                    }
                    Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
                }
            }
            Err(_) => {
                QueryResult::Err(QueryError::GameNotFound(identifier.to_string()))
            }
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameAreaList(game: Game) -> Element {
    let mut storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");

    let identifier = game.identifier.clone();

    let area_query = use_get_query(
        [
            QueryKey::Areas(identifier.clone()),
            QueryKey::Game(identifier.clone()),
            QueryKey::Games
        ],
        move |keys: Vec<QueryKey>| { fetch_areas(keys, token.clone()) },
    );

    match area_query.result().value() {
        QueryResult::Ok(QueryValue::Areas(areas)) => {
            rsx! {
                ul {
                    class: "grid grid-cols-2 gap-4",
                    li {
                        Map { areas: areas.clone() }
                    }
                    for area in areas {
                        li {
                            "data-open": area.open(),
                            class: r#"
                            border
                            p-2
                            theme1:data-[open=true]:border-green-500
                            theme1:data-[open=false]:border-red-500
                            theme1:text-stone-200

                            theme2:border-3
                            theme2:rounded-md
                            theme2:bg-green-200
                            theme2:data-[open=true]:border-green-500
                            theme2:data-[open=false]:border-red-400

                            theme3:border-2
                            theme3:data-[open=true]:border-green-600
                            theme3:data-[open=false]:border-red-500
                            "#,

                            div {
                                class: "flex flex-row gap-2 place-content-between",
                                h4 {
                                    class: r#"
                                    flex-grow
                                    text-xl
                                    theme1:text-amber-300
                                    theme2:text-green-800
                                    theme3:font-semibold
                                    "#,

                                    "{area.name}"
                                }
                                div {
                                    if area.open() {
                                        LockOpenIcon {
                                            class: r#"
                                            size-4
                                            theme1:fill-amber-300
                                            theme2:fill-green-900
                                            "#,
                                        }
                                    } else {
                                        LockClosedIcon {
                                            class: r#"
                                            size-4
                                            theme1:fill-amber-300
                                            theme2:fill-green-900
                                            "#,
                                        }
                                    }
                                }
                            }

                            h5 {
                                class: r#"
                                theme1:text-amber-200

                                theme2:text-green-200
                                theme2:bg-green-800
                                theme2:px-2
                                theme2:rounded-sm

                                theme3:border-gold-rich
                                theme3:border-0
                                theme3:border-b-2
                                "#,

                                "Items"
                            }
                            if area.clone().items.is_empty() {
                                p {
                                    class: "p-2",
                                    "No items"
                                }
                            } else {
                                ul {
                                    class: "p-2",
                                    for item in area.clone().items {
                                        li {
                                            class: "flex flex-row gap-2 items-center pb-1",
                                            ItemIcon {
                                                item: item.clone(),
                                                css_class: r#"
                                                size-8
                                                theme1:fill-amber-500
                                                theme2:fill-green-800
                                                "#,
                                            }
                                            span {
                                                class: "text-sm",
                                                title: item.to_string(),
                                                "{item.to_string()}"
                                            }
                                        }
                                    }
                                }
                            }

                            h5 {
                                class: r#"
                                theme1:text-amber-200

                                theme2:text-green-200
                                theme2:bg-green-800
                                theme2:px-2
                                theme2:rounded-sm

                                theme3:border-gold-rich
                                theme3:border-0
                                theme3:border-b-2
                                "#,

                                "Events"
                            }
                            if area.clone().events.is_empty() {
                                p {
                                    class: "p-2",
                                    "No events"
                                }
                            } else {
                                ul {
                                    class: "p-2",
                                    for event in area.clone().events {
                                        li { "{event}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        QueryResult::Err(e) => {
            rsx! { p { "Something went wrong" } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! {} }
    }
}
