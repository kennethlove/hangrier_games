use crate::cache::QueryError;
use crate::components::icons::lock_closed::LockClosedIcon;
use crate::components::icons::lock_open::LockOpenIcon;
use crate::components::item_icon::ItemIcon;
use crate::components::map::Map;
use crate::env::APP_API_HOST;
use crate::http::WithCredentials;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::areas::AreaDetails;
use shared::DisplayGame;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameAreasQ;

impl QueryCapability for GameAreasQ {
    type Ok = Vec<AreaDetails>;
    type Err = QueryError;
    type Keys = String;

    async fn run(&self, identifier: &String) -> Result<Vec<AreaDetails>, QueryError> {
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                format!("{}/api/games/{}/areas", APP_API_HOST, identifier),
            )
            .with_credentials();
        match request.send().await {
            Ok(response) => match response.json::<Vec<AreaDetails>>().await {
                Ok(areas) => Ok(areas),
                Err(_) => Err(QueryError::GameNotFound(identifier.to_string())),
            },
            Err(_) => Err(QueryError::GameNotFound(identifier.to_string())),
        }
    }
}

#[component]
pub fn GameAreaList(game: DisplayGame) -> Element {
    let identifier = game.identifier.clone();

    let area_query = use_query(Query::new(identifier.clone(), GameAreasQ));
    let reader = area_query.read();
    let state = reader.state();

    match &*state {
        QueryStateData::Settled { res: Ok(areas), .. } => {
            rsx! {
                ul {
                    class: "grid grid-cols-2 gap-4",
                    li {
                        Map { areas: areas.clone() }
                    }
                    for area in areas {
                        li {
                            "data-open": area.is_open(),
                            class: r#"
                            border
                            p-2

                            "#,

                            div {
                                class: "flex flex-row gap-2 place-content-between",
                                h4 {
                                    class: r#"
                                    flex-grow
                                    text-xl

                                    "#,

                                    "{area.name}"
                                    // TODO: Display terrain information alongside area name
                                    // Example: "{area.name} ({area.terrain.base_terrain})"
                                    // Requires terrain data to be properly populated from backend
                                }
                                div {
                                    if area.is_open() {
                                        LockOpenIcon {
                                            class: r#"
                                            size-4

                                            "#,
                                        }
                                    } else {
                                        LockClosedIcon {
                                            class: r#"
                                            size-4

                                            "#,
                                        }
                                    }
                                }
                            }

                            h5 {
                                class: r#"

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
        QueryStateData::Settled { res: Err(_), .. } => {
            rsx! { p { "Something went wrong" } }
        }
        QueryStateData::Loading { .. } => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx! {}
        }
    }
}
