use crate::LoadingState;
use crate::cache::MutationError;
use crate::components::games_list::GamesListQ;
use crate::components::{Input, ThemedButton};
use crate::http::WithCredentials;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::Game;
use shared::CreateGame;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct CreateGameM;

impl MutationCapability for CreateGameM {
    type Ok = Game;
    type Err = MutationError;
    type Keys = Option<String>;

    async fn run(&self, args: &Self::Keys) -> Result<Game, MutationError> {
        let name = args.clone();
        let client = reqwest::Client::new();
        let json_body = CreateGame {
            name,
            item_quantity: Default::default(),
            event_frequency: Default::default(),
            starting_health_range: None,
        };

        let response = client
            .request(reqwest::Method::POST, crate::api_url::api_url("/api/games"))
            .with_credentials()
            .json(&json_body);

        match response.send().await {
            Ok(response) => {
                let status = response.status();
                if status == reqwest::StatusCode::UNAUTHORIZED {
                    tracing::warn!("create_game unauthorized");
                    return Err(MutationError::Unauthorized);
                }
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
    let mutate = use_mutation(Mutation::new(CreateGameM));
    let mut loading_signal = use_context::<Signal<LoadingState>>();
    let mut error_signal = use_signal(String::default);
    let navigator = use_navigator();

    let onclick = move |_| {
        loading_signal.set(LoadingState::Loading);
        error_signal.set(String::default());
        spawn(async move {
            let reader = mutate.mutate_async(None).await;
            match &*reader.state() {
                MutationStateData::Settled {
                    res: Err(MutationError::Unauthorized),
                    ..
                } => {
                    navigator.replace(Routes::AccountsPage {});
                }
                MutationStateData::Settled { res: Err(_), .. } => {
                    error_signal
                        .set("Could not create game. Try again or check the server.".to_string());
                }
                _ => {}
            }
            loading_signal.set(LoadingState::Loaded);
        });
    };

    rsx! {
        div {
            class: "flex flex-col gap-1",
            ThemedButton {
                onclick,
                "Quickstart"
            }
            if !error_signal.read().is_empty() {
                p {
                    class: "text-sm text-red-500",
                    role: "alert",
                    "{error_signal.read()}"
                }
            }
        }
    }
}

#[component]
pub fn CreateGameForm() -> Element {
    let mut game_name_signal: Signal<String> = use_signal(String::default);
    let mutate = use_mutation(Mutation::new(CreateGameM));
    let mut loading_signal = use_context::<Signal<LoadingState>>();
    let mut error_signal = use_signal(String::default);
    let navigator = use_navigator();

    let onsubmit = move |_| {
        let name = game_name_signal.peek().clone();
        if name.is_empty() {
            return;
        }
        loading_signal.set(LoadingState::Loading);
        error_signal.set(String::default());

        spawn(async move {
            let reader = mutate.mutate_async(Some(name)).await;
            match &*reader.state() {
                MutationStateData::Settled { res: Ok(_), .. } => {
                    game_name_signal.set(String::default());
                }
                MutationStateData::Settled {
                    res: Err(MutationError::Unauthorized),
                    ..
                } => {
                    navigator.replace(Routes::AccountsPage {});
                }
                MutationStateData::Settled { res: Err(_), .. } => {
                    error_signal.set("Could not create game.".to_string());
                }
                _ => {}
            }
            loading_signal.set(LoadingState::Loaded);
        });
    };

    rsx! {
        div {
            class: "flex flex-col gap-1",
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
            if !error_signal.read().is_empty() {
                p {
                    class: "text-sm text-red-500 text-center",
                    role: "alert",
                    "{error_signal.read()}"
                }
            }
        }
    }
}
