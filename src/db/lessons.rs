use diesel::SqliteConnection;
use diesel::prelude::*;
use super::schema::lessons::dsl::*;
use super::models::Lesson;
use log::{error, info};

pub fn query_all_lessons(conn: &mut SqliteConnection) -> Vec<Lesson> {
    info!("Query the lessons");

    let res = lessons.get_results(conn)
        .map_err(|err| {
            error!("Error running the query lessons: {:?}", err);
        }).unwrap();

    info!("DB: lessons: {:?}", res.len());

    res
}

