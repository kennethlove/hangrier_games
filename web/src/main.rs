use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::games::{Game, GameStatus};
use num_traits::ToPrimitive;
use std::str::FromStr;
use serde::Deserialize;
use std::env;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[derive(Clone, PartialEq, Eq, Hash)]
enum QueryKey {
    AllGames,
    Game(usize),
    Games,
}

#[derive(PartialEq, Debug)]
enum QueryError {
    GameNotFound(usize),
    NoGames,
    Unknown
}

#[derive(PartialEq, Debug)]
enum QueryValue {
    Games(Vec<Game>),
    GameName(String),
}

async fn fetch_games(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    if let Some(QueryKey::Games) = keys.first() {
        let db: Resource<Surreal<Client>> = use_context();
        let db = db.value().unwrap();
        db.use_ns("hangry-games").use_db("games").await.expect("Failed to use games database");
        dioxus_logger::tracing::info!("Fetching games");
        match db.select("game").await {
            Ok(games) => QueryResult::Ok(QueryValue::Games(games)),
            Err(_) => QueryResult::Err(QueryError::NoGames),
        }
    } else {
        QueryResult::Err(QueryError::Unknown)
    }
}

fn main() {
    dotenvy::dotenv().expect("Failed to read .env file");
    launch(App);
}

fn App() -> Element {
    dioxus_logger::tracing::info!("Initialised");

    // let db: Resource<Surreal<Client>> = use_resource(|| async move {
    //     let surreal: Surreal<Client> = Surreal::init();
    //     surreal.connect::<Ws>("http://surrealdb.eyeheartzombies.com").await.unwrap();
    //     surreal.signin(Root {
    //         username: &env::var("SURREAL_USER").unwrap(),
    //         password: &env::var("SURREAL_PASS").unwrap(),
    //     }).await.unwrap();
    //     surreal
    // });
    // use_context_provider(|| db.clone());

    rsx! {
        h1 { "Hangry Games" }
        GamesList {}
    }
}

#[component]
fn GamesList() -> Element {
    let games_query = use_get_query([QueryKey::AllGames, QueryKey::Games], fetch_games);
    let games = games_query.result();
    let games = games.value();
    dioxus_logger::tracing::info!("games {:?}", games);
    match games {
        QueryResult::Err(QueryError::NoGames) => {
            rsx! { p { "No games" } }
        }
        QueryResult::Ok(games) => {
            match games {
                QueryValue::Games(games) => {
                    if games.is_empty() {
                        rsx! { p { "No games yet" } }
                    } else {
                        rsx! {
                            for game in games {
                                p { "{game.name}" }
                            }
                        }
                    }
                },
                _ => {
                    rsx! { p { "Wrong result type" } }
                }
            }

        }
        QueryResult::Loading(_) => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx! { p { "No idea how you got here." } }
        }
    }
}
