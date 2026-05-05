use crate::cache::QueryError;
use crate::components::icons::loading::LoadingIcon;
use crate::components::icons::map_pin::MapPinIcon;
use crate::components::item_icon::ItemIcon;
use crate::components::tribute_edit::TributeEdit;
use crate::components::tribute_state_strip::TributeStateStrip;
use crate::components::tribute_status_icon::TributeStatusIcon;
use crate::http::WithCredentials;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::items::Item;
use game::tributes::Tribute;
use serde::{Deserialize, Serialize};
use shared::{DisplayGame, GameStatus, PaginationMetadata};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PaginatedTributesResponse {
    pub tributes: Vec<Tribute>,
    pub pagination: PaginationMetadata,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameTributesQ;

impl QueryCapability for GameTributesQ {
    type Ok = PaginatedTributesResponse;
    type Err = QueryError;
    type Keys = String;

    async fn run(&self, game_identifier: &String) -> Result<PaginatedTributesResponse, QueryError> {
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                crate::api_url::api_url(&format!(
                    "/api/games/{}/tributes?limit=24&offset=0",
                    game_identifier
                )),
            )
            .with_credentials();
        match request.send().await {
            Ok(response) => match response.json::<PaginatedTributesResponse>().await {
                Ok(tributes) => Ok(tributes),
                Err(_) => Err(QueryError::BadJson),
            },
            Err(_) => Err(QueryError::GameNotFound(game_identifier.to_string())),
        }
    }
}

#[component]
pub fn GameTributes(game: DisplayGame) -> Element {
    let identifier = game.identifier.clone();

    let tribute_query = use_query(Query::new(identifier.clone(), GameTributesQ));
    let reader = tribute_query.read();
    let state = reader.state();

    match &*state {
        QueryStateData::Settled {
            res: Ok(response), ..
        } => {
            let tributes = response.tributes.clone();
            rsx! {
                ul {
                    class: "grid gap-2 grid-cols-2",
                    for chunk in tributes.as_slice().chunks(2) {
                        li {
                            class: r#"
                            col-span-2
                            pb-4

                            "#,
                            h3 {
                                class: r#"
                                text-xl
                                text-center
                                mb-2

                                "#,
                                "District {chunk.first().unwrap().district}"
                            }
                            ul {
                                GameTributeListMember {
                                    tribute: chunk.first().unwrap().clone(),
                                    game_identifier: identifier.clone(),
                                    game_status: game.status.clone(),
                                    current_phase: game.day.map(|d| d * 2),
                                }
                                GameTributeListMember {
                                    tribute: chunk.last().unwrap().clone(),
                                    game_identifier: identifier.clone(),
                                    game_status: game.status.clone(),
                                    current_phase: game.day.map(|d| d * 2),
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
            rsx! {
                div {
                    class: "flex justify-center",
                    LoadingIcon {}
                }
            }
        }
        _ => {
            rsx! {}
        }
    }
}

#[component]
pub fn GameTributeListMember(
    tribute: Tribute,
    game_identifier: String,
    game_status: GameStatus,
    current_phase: Option<u32>,
) -> Element {
    let fist_item = Item::new_weapon("basic fist");

    rsx! {
        li {
            "data-alive": tribute.is_alive(),
            class: r#"
            border
            p-2
            self-start
            overflow-hidden

            "#,

            div {
                class: "flex flex-row gap-2",
                img {
                    class: "w-full h-full border-2 border-amber-500 min-w-16 max-w-24",
                    src: "{tribute.avatar()}",
                }
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

                        "#,

                        Link {
                            class: r#"

                            "#,
                            to: Routes::TributeDetail {
                                game_identifier: game_identifier.clone(),
                                tribute_identifier: tribute.identifier.clone()
                            },
                            "{tribute.name}"
                        }
                    }
                    div {
                        if game_status == GameStatus::NotStarted {
                            div {
                                TributeEdit {
                                    identifier: tribute.identifier.clone(),
                                    name: tribute.name.clone(),
                                    avatar: tribute.avatar(),
                                    game_identifier: game_identifier.clone(),
                                }
                            }
                        }
                    }
                }

                div {
                    div {
                        class: "flex flex-row gap-2 flex-wrap place-items-center text-sm",
                        TributeStatusIcon {
                            status: tribute.status.clone(),
                            css_class: r#"
                            size-6
                            ml-1

                            mb-2
                            "#,
                        }
                        span {
                            class: "",
                            "{tribute.status.to_string()}"
                        }
                    }
                    TributeStateStrip {
                        tribute: tribute.clone(),
                        current_phase: current_phase,
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
    }
}
