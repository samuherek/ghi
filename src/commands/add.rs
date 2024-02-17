use std::io::{self, Read};
use anyhow::Result;
use crate::db::schema::quests;
use crate::db::models::NewQuest;
use diesel::RunQueryDsl;
use diesel;
use diesel::SqliteConnection;
use crossterm;
use crate::window::Screen;

fn get_default_lesson(conn: &mut SqliteConnection) -> i32 {
    use diesel::prelude::*;
    use crate::db::schema::lessons::dsl::*;
    let default_id = lessons.filter(name.eq("default")).select(id).first(conn).expect("Failure to find default lesson");
    default_id
}

fn read_input() -> Result<String> {
    use crossterm::{terminal, event, execute};
    use std::io::Write;

    let mut stdout = std::io::stdout();
    let mut page = Screen::start()?;

    let mut answer = String::new();
    while !page.get_quit() {
        while event::poll(std::time::Duration::ZERO)? {
            match event::read()? {
                event::Event::Key(ev) if ev.kind == event::KeyEventKind::Press => {
                    match ev.code {
                        event::KeyCode::Char(ch) => {
                            if ev.modifiers.contains(event::KeyModifiers::CONTROL) && ch == 'c' {
                                page.set_quit();
                                std::process::exit(1);
                            } else {
                                print!("{ch}");
                                stdout.flush()?;
                                answer.push(ch);
                            }
                        },
                        event::KeyCode::Enter => page.set_quit(), 
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
    let mut src = String::new(); 

    if let Some(value) = value {
        src = value.clone();
    } else {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Ok(_) => { src = buf; }, 
            Err(err) => eprintln!("Error reading stdion: {}", err)
        };
    };

    let name = src.split_whitespace().next().unwrap_or_default();
    println!("Command name: {name}");
    println!("---");
    println!("Transalte {} to pattern: (read docs to see how to do it)", src);
    let pattern = read_input().unwrap();

    println!("\n---");
    println!("Provide the question for the quest");
    let quest = read_input().unwrap();

    println!("\n---");
    println!("Any notes about this command? (press enter for none)");
    let note = read_input().unwrap();

    let lesson_id = get_default_lesson(conn);

    let new_quest = NewQuest {
       cmd: &name,
       quest: &quest,
       pattern: &pattern,
       is_pattern_literal: false,
       notes: &note,
       lesson_id
    };

    let _res = diesel::insert_into(quests::table)
        .values(&new_quest)
        .execute(conn)
        .expect("Error saving the quest");

    println!("command \"{}\" added", src);

    Ok(())
}
