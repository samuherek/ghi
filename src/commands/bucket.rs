use std::io::{self, Read};
use anyhow::Result;
use diesel;
use diesel::SqliteConnection;
use crossterm;
use crate::window::Screen;
use crate::db::models::NewBucket;
use log::{error, info};

fn insert_bucket(conn: &mut SqliteConnection, new_bucket: NewBucket) -> Result<(), diesel::result::Error> {
    use diesel::prelude::*;
    use crate::db::schema::bucket;

    info!("DB: Inserting bucket to database");

    let insert = diesel::insert_into(bucket::table) 
        .values(&new_bucket)
        .execute(conn);

    match insert {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("Error saving a bucket to db: {:?}", err);
            Err(err)
        }
    }
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

    println!("Add context notes for later reference.");
    let notes = if let Ok(note) = read_input() {
        Some(note)
    } else {
        None
    };

    let new_bucket = NewBucket {
        value: &src,
        notes: notes.as_deref()
    };

    match insert_bucket(conn, new_bucket) {
        Ok(_) =>  {
            println!("");
            println!("added: \"{}\"", src);
        },
        Err(_) => {
            println!("Something unexpected happened!")
        }
    }

    Ok(())
}
