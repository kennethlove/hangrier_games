use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use axum::extract::State;
use surrealdb::opt::auth::Record;
use crate::{AppError, AppState};

pub static USERS_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", get(session).post(user_create))
        .route("/authenticate", post(user_authenticate))
});

#[derive(Serialize, Deserialize, Debug)]
struct Params {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JwtResponse {
    jwt: String,
}

async fn session(state: State<AppState>) -> Result<Json<String>, AppError> {
    let res: Option<String> = state.db.query("RETURN <string>$session").await.unwrap().take(0).unwrap();
    Ok(Json(res.unwrap_or("No session data found!".into())))
}

async fn user_create(state: State<AppState>, Json(payload): Json<Params>) -> Result<Json<JwtResponse>, AppError> {
    let username = payload.username;
    let password = payload.password;

    match state.db.signup(Record {
        access: "user",
        namespace: "hangry-games",
        database: "games",
        params: Params {
            username: username.clone(),
            password: password.clone()
        },
    }).await {
        Ok(token) => {
            Ok(Json(JwtResponse { jwt: token.into_insecure_token() }))
        }
        Err(_) => {
            Err(AppError::DbError("Failed to create user".to_string()))
        }
    }
}

async fn user_authenticate(state: State<AppState>, Json(payload): Json<Params>) -> Result<Json<JwtResponse>, AppError> {
    let username = payload.username;
    let password = payload.password;

    match state.db.signin(Record {
        access: "user",
        namespace: "hangry-games",
        database: "games",
        params: Params {
            username: username.clone(),
            password: password.clone()
        },
    }).await {
        Ok(auth_user) => { Ok(Json(JwtResponse { jwt: auth_user.into_insecure_token() })) }
        Err(_) => {
            Err(AppError::DbError("Failed to authenticate user".to_string()))
        }
    }
}
