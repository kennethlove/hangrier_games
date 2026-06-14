pub mod auth;
pub mod filters;
pub mod game_detail;
pub mod pages;
pub mod tera_engine;
pub mod timeline;
pub mod tribute_detail;

/// Authentication state passed to templates for conditional rendering.
#[derive(Clone)]
pub enum AuthState {
    Guest {
        csrf_token: String,
    },
    Authenticated {
        id: String,
        username: String,
        csrf_token: String,
    },
}

impl AuthState {
    pub fn guest(csrf: impl Into<String>) -> Self {
        AuthState::Guest {
            csrf_token: csrf.into(),
        }
    }

    pub fn authenticated(
        id: impl Into<String>,
        username: impl Into<String>,
        csrf: impl Into<String>,
    ) -> Self {
        AuthState::Authenticated {
            id: id.into(),
            username: username.into(),
            csrf_token: csrf.into(),
        }
    }

    pub fn csrf_token(&self) -> &str {
        match self {
            AuthState::Guest { csrf_token } | AuthState::Authenticated { csrf_token, .. } => {
                csrf_token
            }
        }
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated { .. })
    }

    pub fn username(&self) -> Option<&str> {
        match self {
            AuthState::Authenticated { username, .. } => Some(username),
            AuthState::Guest { .. } => None,
        }
    }
}
