pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::env;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dirs;
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

pub fn ensure_tables(con: &mut SqliteConnection) {
    use crate::db::schema::lessons;
    use crate::db::schema::lessons::dsl::*;
    use crate::db::models::NewLesson;

    info!("DB: ensure the base tables are ready.");

    if let Ok([0]) = lessons.filter(name.eq("default")).count().get_results::<i64>(con).as_deref() {
        let default_lesson = NewLesson {
            name: "default" ,
            cmd: "",
            description: "The default bucket for added commands",
            remote: false
        };

        let _res = diesel::insert_into(lessons::table)
            .values(&default_lesson)
            .execute(con)
            .expect("Error saving the default lesson");
    } 
}
