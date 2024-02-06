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
    pub cmd_pattern: String,
    pub cmd_quest: String,
    pub notes: Option<String>,
    pub mock_output: Option<String>,
    pub display_count: i32,
    pub ok_count: i32,
    pub miss_count: i32,
    pub created_at: NaiveDateTime, 
    pub updated_at: NaiveDateTime, 
}

#[derive(Insertable)]
#[diesel(table_name = quests)]
pub struct NewQuest<'a> {
    pub cmd_name: &'a str,
    pub cmd_pattern: &'a str,
    pub cmd_quest: &'a str,
    pub notes: &'a str,
}
