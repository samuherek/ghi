use diesel::prelude::*;
use chrono::NaiveDateTime;
use crate::db::schema::quests;

#[derive(Debug)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = quests)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Quest {
    pub id: i32,
    pub cmd_name: String,
    pub pattern: String,
    pub query: String,
    pub created_at: NaiveDateTime, 
    pub updated_at: NaiveDateTime, 
}

#[derive(Insertable)]
#[diesel(table_name = quests)]
pub struct NewQuest<'a> {
    pub cmd_name: &'a str,
    pub pattern: &'a str,
    pub query: &'a str,
    pub created_at: NaiveDateTime, 
    pub updated_at: NaiveDateTime, 
}
