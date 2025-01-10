use clap::{Parser, Subcommand};
use game::games::Game;
use serde_json::json;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    NewGame
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::NewGame) => {
            let mut game = Game::default();
            game.add_random_tribute();
            game.add_random_tribute();
            game.shuffle_tributes();

            println!("{}", json!(game));
        }
        None => {}
    }
}
