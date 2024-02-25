use diesel::prelude::*;
use diesel::SqliteConnection;
use super::schema::quests::dsl;
use super::models::Quest;
use log::{info, error};

pub fn query_quests(conn: &mut SqliteConnection, id: i32) -> Vec<Quest> {
    info!("Query the quests wiht lesson id {}", id);

    let res = dsl::quests
        .filter(dsl::lesson_id.eq(id))
        .filter(dsl::pattern.is_not(""))
        .get_results(conn)
        .map_err(|err| {
            error!("Error running the query quests: {:?}", err);
        }).unwrap();

    info!("DB: quests: {:?}", res.len());

    res
}

pub fn query_quest(conn: &mut SqliteConnection, id: i32) -> Option<Quest> {
    info!("Query the quest wiht id {}", id);

    let res = dsl::quests
        .find(id)
        .first(conn)
        .map_err(|err| {
            error!("Error runing the query quest: {:?}", err);
        }).unwrap();

    info!("DB: quest done");

    Some(res) 
}
