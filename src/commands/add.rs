use std::io::{self, Read};
use crate::store::Store;
use anyhow::Result;
use tempfile::NamedTempFile;
use std::process::Command;
use std::env;
use std::fs;
use crate::db::establish_connection;
use crate::db::schema::quests;
use diesel::SelectableHelper;
use crate::db::models::{Quest, NewQuest};
use chrono::Utc;
use crate::db::schema::quests::dsl::*;
use diesel::RunQueryDsl;
use diesel;
use dotenv;

pub fn run(value: &Option<String>) -> Result<()> {
    let mut store = Store::new();
    store.init_database()?;
    let mut input = String::new();

    if let Some(value) = value {
        store.create_from_string(value)?;
        input = value.clone();
    } else {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Ok(_) => {
                store.create_from_string(&buf)?;
                input = buf;
            }, 
            Err(err) => eprintln!("Error reading stdion: {}", err)
        };
    };

    let file = NamedTempFile::new()?;
    let path = file.path();
    fs::write(path, input.as_bytes())?;

    let editor = env::var("EDITOR")?;
    Command::new(editor).arg(path).status()?;

    let i = fs::read_to_string(path)?;

    dotenv::dotenv().ok();
    let mut conn = establish_connection();
    
    let new_quest = NewQuest {
       cmd_name: input.split_whitespace().next().unwrap(),
       query: "",
       pattern: &input,
       created_at: Utc::now().naive_utc(),
       updated_at: Utc::now().naive_utc()
    };

    let res = diesel::insert_into(quests::table)
        .values(&new_quest)
        .execute(&mut conn)
        .expect("Error saving the quest");

    println!("User input: {}", i);
    println!("added: {:?}", input);
    
    println!("{:?}", res);

    Ok(())
}
