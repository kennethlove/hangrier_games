use game::areas::Area;
use game::games::Game;
use serde::{Deserialize, Serialize};
use std::iter::zip;
use surrealdb::engine::local::Mem;
use surrealdb::{Error, RecordId, Surreal};
use game::areas::areas::Areas;

#[derive(Debug, Deserialize)]
struct Record {
    id: RecordId,
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
    let game: Result<Option<Game>, surrealdb::Error> = created_game.take(0_usize);
    let game = {
        if let Ok(Some(game)) = game {
            game
        } else {
            return Err(Error::from(surrealdb::error::Db::IdNotFound { value: "Invalid id".to_string() }))
        }
    };

    // Create areas
    let the_cornucopia: Option<Record> = area_db.create("area").content(Area::new("The Cornucopia")).await?;
    let northwest: Option<Record> = area_db.create("area").content(Area::new("Northwest")).await?;
    let northeast: Option<Record> = area_db.create("area").content(Area::new("Northeast")).await?;
    let southwest: Option<Record> = area_db.create("area").content(Area::new("Southwest")).await?;
    let southeast: Option<Record> = area_db.create("area").content(Area::new("Southeast")).await?;

    let northwest = northwest.unwrap().id;
    let northeast = northeast.unwrap().id;
    let southwest = southwest.unwrap().id;
    let southeast = southeast.unwrap().id;
    let the_cornucopia = the_cornucopia.unwrap().id;

    // Set Cornucopia neighbors
    let corn_neighbors: Vec<RecordId> = vec![northwest.clone(), northeast.clone(), southeast.clone(), southwest.clone()];
    for neighbor in corn_neighbors.iter() {
        let _ = area_db
            .query("RELATE $area1->neighbors->$area2")
            .bind(("area1", the_cornucopia.clone()))
            .bind(("area2", neighbor.clone()))
            .await?;
    }

    // Set NW neighbors
    let nw_neighbors: Vec<RecordId> = vec![the_cornucopia.clone(), northeast.clone(), southwest.clone()];
    for neighbor in nw_neighbors.iter() {
        let _ = area_db
            .query("RELATE $area1->neighbors->$area2")
            .bind(("area1", northwest.clone()))
            .bind(("area2", neighbor.clone()))
            .await?;
    }

    // Set NE neighbors
    let ne_neighbors: Vec<RecordId> = vec![the_cornucopia.clone(), northwest.clone(), southeast.clone()];
    for neighbor in ne_neighbors.iter() {
        let _ = area_db
            .query("RELATE $area1->neighbors->$area2")
            .bind(("area1", northeast.clone()))
            .bind(("area2", neighbor.clone()))
            .await?;
    }

    // Set SE neighbors
    let se_neighbors: Vec<RecordId> = vec![the_cornucopia.clone(), northeast.clone(), southwest.clone()];
    for neighbor in se_neighbors.iter() {
        let _ = area_db
            .query("RELATE $area1->neighbors->$area2")
            .bind(("area1", southeast.clone()))
            .bind(("area2", neighbor.clone()))
            .await?;
    }

    // Set SW neighbors
    let sw_neighbors: Vec<RecordId> = vec![the_cornucopia.clone(), northwest.clone(), southeast.clone()];
    for neighbor in sw_neighbors.iter() {
        let _ = area_db
            .query("RELATE $area1->neighbors->$area2")
            .bind(("area1", southwest.clone()))
            .bind(("area2", neighbor.clone()))
            .await?;
    }

    let result = area_db
        .query("SELECT ->neighbors->area.name FROM $area")
        .bind(("area", the_cornucopia.clone()))
        .await?;
    dbg!(result);

    Ok(())
}
