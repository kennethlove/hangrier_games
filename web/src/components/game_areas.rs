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

async fn fetch_areas(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Areas(identifier)) = keys.first() {
        let response = reqwest::get(format!("{}/api/games/{}/areas", API_HOST, identifier))
            .await
            .unwrap();

        match response.json::<Vec<AreaDetails>>().await {
            Ok(areas) => {
                QueryResult::Ok(QueryValue::Areas(areas))
            }
            Err(_) => QueryResult::Err(QueryError::GameNotFound(identifier.to_string())),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

#[component]
pub fn GameAreaList() -> Element {
    let game_signal: Signal<Option<Game>> = use_context();

    let game = game_signal.read().clone();
    let game = game.unwrap();
    let identifier = game.identifier.clone();

    let area_query = use_get_query(
        [
            QueryKey::Areas(identifier.clone()),
            QueryKey::Game(identifier.clone()),
            QueryKey::Games
        ],
        fetch_areas,
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
                                            class: "flex flex-row gap-2 items-center",
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
            dioxus_logger::tracing::error!("{:?}", e);
            rsx! { p { "Something went wrong" } }
        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => { rsx! {} }
    }
}
