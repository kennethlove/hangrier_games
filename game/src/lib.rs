pub mod areas;
pub mod games;
pub mod items;
pub mod messages;
pub mod threats;
pub mod tributes;
mod database;
mod operations;


/// Initialize the library, including the database connection
pub fn initialize_library() {
    database::init_db();
    println!("Library initialized");
}
