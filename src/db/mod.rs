pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::env;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dirs;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
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
