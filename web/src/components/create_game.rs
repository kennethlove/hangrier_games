use crate::LoadingState;
use crate::cache::MutationError;
use crate::components::games_list::GamesListQ;
use crate::components::{Input, ThemedButton};
use crate::env::APP_API_HOST;
use crate::storage::{AppState, use_persistent};
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::Game;
use shared::CreateGame;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct CreateGameM;

impl MutationCapability for CreateGameM {
    type Ok = Game;
    type Err = MutationError;
    type Keys = (Option<String>, String);

    async fn run(&self, args: &Self::Keys) -> Result<Game, MutationError> {
        let name = args.0.clone();
        let token = args.1.clone();
        let client = reqwest::Client::new();
        let json_body = CreateGame {
            name,
            item_quantity: Default::default(),
            event_frequency: Default::default(),
            starting_health_range: None,
        };

        let response = client
            .request(reqwest::Method::POST, format!("{}/api/games", APP_API_HOST))
            .bearer_auth(token)
            .json(&json_body);

        match response.send().await {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    let body = response.text().await.unwrap_or_default();
                    tracing::error!("create_game failed: status={} body={}", status, body);
                    return Err(MutationError::UnableToCreateGame);
                }
                match response.json::<Game>().await {
                    Ok(game) => Ok(game),
                    Err(e) => {
                        tracing::error!("create_game parse error: {:?}", e);
                        Err(MutationError::UnableToCreateGame)
                    }
                }
            }
            Err(e) => {
                tracing::error!("error creating game: {:?}", e);
                Err(MutationError::UnableToCreateGame)
            }
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Game, MutationError>) {
        if result.is_ok() {
            QueriesStorage::<GamesListQ>::invalidate_all().await;
        }
    }
}

#[component]
pub fn CreateGameButton() -> Element {
    let storage = use_persistent("hangry-games", AppState::default);
    let mutate = use_mutation(Mutation::new(CreateGameM));
    let mut loading_signal = use_context::<Signal<LoadingState>>();

    let onclick = move |_| {
        loading_signal.set(LoadingState::Loading);
        let token = storage.get().jwt.unwrap_or_default();
        spawn(async move {
            let _ = mutate.mutate_async((None, token)).await;
            loading_signal.set(LoadingState::Loaded);
        });
    };

    rsx! {
        ThemedButton {
            onclick,
            "Quickstart"
        }
    }
}

#[component]
pub fn CreateGameForm() -> Element {
    let storage = use_persistent("hangry-games", AppState::default);

    let mut game_name_signal: Signal<String> = use_signal(String::default);
    let mutate = use_mutation(Mutation::new(CreateGameM));
    let mut loading_signal = use_context::<Signal<LoadingState>>();

    let onsubmit = move |_| {
        let token = storage.get().jwt.unwrap_or_default();
        let name = game_name_signal.peek().clone();
        if name.is_empty() {
            return;
        }
        loading_signal.set(LoadingState::Loading);

        spawn(async move {
            let reader = mutate.mutate_async((Some(name), token)).await;
            if reader.state().is_ok() {
                game_name_signal.set(String::default());
            }
            loading_signal.set(LoadingState::Loaded);
        });
    };

    rsx! {
        form {
            class: "flex flex-row justify-center gap-2",
            onsubmit,
            label {
                r#for: "game-name",
                class: "sr-only",
                "Game name"
            }
            Input {
                id: "game-name",
                name: "game-name",
                r#type: "text",
                placeholder: "Game name",
                value: game_name_signal.read().clone(),
                oninput: move |e: Event<FormData>| {
                    game_name_signal.set(e.value().clone());
                }
            }
            ThemedButton {
                r#type: "submit",
                "Create game"
            }
        }
    }
}
