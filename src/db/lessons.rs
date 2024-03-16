use diesel::SqliteConnection;
use diesel::prelude::*;
use super::schema::lessons::dsl::*;
use super::models::Lesson;

#[tracing::instrument(name = "Query all lessons", skip(conn))]
pub fn query_all_lessons(conn: &mut SqliteConnection) -> Vec<Lesson> {
    match lessons.get_results(conn) {
        Ok(res) => {
            tracing::info!("Query all lessons successful");
            res
        },
        Err(e) => {
            tracing::error!("Failed to query all lessons: {}", e);
            vec![]
        }
    }
}

