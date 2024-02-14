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
    let template = format!(r#"#Input (convert to pattern)
{}

#Quest (one liners)

#Notes (any additional information for yourselfe)"#,
    input);
    fs::write(path, template.as_bytes())?;

    let editor = env::var("EDITOR")?;
    Command::new(editor).arg(path).status()?;

    let file_input = fs::read_to_string(path)?;
    let mut name = String::new();
    let mut quest = String::new();
    let mut pattern = String::new();
    let mut note = String::new();

    for section in file_input.split("#").collect::<Vec<_>>() {
        if section.starts_with("Input") {
            let input = section.lines().nth(1).unwrap();
            name.push_str(input.split_whitespace().next().unwrap());
            pattern.push_str(input);
        } else if section.starts_with("Quest") {
            quest.push_str(section.lines().nth(1).unwrap());
        } else if section.starts_with("Notes") {
            let n: String = section.lines().skip(1).collect();
            note.push_str(&n);
        }
    }

    println!("name: {name}");
    println!("quest: {quest}");
    println!("pattern: {pattern}");
    println!("note: {note}");

    dotenv::dotenv().ok();
    let mut conn = establish_connection();
    
    let new_quest = NewQuest {
       cmd_name: &name,
       cmd_quest: &quest,
       cmd_pattern: &pattern,
       notes: &note,
       course_id: 1 
    };

    let res = diesel::insert_into(quests::table)
        .values(&new_quest)
        .execute(&mut conn)
        .expect("Error saving the quest");

    println!("User input: {}", file_input);
    println!("added: {:?}", input);
    
    // println!("{:?}", res);

    Ok(())
}
