use std::io::{self, Read};
use anyhow::Result;
use diesel;
use diesel::SqliteConnection;
use crossterm;
use crate::window::Screen;

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
    let note = read_input().unwrap();

    println!("");
    println!("added: \"{}\"", src);

    Ok(())
}
