use crate::DATABASE;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use surrealdb::opt::auth::Record;

mod error {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use axum::response::Response;
    use axum::Json;
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("database error")]
        Db,
    }

    impl IntoResponse for Error {
        fn into_response(self) -> Response {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(self.to_string())).into_response()
        }
    }

    impl From<surrealdb::Error> for Error {
        fn from(error: surrealdb::Error) -> Self {
            eprintln!("{error}");
            Self::Db
        }
    }
}

pub static USERS_ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/", post(user_create).get(session))
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

async fn session() -> Result<Json<String>, error::Error> {
    let res: Option<String> = DATABASE.query("RETURN <string>$session").await?.take(0)?;
    Ok(Json(res.unwrap_or("No session data found!".into())))
}

async fn user_create(Json(payload): Json<Params>) -> Result<Json<JwtResponse>, error::Error> {
    let username = payload.username;
    let password = payload.password;

    match DATABASE.signup(Record {
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
        Err(e) => {
            tracing::error!(target: "api", "Error creating user: {e}");
            Err(error::Error::Db)
        }
    }
}

async fn user_authenticate(Json(payload): Json<Params>) -> Result<Json<JwtResponse>, error::Error> {
    let username = payload.username;
    let password = payload.password;

    let jwt = DATABASE.signin(Record {
        access: "user",
        namespace: "hangry-games",
        database: "games",
        params: Params {
            username: username.clone(),
            password: password.clone()
        },
    })
    .await?
    .into_insecure_token();

    Ok(Json(JwtResponse { jwt }))
}
