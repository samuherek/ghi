pub mod models;
pub mod schema;
pub mod lessons;
pub mod quests;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::{info, debug};
use super::GhiConfig;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn establish_connection(config: &GhiConfig) -> SqliteConnection {
    let database_url = config.database_dir.join("database.sql").display().to_string();
    debug!("database_url {}", database_url);
    info!("Setup database connection");
    let mut connection = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    info!("Run pending mutations.");
    if let Err(e) = connection.run_pending_migrations(MIGRATIONS) {
        panic!("Failed to run migrations with error: {}", e);
    }

    connection
}
