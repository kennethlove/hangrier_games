pub mod auth;
pub mod dev;
pub mod games;

// Re-export all handler functions so main.rs can access them via `use routes::*;`
pub use auth::{
    auth_handler, check_email_handler, email_verified_handler, login_post_handler, logout_handler,
    register_post_handler, resend_verification_handler, verify_email_handler,
};
pub use dev::dev_verify_email_handler;
pub use games::{
    account_handler, create_game_handler, create_game_post_handler, game_areas_handler,
    game_detail_handler, game_log_handler, game_tribute_detail_handler, game_tributes_handler,
    games_list_handler, home_handler,
};
