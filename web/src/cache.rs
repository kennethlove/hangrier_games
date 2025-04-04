use game::areas::AreaDetails;
use game::games::Game;
use game::messages::GameMessage;
use game::tributes::Tribute;
use shared::TributeKey;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum QueryKey {
    AllGames,
    Game(String),
    Games,
    Tributes(String),
    Areas(String),
    Tribute(String),
    GameLog(String), // Game identifier
    GameDayLog(String, u32), // Game identifier, day
    TributeDayLog(String, u32), // Tribute identifier, day
    GameSummary(String),
    GameDaySummary(String, u32), // Game identifier, day
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
}

#[derive(PartialEq, Debug)]
pub(crate) enum MutationError {
    UnableToCreateGame,
    Unknown,
    UnableToAdvanceGame,
}
