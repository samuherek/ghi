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
use crate::screen;

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
}

fn read_input() -> Result<String> {
    use crossterm::{terminal, event, execute};
    use std::io::Write;

    let mut stdout = std::io::stdout();
    let mut page = screen::Screen::start()?;

    let mut answer = String::new();
    while !page.quit {
        while event::poll(std::time::Duration::ZERO)? {
            match event::read()? {
                event::Event::Key(ev) if ev.kind == event::KeyEventKind::Press => {
                    match ev.code {
                        event::KeyCode::Char(ch) => {
                            if ev.modifiers.contains(event::KeyModifiers::CONTROL) && ch == 'c' {
                                page.quit = true;
                            } else {
                                print!("{ch}");
                                stdout.flush()?;
                                answer.push(ch);
                            }
                        },
                        event::KeyCode::Enter => page.quit(),
                        event::KeyCode::Backspace => { 
                            answer.pop(); 
                            execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
                            print!("\r{answer}");
                            stdout.flush()?;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(answer)
}

pub fn run(conn: &mut SqliteConnection, value: &Option<String>) -> Result<()> {
    let mut state = State::new();

    if let Some(value) = value {
        state.src = value.clone();
    } else {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Ok(_) => { state.src = buf; }, 
            Err(err) => eprintln!("Error reading stdion: {}", err)
        };
    };

    let name = state.src.split_whitespace().next().unwrap_or_default();
    println!("Command name: {name}");
    println!("---");
    println!("Transalte {} to pattern: (read docs to see how to do it)", state.src);
    let pattern = read_input().unwrap();

    println!("\n---");
    println!("Provide the question for the quest");
    let quest = read_input().unwrap();

    println!("\n---");
    println!("Any notes about this command? (press enter for none)");
    let note = read_input().unwrap();

    let new_quest = NewQuest {
       cmd_name: &name,
       cmd_quest: &quest,
       cmd_pattern: &pattern,
       notes: &note,
       lesson_id: 1 
    };

    let res = diesel::insert_into(quests::table)
        .values(&new_quest)
        .execute(conn)
        .expect("Error saving the quest");

    //
    // println!("User input: {}", file_input);
    // println!("added: {:?}", input);
    // 
    println!("{:?}", res);

    Ok(())
}
