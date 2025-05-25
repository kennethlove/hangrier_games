use game::areas::AreaDetails;
use game::games::Game;
use game::messages::GameMessage;
use game::tributes::Tribute;
use shared::{AuthenticatedUser, DisplayGame, TributeKey};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum QueryKey {
    AllGames,
    DisplayGame(String),
    _DisplayGames,
    Game(String),
    Games,
    Tributes(String),
    Areas(String),
    Tribute(String, String), // Game identifier, tribute identifier
    _GameLog(String), // Game identifier
    GameDayLog(String, u32), // Game identifier, day
    TributeLog(String), // Tribute identifier
    _TributeDayLog(String, u32), // Tribute identifier, day
    _GameSummary(String),
    _GameDaySummary(String, u32), // Game identifier, day
    User,
    ServerVersion,
}

#[derive(PartialEq, Debug)]
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
#[derive(PartialEq, Debug)]
pub(crate) enum QueryValue {
    Areas(Vec<AreaDetails>),
    DisplayGame(Box<DisplayGame>),
    DisplayGames(Vec<DisplayGame>),
    Game(Box<Game>),
    Games(Vec<Game>),
    Tribute(Box<Tribute>),
    Tributes(Vec<Tribute>),
    GameTributes(Vec<TributeKey>),
    Logs(Vec<GameMessage>),
    Summary(String),
    ServerVersion(String),
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
