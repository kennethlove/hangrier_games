use crate::cache::QueryError;
use crate::components::icons::loading::LoadingIcon;
use crate::components::icons::map_pin::MapPinIcon;
use crate::components::item_icon::ItemIcon;
use crate::components::tribute_edit::TributeEdit;
use crate::components::tribute_status_icon::TributeStatusIcon;
use crate::env::APP_API_HOST;
use crate::routes::Routes;
use crate::storage::{AppState, use_persistent};
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
pub(crate) struct GameTributesQ {
    pub token: String,
}

impl QueryCapability for GameTributesQ {
    type Ok = PaginatedTributesResponse;
    type Err = QueryError;
    type Keys = String;

    async fn run(&self, game_identifier: &String) -> Result<PaginatedTributesResponse, QueryError> {
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                format!(
                    "{}/api/games/{}/tributes?limit=24&offset=0",
                    APP_API_HOST, game_identifier
                ),
            )
            .bearer_auth(&self.token);
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
    let storage = use_persistent("hangry-games", AppState::default);
    let token = storage.get().jwt.expect("No JWT found");

    let identifier = game.identifier.clone();

    let tribute_query = use_query(Query::new(identifier.clone(), GameTributesQ { token }));
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
                            theme1:not-last-of-type:border-b-2
                            theme1:border-amber-900
                            theme2:not-last-of-type:border-b-1
                            theme2:border-dotted
                            theme2:border-green-200
                            theme3:not-last-of-type:border-b-2
                            theme3:border-gold-rich
                            "#,
                            h3 {
                                class: r#"
                                text-xl
                                text-center
                                mb-2
                                theme1:text-stone-200
                                theme1:font-[Cinzel]
                                theme2:text-green-200
                                theme2:font-[Playfair_Display]
                                theme3:text-yellow-700
                                theme3:drop-shadow-sm
                                theme3:font-[Orbitron]
                                "#,
                                "District {chunk.first().unwrap().district}"
                            }
                            ul {
                                GameTributeListMember {
                                    tribute: chunk.first().unwrap().clone(),
                                    game_identifier: identifier.clone(),
                                    game_status: game.status.clone(),
                                }
                                GameTributeListMember {
                                    tribute: chunk.last().unwrap().clone(),
                                    game_identifier: identifier.clone(),
                                    game_status: game.status.clone(),
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
) -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let _token = storage.get().jwt.expect("No JWT found");

    let fist_item = Item::new_weapon("basic fist");

    rsx! {
        li {
            "data-alive": tribute.is_alive(),
            class: r#"
            border
            p-2
            self-start
            overflow-hidden

            theme1:border-1
            theme1:text-stone-200
            theme1:data-[alive=false]:border-red-500/50
            theme1:motion-safe:data-[alive=true]:border-tracer
            theme1:motion-safe:data-[alive=true]:border-2
            theme1:motion-reduce:data-[alive=true]:border-green-500

            theme2:data-[alive=true]:border-green-400
            theme2:data-[alive=false]:border-red-400
            theme2:data-[alive=false]:bg-linear-to-r
            theme2:data-[alive=false]:from-green-900
            theme2:data-[alive=false]:to-red-900/75
            theme2:text-green-200
            theme2:rounded-md

            theme3:border-3
            theme3:data-[alive=true]:border-gold-rich
            theme3:data-[alive=false]:opacity-50
            theme3:bg-stone-50
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

                        theme1:font-[Cinzel]
                        theme1:text-lg

                        theme2:font-[Playfair_Display]
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

                            theme2:text-green-200
                            theme2:hover:text-green-200

                            theme3:hover:border-b-2
                            theme3:hover:border-yellow-500
                            theme3:hover:text-yellow-500
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
                            theme1:fill-amber-500
                            theme2:fill-green-200
                            mb-2
                            "#,
                        }
                        span {
                            class: "",
                            "{tribute.status.to_string()}"
                        }
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
                            theme1:fill-amber-500
                            theme2:fill-green-200
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
                                    theme1:fill-amber-500
                                    theme2:fill-green-200
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
                                        theme1:fill-amber-500
                                        theme2:fill-green-200
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
