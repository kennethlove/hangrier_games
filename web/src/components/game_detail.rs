use crate::LoadingState;
use crate::cache::{MutationError, QueryError};
use crate::components::game_edit::GameEdit;
use crate::components::games_list::GamesListQ;
use crate::components::period_timeline::PeriodTimeline;
use crate::components::timeline::PeriodFilters;
use crate::components::ui::{Button, ButtonVariant};
use crate::hooks::use_timeline_summary::use_timeline_summary;
use crate::hooks::{ConnectionState, use_game_websocket};
use crate::http::WithCredentials;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use reqwest::StatusCode;
use shared::messages::GameMessage;
use shared::messages::MessagePayload;
use shared::{DisplayGame, GameStatus};

/// Returns true if `s` contains any non-whitespace character.
fn has_visible_content(s: &str) -> bool {
    !s.chars().all(char::is_whitespace)
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct DisplayGameQ;

impl QueryCapability for DisplayGameQ {
    type Ok = Box<DisplayGame>;
    type Err = QueryError;
    type Keys = String;

    async fn run(&self, identifier: &String) -> Result<Box<DisplayGame>, QueryError> {
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                crate::api_url::api_url(&format!("/api/games/{}/display", identifier)),
            )
            .with_credentials();
        match request.send().await {
            Ok(response) => match response.error_for_status() {
                Ok(response) => match response.json::<DisplayGame>().await {
                    Ok(game) => Ok(Box::new(game)),
                    Err(_) => Err(QueryError::BadJson),
                },
                Err(e) => {
                    if e.status() == Some(StatusCode::UNAUTHORIZED) {
                        Err(QueryError::Unauthorized)
                    } else {
                        Err(QueryError::GameNotFound(identifier.to_string()))
                    }
                }
            },
            Err(e) => {
                if e.status() == Some(StatusCode::UNAUTHORIZED) {
                    Err(QueryError::Unauthorized)
                } else {
                    Err(QueryError::GameNotFound(identifier.to_string()))
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct NextStepM;

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum NextStepResult {
    Started(String),
    Finished(String),
    Advanced(String),
}

impl NextStepResult {
    pub fn identifier(&self) -> &String {
        match self {
            NextStepResult::Started(s)
            | NextStepResult::Finished(s)
            | NextStepResult::Advanced(s) => s,
        }
    }
}

impl MutationCapability for NextStepM {
    type Ok = NextStepResult;
    type Err = MutationError;
    type Keys = String;

    async fn run(&self, args: &String) -> Result<NextStepResult, MutationError> {
        let identifier = args.clone();
        let client = reqwest::Client::new();
        let url: String = crate::api_url::api_url(&format!("/api/games/{}/next", identifier));
        let request = client.request(reqwest::Method::PUT, url).with_credentials();
        match request.send().await {
            Ok(response) => match response.status() {
                StatusCode::NO_CONTENT => Ok(NextStepResult::Finished(identifier)),
                StatusCode::CREATED => Ok(NextStepResult::Started(identifier)),
                StatusCode::OK => Ok(NextStepResult::Advanced(identifier)),
                _ => Err(MutationError::UnableToAdvanceGame),
            },
            Err(_) => Err(MutationError::UnableToAdvanceGame),
        }
    }

    async fn on_settled(&self, _keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            QueriesStorage::<DisplayGameQ>::invalidate_all().await;
            QueriesStorage::<GamesListQ>::invalidate_all().await;
        }
    }
}

#[component]
pub fn GamePage(identifier: String) -> Element {
    let (ws_events, ws_connection) = use_game_websocket(identifier.clone());

    let summary_q = use_timeline_summary(identifier.clone());
    let game_q = use_query(Query::new(identifier.clone(), DisplayGameQ));
    // Also ensure the GamesListQ storage exists in the root context so that
    // NextStepM::on_settled's QueriesStorage::<GamesListQ>::invalidate_all()
    // call doesn't panic when the user navigates straight to a game page
    // without ever visiting the home list (games would otherwise vanish from
    // the UI until a new game was created and the list rendered).
    let _games_list_q = use_query(Query::new((), crate::components::games_list::GamesListQ));

    let mut last_seen = use_signal(|| 0usize);
    use_effect(move || {
        let evs = ws_events.read();
        let len = evs.len();
        let start = (*last_seen.peek()).min(len);
        let bump_phase = evs[start..].iter().any(|msg| {
            matches!(
                msg.payload,
                MessagePayload::CycleStart { .. }
                    | MessagePayload::CycleEnd { .. }
                    | MessagePayload::GameEnded { .. }
            )
        });
        drop(evs);
        last_seen.set(len);
        if bump_phase {
            summary_q.invalidate();
            game_q.invalidate();
        }
    });

    rsx! {
        div {
            class: r#"
            mt-4
            flex
            flex-col
            gap-4
            "#,

            {match ws_connection() {
                ConnectionState::Connected => rsx! {
                    div { class: "text-sm text-primary", "Live updates connected" }
                },
                ConnectionState::Connecting => rsx! {
                    div { class: "text-sm text-gold", "Connecting to live updates..." }
                },
                ConnectionState::Disconnected => rsx! {
                    div { class: "text-sm text-muted", "Live updates disconnected" }
                },
                ConnectionState::Error(ref msg) => rsx! {
                    div { class: "text-sm text-danger", "Connection error: {msg}" }
                },
            }}

            PeriodTimeline { identifier: identifier.clone() }

            PeriodTimeline { identifier: identifier.clone() }

            GameState { identifier: identifier.clone() }
            GameStats { identifier: identifier.clone() }
            GameLog { identifier: identifier.clone(), ws_events }
        }
    }
}

#[cfg(test)]
mod tests {
    use shared::messages::{
        AreaEventKind, AreaRef, GameMessage, MessagePayload, MessageSource, Phase, TributeRef,
    };

    fn make(content: &str, payload: MessagePayload) -> GameMessage {
        GameMessage::new(
            MessageSource::Game("g".into()),
            1,
            Phase::Day,
            0,
            0,
            "subj".into(),
            content.into(),
            payload,
        )
    }

    #[test]
    fn cycle_start_payload_round_trips_through_message() {
        let msg = make(
            "Day 1 dawns over the arena.",
            MessagePayload::CycleStart {
                day: 1,
                phase: Phase::Day,
            },
        );
        assert_eq!(msg.content, "Day 1 dawns over the arena.");
        assert!(matches!(
            msg.payload,
            MessagePayload::CycleStart {
                day: 1,
                phase: Phase::Day
            }
        ));
    }

    #[test]
    fn arbitrary_payloads_keep_their_content() {
        let msg = make(
            "fire in north",
            MessagePayload::AreaEvent {
                area: AreaRef {
                    identifier: "a".into(),
                    name: "north".into(),
                },
                kind: AreaEventKind::Fire,
                description: "fire".into(),
            },
        );
        assert_eq!(msg.content, "fire in north");
    }

    #[test]
    fn tribute_killed_carries_victim() {
        let msg = make(
            "Rue died: spear",
            MessagePayload::TributeKilled {
                victim: TributeRef {
                    identifier: "t1".into(),
                    name: "Rue".into(),
                },
                killer: None,
                cause: "spear".into(),
            },
        );
        assert_eq!(msg.content, "Rue died: spear");
        assert!(msg.payload.involves("t1"));
    }
}

#[component]
fn GameState(identifier: String) -> Element {
    let mut loading_signal = use_context::<Signal<LoadingState>>();
    let mut filters = use_context::<Signal<PeriodFilters>>();

    let game_query = use_query(Query::new(identifier.clone(), DisplayGameQ));

    let mutate = use_mutation(Mutation::new(NextStepM));

    let reader = game_query.read();
    let state = reader.state();

    match &*state {
        QueryStateData::Settled {
            res: Ok(game_data), ..
        } => {
            let game = (**game_data).clone();
            let game_id = game.identifier.clone();
            let g_id = game_id.clone();
            let game_name = game.name.clone();
            let game_status = game.status.clone();
            let is_mine = game.is_mine;
            let is_ready = game.ready;
            let is_finished = game.status == GameStatus::Finished;
            let game_private = game.private;
            let creator = game.created_by.username.clone();
            let day = game.day.unwrap_or(0);

            let game_next_step = match game_status {
                GameStatus::NotStarted => if is_ready { "Start" } else { "Wait" }.to_string(),
                GameStatus::InProgress => format!("Play day {}", day + 1),
                GameStatus::Finished => "Done!".to_string(),
            };

            let next_step_handler = move |_| {
                let game_id_clone = game_id.clone();
                spawn(async move {
                    loading_signal.set(LoadingState::Loading);
                    let reader = mutate.mutate_async(game_id_clone).await;
                    let state = reader.state();
                    match &*state {
                        MutationStateData::Settled {
                            res: Ok(result), ..
                        } => {
                            filters.write().bump(result.identifier());
                            loading_signal.set(LoadingState::Loaded);
                        }
                        _ => {
                            loading_signal.set(LoadingState::Loaded);
                        }
                    }
                });
            };

            rsx! {
                div {
                    class: "flex flex-col gap-2",
                    div {
                        class: r#"
                        flex
                        flex-row
                        gap-4
                        place-content-between
                        align-middle
                        "#,

                        h2 {
                            class: r#"

                            "#,

                            "{game_name}"
                        }

                        if is_mine {
                            GameEdit {
                                identifier: g_id.clone(),
                                name: game_name.clone(),
                                private: game_private,
                                icon_class: r#"
                                size-4

                                "#
                            }
                        } else {
                            span {
                                class: r#"
                                text-sm

                                "#,
                                "By {creator}"
                            }
                        }
                    }
                    if is_mine && !is_finished {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: next_step_handler,
                            disabled: is_finished,
                            "{game_next_step}"
                        }
                    }
                }
            }
        }
        QueryStateData::Settled {
            res: Err(QueryError::GameNotFound(_)),
            ..
        } => {
            rsx! {
                p {
                    class: r#"
                    text-center

                    "#,
                    "Game not found"
                }
            }
        }
        QueryStateData::Settled {
            res: Err(QueryError::Unauthorized),
            ..
        } => {
            rsx! {
                p {
                    class: r#"
                    text-center

                    "#,

                    h2 {
                        class: r#"
                        text-2xl

                        "#,
                        "Unauthorized"
                    }
                    p {
                        "Do you need to "
                        Link {
                            class: r#"
                            underline

                            "#,
                            to: Routes::AccountsPage {},
                            "login or signup?"
                        }
                    }
                }
            }
        }
        QueryStateData::Settled { res: Err(e), .. } => {
            rsx! {
                p {
                    class: r#"
                    text-center

                    "#,
                    "Failed to load: {e:?}"
                }
            }
        }
        QueryStateData::Loading { .. } => {
            rsx! {
                p {
                    class: r#"
                    text-center

                    "#,
                    "Loading..."
                }
            }
        }
        _ => {
            rsx! {}
        }
    }
}

#[component]
fn GameStats(identifier: String) -> Element {
    let game_query = use_query(Query::new(identifier.clone(), DisplayGameQ));
    let reader = game_query.read();
    let state = reader.state();

    match &*state {
        QueryStateData::Settled { res: Ok(game), .. } => {
            let game_day = game.day.unwrap_or(0);
            let tribute_count = game.living_count;

            let game_status = match game.status {
                GameStatus::NotStarted => "Not started".to_string(),
                GameStatus::InProgress => "In progress".to_string(),
                GameStatus::Finished => "Finished".to_string(),
            };

            rsx! {
                div {
                    class: "flex flex-col gap-2 mt-4",
                    div {
                        class: "flex flex-row place-content-between pr-2",

                        p {
                            class: r#"
                            flex-grow

                            "#,

                            span {
                                class: r#"
                                block
                                text-sm

                                "#,

                                "status"
                            }
                            "{game_status}"
                        }
                        p {
                            class: r#"
                            flex-grow

                            "#,

                            span {
                                class: r#"
                                block
                                text-sm

                                "#,

                                "day"
                            }
                            "{game_day}"
                        }
                        p {
                            class: r#"

                            "#,

                            span {
                                class: r#"
                                block
                                text-sm

                                "#,

                                "tributes alive"
                            }
                            "{tribute_count}"
                        }
                    }
                }
            }
        }
        QueryStateData::Settled { res: Err(_), .. } => {
            rsx! {}
        }
        QueryStateData::Loading { .. } => {
            rsx! {
                p {
                    class: r#"
                    text-center

                    "#,
                    "Loading..."
                }
            }
        }
        _ => {
            rsx! {}
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameLogQ;

impl QueryCapability for GameLogQ {
    type Ok = Vec<GameMessage>;
    type Err = QueryError;
    type Keys = String;

    async fn run(&self, identifier: &String) -> Result<Vec<GameMessage>, QueryError> {
        let url = crate::api_url::api_url(&format!("/api/games/{identifier}/log"));
        let resp = reqwest::Client::new()
            .get(&url)
            .with_credentials()
            .send()
            .await;
        match resp {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Vec<GameMessage>>().await {
                    Ok(v) => Ok(v),
                    Err(_) => Err(QueryError::BadJson),
                },
                StatusCode::UNAUTHORIZED => Err(QueryError::Unauthorized),
                StatusCode::NOT_FOUND => Err(QueryError::GameNotFound(identifier.clone())),
                _ => Err(QueryError::Unknown),
            },
            Err(_) => Err(QueryError::ServerNotFound),
        }
    }
}

#[component]
fn GameLog(identifier: String, ws_events: Signal<Vec<GameMessage>>) -> Element {
    let log_q = use_query(Query::new(identifier.clone(), GameLogQ));

    let messages = use_memo(move || {
        let historical = {
            let reader = log_q.read();
            let state = reader.state();
            match &*state {
                QueryStateData::Settled { res: Ok(msgs), .. } => msgs.clone(),
                _ => vec![],
            }
        };
        let live = ws_events.read().clone();

        let mut seen: std::collections::HashMap<String, GameMessage> =
            std::collections::HashMap::new();
        for msg in historical {
            seen.insert(msg.identifier.clone(), msg);
        }
        for msg in live {
            seen.insert(msg.identifier.clone(), msg);
        }

        let mut merged_msgs: Vec<GameMessage> = seen.into_values().collect();
        merged_msgs.sort_by(|a, b| {
            a.game_day
                .cmp(&b.game_day)
                .then(a.phase.cmp(&b.phase))
                .then(a.tick.cmp(&b.tick))
                .then(a.emit_index.cmp(&b.emit_index))
        });

        merged_msgs
    });

    rsx! {
        div {
            class: "bg-surface p-4 rounded-lg",
            h3 { class: "font-bold mb-2", "Game Log" }
            div {
                class: "max-h-96 overflow-y-auto space-y-1",
                if messages.is_empty() {
                    p { class: "text-sm text-muted", "No events yet" }
                } else {
                    for msg in messages.iter() {
                        if has_visible_content(&msg.content) {
                            div { class: "text-sm py-1 border-b border-border",
                                "{msg.content}"
                            }
                        }
                    }
                }
            }
        }
    }
}
