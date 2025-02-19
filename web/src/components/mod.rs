mod app;
pub(crate) use app::App;

mod create_game;
pub(crate) use create_game::CreateGameButton;
pub(crate) use create_game::CreateGameForm;

mod games;
pub(crate) use games::Games;

mod game_detail;
pub(crate) use game_detail::GameDetailPage;

mod games_list;
pub(crate) use games_list::GamesList;

mod game_delete;
mod game_tributes;
mod tribute_delete;
mod tribute_edit;
mod game_edit;

pub(crate) use game_delete::DeleteGameModal;
pub(crate) use game_delete::GameDelete;






