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
use diesel::SqliteConnection;
use crossterm;

enum View {
    Name,
    Pattern,
    Quest,
    Notes,
}

struct State {
    src: String,
    name: String, 
    pattern: String, 
    quest: String,
    notes: String,
    view: View,
}

impl State {
    fn new() -> Self {
        Self {
            src: String::new(),
            name: String::new(),
            pattern: String::new(),
            quest: String::new(),
            notes: String::new(),
            view: View::Name
        }
    }

    fn start() {
        let _ = crossterm::terminal::enable_raw_mode().map_err(|err| {
            eprintln!("ERROR: enable raw mode : {}", err);
        });
    }
}

impl Drop for State {
    fn drop(&mut self) {
       let _ = crossterm::terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode : {}", err);
       });
    } 
}

fn read_input() -> Result<String> {
    use crossterm::event;
    let mut answer = String::new();
    loop {
        if let event::Event::Key(event::KeyEvent { code, .. }) = event::read()? {
            match code {
                event::KeyCode::Char(c) => answer.push(c),
                event::KeyCode::Enter => break,
                event::KeyCode::Backspace => { answer.pop(); },
                _ => {}
            }
        }
    }
    Ok(answer)
}

pub fn run(conn: &SqliteConnection, value: &Option<String>) -> Result<()> {
    let mut state = State::new();

    if let Some(value) = value {
        state.src = value.clone();
    } else {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Ok(_) => {
                state.src = buf;
            }, 
            Err(err) => eprintln!("Error reading stdion: {}", err)
        };
    };

    let name = state.src.split_whitespace().next().unwrap_or_default();
    println!("Command grouping is: {name}");
    println!("Provide the pattern");

    let value = read_input().unwrap();

     println!("we are kind of here {}", state.src);
     println!("pattern {}", value);
    
    // let new_quest = NewQuest {
    //    cmd_name: &name,
    //    cmd_quest: &quest,
    //    cmd_pattern: &pattern,
    //    notes: &note,
    //    lesson_id: 1 
    // };
    //
    // let res = diesel::insert_into(quests::table)
    //     .values(&new_quest)
    //     .execute(&mut conn)
    //     .expect("Error saving the quest");
    //
    // println!("User input: {}", file_input);
    // println!("added: {:?}", input);
    // 
    // println!("{:?}", res);

    Ok(())
}
