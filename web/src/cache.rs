#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum QueryError {
    BadJson,
    GameNotFound(String),
    NoGames,
    TributeNotFound(String),
    Unauthorized,
    Unknown,
    ServerNotFound,
    ServerVersionNotFound,
}

#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum MutationError {
    UnableToCreateGame,
    Unknown,
    UnableToAdvanceGame,
    UnableToRegisterUser,
    UnableToAuthenticateUser,
    _UnableToPublishGame,
    _UnableToUnpublishGame,
}
