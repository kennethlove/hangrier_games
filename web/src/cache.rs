use game::areas::AreaDetails;
use game::games::Game;
use game::messages::GameMessage;
use game::tributes::Tribute;
use shared::{AuthenticatedUser, TributeKey};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum QueryKey {
    AllGames,
    Game(String),
    Games,
    Tributes(String),
    Areas(String),
    Tribute(String),
    _GameLog(String), // Game identifier
    GameDayLog(String, u32), // Game identifier, day
    TributeLog(String), // Tribute identifier
    TributeDayLog(String, u32), // Tribute identifier, day
    _GameSummary(String),
    GameDaySummary(String, u32), // Game identifier, day
    User,
}

#[derive(PartialEq, Debug)]
pub(crate) enum QueryError {
    BadJson,
    GameNotFound(String),
    NoGames,
    TributeNotFound(String),
    Unknown,
}

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub(crate) enum QueryValue {
    Areas(Vec<AreaDetails>),
    Game(Box<Game>),
    Games(Vec<Game>),
    Tribute(Box<Tribute>),
    Tributes(Vec<Tribute>),
    GameTributes(Vec<TributeKey>),
    Logs(Vec<GameMessage>),
    Summary(String),
}

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub(crate) enum MutationValue {
    NewGame(Game),
    GameDeleted(String, String), // Identifier, name
    TributeDeleted(String),
    TributeUpdated(String),
    GameUpdated(String),
    GameFinished(String),
    GameStarted(String),
    GameAdvanced(String),
    User(AuthenticatedUser),
}

#[derive(PartialEq, Debug)]
pub(crate) enum MutationError {
    UnableToCreateGame,
    Unknown,
    UnableToAdvanceGame,
    UnableToCreateUser,
    UnableToRegisterUser,
    UnableToAuthenticateUser,
}
