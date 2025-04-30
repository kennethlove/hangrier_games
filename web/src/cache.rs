use game::areas::AreaDetails;
use game::games::{DisplayGame, Game};
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
    _TributeDayLog(String, u32), // Tribute identifier, day
    _GameSummary(String),
    _GameDaySummary(String, u32), // Game identifier, day
    User,
}

#[derive(PartialEq, Debug)]
pub(crate) enum QueryError {
    BadJson,
    GameNotFound(String),
    NoGames,
    TributeNotFound(String),
    Unauthorized,
    Unknown,
}

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub(crate) enum QueryValue {
    Areas(Vec<AreaDetails>),
    Game(Box<DisplayGame>),
    Games(Vec<DisplayGame>),
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
    GamePublished(String),
    GameUnpublished(String),
}

#[derive(PartialEq, Debug)]
pub(crate) enum MutationError {
    UnableToCreateGame,
    Unknown,
    UnableToAdvanceGame,
    UnableToRegisterUser,
    UnableToAuthenticateUser,
    _UnableToPublishGame,
    _UnableToUnpublishGame,
}
