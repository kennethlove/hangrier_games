use game::games::Game;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum QueryKey {
    AllGames,
    CreateGame(Option<String>),
    Game(String),
    Games,
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
}

#[derive(PartialEq, Debug)]
pub(crate) enum MutationValue {
    NewGame(Game),
    GameDeleted(String),
}

#[derive(PartialEq, Debug)]
pub(crate) enum MutationError {
    UnableToCreateGame,
    Unknown,
}
