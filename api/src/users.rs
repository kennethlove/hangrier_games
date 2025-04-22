use std::sync::LazyLock;
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Record;
use crate::DATABASE;

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
});

#[derive(Serialize, Deserialize, Debug)]
struct Params {
    email: String,
    pass: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JwtResponse {
    jwt: String,
}

pub async fn session() -> Result<Json<String>, error::Error> {
    let res: Option<String> = DATABASE.query("RETURN <string>$session").await?.take(0)?;
    Ok(Json(res.unwrap_or("No session data found!".into())))
}

pub async fn user_create(Json(payload): Json<Params>) -> Result<Json<JwtResponse>, error::Error> {
    let email = payload.email;
    let password = payload.pass;

    let jwt = DATABASE.signup(Record {
        access: "account",
        namespace: "hangry-games",
        database: "games",
        params: Params {
            email: email.clone(),
            pass: password.clone()
        },
    })
    .await?
    .into_insecure_token();

    Ok(Json(JwtResponse { jwt }))
}
