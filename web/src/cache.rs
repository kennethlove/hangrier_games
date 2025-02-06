use game::games::Game;
use game::tributes::Tribute;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum QueryKey {
    AllGames,
    CreateGame(Option<String>),
    Game(String),
    Games,
    Tributes,
    GameTributes,
}

#[derive(PartialEq, Debug)]
pub(crate) enum QueryError {
    GameNotFound(String),
    NoGames,
    Unknown,
    BadJson,
}

#[derive(PartialEq, Debug)]
pub(crate) enum QueryValue {
    Games(Vec<Game>),
    Game(Game),
    Tributes(Vec<Tribute>),
}

#[derive(PartialEq, Debug)]
pub(crate) enum MutationValue {
    NewGame(Game),
    GameDeleted(String),
    NewTribute(Tribute),
}

#[derive(PartialEq, Debug)]
pub(crate) enum MutationError {
    UnableToCreateGame,
    Unknown,
    UnableToCreateTribute,
}
