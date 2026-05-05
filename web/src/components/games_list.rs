use crate::cache::QueryError;
use crate::components::game_edit::GameEdit;
use crate::components::icons::eye_closed::EyeClosedIcon;
use crate::components::icons::eye_open::EyeOpenIcon;
use crate::components::{Button, CreateGameButton, CreateGameForm, DeleteGameModal, GameDelete};
use crate::env::APP_API_HOST;
use crate::http::WithCredentials;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use serde::{Deserialize, Serialize};
use shared::{GameStatus, ListDisplayGame, PaginationMetadata};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PaginatedGamesResponse {
    pub games: Vec<ListDisplayGame>,
    pub pagination: PaginationMetadata,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct GamesListQ;

impl QueryCapability for GamesListQ {
    type Ok = PaginatedGamesResponse;
    type Err = QueryError;
    type Keys = ();

    async fn run(&self, _keys: &()) -> Result<PaginatedGamesResponse, QueryError> {
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                format!("{}/api/games?limit=20&offset=0", APP_API_HOST),
            )
            .with_credentials();

        match request.send().await {
            Ok(response) => match response.json::<PaginatedGamesResponse>().await {
                Ok(r) => Ok(r),
                Err(_) => Err(QueryError::BadJson),
            },
            Err(_) => Err(QueryError::NoGames),
        }
    }
}

#[component]
fn NoGames() -> Element {
    rsx! {
        p {
            class: "pb-4 text-center text-text-muted",
            "No games yet"
        }
    }
}

#[component]
pub fn GamesList() -> Element {
    let games_query = use_query(Query::new((), GamesListQ));

    let reader = games_query.read();
    let body = match &*reader.state() {
        QueryStateData::Settled {
            res: Ok(response), ..
        }
        | QueryStateData::Loading {
            res: Some(Ok(response)),
        } => {
            let games = response.games.clone();
            let pagination = response.pagination.clone();
            rsx! {
                if games.is_empty() {
                    NoGames {}
                } else {
                    ul {
                        class: "xl:grid xl:grid-cols-2 xl:gap-4",
                        for game in games {
                            GameListMember { game: game.clone() }
                        }
                    }

                    if pagination.has_more {
                        div {
                            class: "flex justify-center gap-2 mt-4",
                            LoadMoreButton {
                                current_offset: pagination.offset,
                                limit: pagination.limit,
                            }
                        }
                    }
                }
            }
        }
        QueryStateData::Loading { .. } | QueryStateData::Pending => rsx! { p {
            class: "pb-4 text-center text-text-muted",
            "Loading..."
        } },
        QueryStateData::Settled {
            res: Err(QueryError::NoGames),
            ..
        } => rsx! { NoGames {} },
        QueryStateData::Settled {
            res: Err(QueryError::BadJson),
            ..
        } => rsx! { p {
            class: "pb-4 text-center text-text-muted",
            "Bad JSON response"
        } },
        QueryStateData::Settled { res: Err(_), .. } => rsx! { p { "Something went wrong" } },
    };

    rsx! {
        div {
            class: "flex flex-col flex-col-reverse sm:flex-row flex-wrap sm:flex-nowrap gap-2 place-content-center py-2 mb-4",
            CreateGameButton {}
            CreateGameForm {}
        }

        {body}

        div {
            class: "mt-4",
            RefreshButton {}
        }

        DeleteGameModal {}
    }
}

#[component]
fn RefreshButton() -> Element {
    let onclick = move |_| {
        spawn(async move {
            QueriesStorage::<GamesListQ>::invalidate_all().await;
        });
    };

    rsx! {
        div {
            class: "text-center",
            Button {
                onclick,
                "Refresh"
            }
        }
    }
}

#[component]
pub fn GameListMember(game: ListDisplayGame) -> Element {
    let created_by = game.clone().created_by;
    let is_mine = game.clone().is_mine;

    let living_count = game.living_count;

    rsx! {
        li {
            class: "block w-full border border-border bg-surface rounded-card p-4 mb-4 xl:mb-0 transition hover:bg-surface-2",
            div {
                class: "flex place-content-between",
                h2 {
                    class: "font-display text-2xl tracking-wide text-text hover:text-primary",
                    Link {
                        to: Routes::GamePage {
                            identifier: game.identifier.clone()
                        },
                        title: r#"Play "{game.name}""#,
                        "{game.name}"
                    }
                }
                if is_mine {
                    div {
                        class: "flex flex-row gap-2",
                        GameEdit {
                            identifier: game.identifier.clone(),
                            name: game.name.clone(),
                            private: game.private,
                            icon_class: "size-4 fill-text-muted hover:fill-primary",
                        }
                        GameDelete {
                            game_name: game.name.clone(),
                            game_identifier: game.identifier.clone(),
                            icon_class: "size-4 fill-text-muted hover:fill-danger",
                        }
                    }
                } else {
                    div {
                        class: "flex flex-row gap-2",
                        p {
                            class: "text-sm text-text-muted",
                            "By {created_by.username}"
                        }
                    }
                }
            }
            div {
                class: "flex flex-row place-content-between text-xs font-mono text-text-muted mt-2",
                p { class: "flex-grow", "{living_count} / {game.tribute_count} tributes" }
                p { class: "flex-grow", "Day {game.day.unwrap_or_default()}" }
                p {
                    class: "flex-grow",
                    match game.status {
                        GameStatus::InProgress => "In progress",
                        GameStatus::Finished => "Finished",
                        GameStatus::NotStarted => "Not started",
                    }
                }
                div {
                    class: "px-2",
                    if game.private {
                        EyeClosedIcon { class: "size-4 fill-text-muted" }
                    } else {
                        EyeOpenIcon { class: "size-4 fill-text-muted" }
                    }
                }
            }
        }
    }
}

#[component]
fn LoadMoreButton(current_offset: u32, limit: u32) -> Element {
    let _next_offset = current_offset + limit;

    let onclick = move |_| {
        spawn(async move {
            QueriesStorage::<GamesListQ>::invalidate_all().await;
        });
    };

    rsx! {
        Button {
            onclick,
            "Load More Games"
        }
    }
}
