use game::areas::Area;
use game::games::Game;
use serde::{Deserialize, Serialize};
use std::iter::zip;
use surrealdb::engine::local::Mem;
use surrealdb::{Error, RecordId, Surreal};

#[derive(Debug, Deserialize)]
struct Record {
    id: RecordId,
}

#[derive(Debug, Serialize, Deserialize)]
struct Neighbors {
    neighbors: Vec<Area>,
}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("hangry-games").await?;

    // Get the GAMES database
    let game_db = db.clone();
    game_db.use_db("games").await?;

    let area_db = db.clone();
    area_db.use_db("areas").await?;

    // Create a game
    let create_game: Option<Record> = game_db
        .create("game")
        .content(Game::default())
        .await?;

    // Get created game
    let mut created_game = game_db
        .query("SELECT * FROM game WHERE id = $id")
        .bind(("id", create_game.unwrap().id))
        .await?;
    // dbg!(&created_game);
    let game: Result<Option<Game>, surrealdb::Error> = created_game.take(0_usize);
    let game = {
        if let Ok(Some(game)) = game {
            game
        } else {
            return Err(Error::from(surrealdb::error::Db::IdNotFound { value: "Invalid id".to_string() }))
        }
    };
    // dbg!(&game);

    let area_names = ["the cornucopia", "northwest", "northeast", "southwest", "southeast"];
    for area_name in &area_names {
        let _: Option<Record> = area_db.create("area").content(Area::new(area_name)).await?;
    }

    let area_records: Vec<Record> = area_db.select("area").await?;
    let areas: Vec<Area> = area_db.select("area").await?;

    for (area, record) in zip(areas, area_records) {
        match area.name.as_str() {
            "northwest" => {},
            "northeast" => {},
            "southwest" => {},
            "southeast" => {},
            _ => {}
        }
    }

    Ok(())
}
