pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::env;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dirs;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection() -> SqliteConnection {
    println!("here we are");
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
    println!("here we are 2");
        match dirs::home_dir() {
            Some(p) => p.join(".ghi/test_database.sql").display().to_string(),
            None => panic!("Could not resolve home direcotry")
        }
    });

    let mut connection = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    if let Err(e) = connection.run_pending_migrations(MIGRATIONS) {
        panic!("Failed to run migrations with error: {}", e);
    }

    connection
}

pub fn ensure_tables(con: &mut SqliteConnection) {
    use crate::db::schema::lessons;
    use crate::db::schema::lessons::dsl::*;
    use crate::db::models::Lesson;

    if let Ok([0]) = lessons.filter(name.eq("default")).count().get_results::<i64>(con).as_deref() {
        println!("no default lesson") ;
    } else {
        println!("has default lesson");
    }
}
