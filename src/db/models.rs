use diesel::prelude::*;
use chrono::NaiveDateTime;
use crate::db::schema::quests;
use crate::db::schema::lessons;
use crate::db::schema::bucket;

#[derive(Debug)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = quests)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Quest {
    pub id: i32,
    pub cmd: String,
    pub pattern: String,
    pub is_pattern_literal: bool,
    pub quest: String,
    pub notes: Option<String>,
    pub mock_output: Option<String>,
    pub display_count: i32,
    pub ok_count: i32,
    pub miss_count: i32,
    pub created_at: NaiveDateTime, 
    pub updated_at: NaiveDateTime, 
    pub lesson_id: i32
}

#[derive(Insertable)]
#[diesel(table_name = quests)]
pub struct NewQuest<'a> {
    pub cmd: &'a str,
    pub pattern: &'a str,
    pub quest: &'a str,
    pub is_pattern_literal: bool,
    pub notes: &'a str,
    pub lesson_id: i32,
}

#[derive(Debug)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = lessons)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Lesson {
    pub id: i32,
    pub cmd: String,
    pub name: String,
    pub description: String,
    pub remote: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = lessons)]
pub struct NewLesson<'a> {
    pub cmd: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub remote: bool
}

#[derive(Debug)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = bucket)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Bucket {
    pub id: i32,
    pub value: String, 
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = bucket)]
pub struct NewBucket<'a> {
    pub value: &'a str,
    pub notes: Option<&'a str>,
}

