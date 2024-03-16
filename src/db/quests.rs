use diesel::prelude::*;
use diesel::SqliteConnection;
use super::schema::quests::dsl;
use super::models::Quest;

#[tracing::instrument(name = "Query quests", skip(conn))]
pub fn query_quests(conn: &mut SqliteConnection, lesson_id: i32) -> Vec<Quest> {
    match dsl::quests
        .filter(dsl::lesson_id.eq(lesson_id))
        .filter(dsl::pattern.is_not(""))
        .get_results(conn)
        {
            Ok(res) => {
                tracing::info!("Query quests has been successful");
                res 
            },
            Err(e) => {
                tracing::error!("Fialed to execute query qyests: {}", e);
                vec![]
            }
        }
}

#[tracing::instrument(name = "Query quest", skip(conn))]
pub fn query_quest(conn: &mut SqliteConnection, id: i32) -> Option<Quest> {
    match dsl::quests
        .find(id)
        .first(conn) 
        {
            Ok(res) => {
                tracing::info!("Query quest successful");
                Some(res)
            },
            Err(e) => {
                tracing::error!("Failed to execute query quest: {}", e);
                None
            }
        }
}
