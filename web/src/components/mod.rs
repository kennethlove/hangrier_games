mod app;

pub(crate) use app::App;
use dioxus::prelude::web;

mod create_game;
pub(crate) use create_game::CreateGameButton;
pub(crate) use create_game::CreateGameForm;

mod games;
pub(crate) use games::Games;

mod game_detail;
pub(crate) use game_detail::GamePage;

mod games_list;
pub(crate) use games_list::GamesList;

mod game_delete;
pub(crate) use game_delete::DeleteGameModal;
pub(crate) use game_delete::GameDelete;

mod game_areas;
mod game_edit;
mod game_tributes;
mod navbar;
mod tribute_detail;
mod tribute_edit;
pub(crate) use navbar::Navbar;
mod full_game_log;
mod home;
pub mod icons;
pub(crate) use full_game_log::GameDayLog;

pub(crate) use home::Home;

pub(crate) use tribute_detail::TributeDetail;

mod game_day_summary;
pub(crate) use game_day_summary::GameDaySummary;
