use std::sync::{Arc, Mutex};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::Surreal;

pub struct Database {
    client: Surreal<Db>,
}

impl Database {
    pub async fn new() -> Self {
        let client = Surreal::new::<Mem>(()).await.unwrap();
        client.use_ns("hangrier games").use_db("games").await.unwrap();
        Self { client }
    }

    pub fn client(&self) -> &Surreal<Db> {
        &self.client
    }
}

lazy_static::lazy_static! {
    pub static ref DB: Arc<Mutex<Option<Database>>> = Arc::new(Mutex::new(None));
}

pub fn init_db() {
    let db = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { Database::new().await });

    let mut db_instance = DB.lock().unwrap();
    *db_instance = Some(db);

    println!("SurrealDB initialized");
}

pub fn get_db() -> Arc<Mutex<Option<Database>>> {
    DB.clone()
}
