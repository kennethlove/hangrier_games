use crate::cache::QueryError;
use crate::components::icons::lock_closed::LockClosedIcon;
use crate::components::icons::lock_open::LockOpenIcon;
use crate::components::icons::uturn::UTurnIcon;
use crate::components::item_icon::ItemIcon;
use crate::http::WithCredentials;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::areas::AreaDetails;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct AreaDetailQ;

impl QueryCapability for AreaDetailQ {
    type Ok = Box<AreaDetails>;
    type Err = QueryError;
    type Keys = (String, String);

    async fn run(&self, keys: &(String, String)) -> Result<Box<AreaDetails>, QueryError> {
        let (game_identifier, area_identifier) = keys;
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                crate::api_url::api_url(&format!(
                    "/api/games/{}/areas/{}",
                    game_identifier, area_identifier
                )),
            )
            .with_credentials();
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AreaDetails>().await {
                        Ok(area) => Ok(Box::new(area)),
                        Err(_) => Err(QueryError::BadJson),
                    }
                } else {
                    Err(QueryError::Unknown)
                }
            }
            Err(_) => Err(QueryError::Unknown),
        }
    }
}

#[component]
pub fn AreaDetail(game_identifier: String, area_identifier: String) -> Element {
    let q = use_query(Query::new(
        (game_identifier.clone(), area_identifier.clone()),
        AreaDetailQ,
    ));
    let reader = q.read();
    let body = match &*reader.state() {
        QueryStateData::Settled { res: Ok(area), .. } => {
            let area = area.as_ref();
            rsx! {
                div {
                    class: "p-4 space-y-4",
                    div {
                        class: "flex flex-row gap-2 items-center",
                        Link {
                            to: Routes::GamePage { identifier: game_identifier.clone() },
                            class: "inline-flex items-center gap-1 text-sm underline",
                            UTurnIcon { class: "size-4 fill-current" }
                            "Back to game"
                        }
                    }
                    div {
                        class: "flex flex-row gap-2 items-center",
                        h1 {
                            class: "text-2xl font-semibold flex-grow",
                            "{area.name}"
                        }
                        if area.is_open() {
                            LockOpenIcon {
                                class: "size-5   "
                            }
                        } else {
                            LockClosedIcon {
                                class: "size-5   "
                            }
                        }
                    }
                    p {
                        class: "text-sm opacity-75",
                        "Terrain: {area.terrain.base:?}"
                    }
                    section {
                        h2 { class: "text-lg font-semibold mb-2", "Items" }
                        if area.items.is_empty() {
                            p { class: "opacity-75", "No items in this area." }
                        } else {
                            ul {
                                class: "space-y-1",
                                for item in area.items.clone() {
                                    li {
                                        class: "flex flex-row gap-2 items-center",
                                        ItemIcon {
                                            item: item.clone(),
                                            css_class: "size-6  ",
                                        }
                                        Link {
                                            to: Routes::ItemDetail {
                                                game_identifier: game_identifier.clone(),
                                                item_identifier: item.identifier.clone(),
                                            },
                                            class: "underline",
                                            "{item.to_string()}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    section {
                        h2 { class: "text-lg font-semibold mb-2", "Events" }
                        if area.events.is_empty() {
                            p { class: "opacity-75", "No events recorded." }
                        } else {
                            ul {
                                class: "space-y-1 list-disc list-inside",
                                for event in area.events.clone() {
                                    li { "{event}" }
                                }
                            }
                        }
                    }
                }
            }
        }
        QueryStateData::Settled { res: Err(_), .. } => {
            rsx! { p { class: "p-4", "Area not found." } }
        }
        QueryStateData::Loading { .. } | QueryStateData::Pending => {
            rsx! { p { class: "p-4", "Loading..." } }
        }
    };
    rsx! { {body} }
}
