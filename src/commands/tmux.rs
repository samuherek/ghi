use std::fs;
use std::io::{self, stdout, Write};
use std::path::PathBuf;
use serde::Deserialize;
use crossterm::{execute, cursor};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType};
use crossterm::QueueableCommand;

#[derive(Deserialize)]
struct Cmd {
    cmd: String, 
    description: String,
    tag: String
}

struct ScreenTmux {
    input: String,
    quest: String,
    answer: String,
    text: Vec<String>,
    quit: bool,
}

impl ScreenTmux {
    fn enable() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(Self {
            input: String::new(),
            quest: String::new(),
            answer: String::new(),
            text: Vec::new(),
            quit: false,
        })
    }

    pub fn append_input(&mut self, x: char) {
        self.input.push(x);
    }

    pub fn backspace_input(&mut self) {
        self.input.pop();
    }

    fn reset(&self, qc: &mut impl Write) -> io::Result<()> {
        qc.queue(Clear(ClearType::All))?;
        qc.queue(cursor::MoveTo(0, 0))?;
        qc.queue(cursor::Hide)?;

        qc.flush()?;

        Ok(())
    }
}

impl Drop for ScreenTmux {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode: {err}")
        });
        let _ = execute!(stdout(), LeaveAlternateScreen).map_err(|err| {
            eprintln!("ERROR: leave alternate screen: {err}")
        });
    }
}


#[derive(Debug)]
enum Arg {
    String(String),
    Int(usize)
}

#[derive(Debug)]
enum Token { 
    Command(String),
    Flag(String),
    Argument(Arg),
}

// [] optional
// <> required
// ... previous element can repeat

fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let parts = input.split_whitespace().collect::<Vec<_>>();

    for part in parts {
        if part.starts_with('[') {
            

        } else if part.starts_with('-') {
            tokens.push(Token::Flag(part.to_string()));
        } else if part.starts_with('<') && part.ends_with('>') {
            let val = &part[1..part.len().saturating_sub(1)];
            let num = val.parse::<usize>().ok();

            if let Some(num) = num {
                tokens.push(Token::Argument(Arg::Int(num)));
            } else {
                tokens.push(Token::Argument(Arg::String(val.to_string())));
            }
        } else {
            tokens.push(Token::Command(part.to_string()));
        }
    }

    return tokens;
}


pub fn run() -> io::Result<()> {
    let data = fs::read_to_string(PathBuf::from("tmux.json")).unwrap();
    let course: Vec<Cmd> = serde_json::from_str(&data)?;
    let quest = course.into_iter().nth(0).unwrap();

    println!("{}", quest.cmd);
    // let l = lex(&quest.cmd);
    // println!("tokens:: {:?}", l);

    // let mut screen = ScreenTmux::enable()?;
    // let mut stdout = stdout();
    // let (screen_cols, screen_rows) = terminal::size()?;
    //
    // screen.quest = quest.description;



    // while !screen.quit {
    //     let _ = screen.reset(&mut stdout)?;
    //
    //     stdout.queue(cursor::MoveTo(0, 0))?;
    //     stdout.queue(style::Print(&screen.quest))?;
    //
    //     for (i, item) in screen.text.iter().enumerate() {
    //         stdout.queue(cursor::MoveTo(0, i as u16 + 2))?;
    //         stdout.queue(style::Print(item))?;
    //     }
    //
    //     for col in 0..screen_cols {
    //         stdout.queue(cursor::MoveTo(col, screen_rows - 2))?;
    //         stdout.queue(style::Print("-"))?;
    //     }
    //
    //     stdout.queue(cursor::MoveTo(0, screen_rows - 1))?;
    //     stdout.queue(style::Print(format!("{}", screen.input)))?;
    //
    //     stdout.flush()?;
    //
    //     if let Event::Key(event) = event::read()? {
    //         match event.code {
    //             KeyCode::Char(x) => {
    //                 if event.modifiers.contains(KeyModifiers::CONTROL) {
    //                     match x {
    //                         'c' => screen.quit = true,
    //                         _ => {}
    //                     }
    //                 } else {
    //                     screen.append_input(x);
    //                 }
    //             },
    //             KeyCode::Backspace => {
    //                 screen.backspace_input();
    //             },
    //             KeyCode::Enter => {
    //                 screen.text.push("submitted".to_string());
    //                 
    //                 let (cmd, args) = screen.input.split_once(" ").unwrap_or(("", ""));
    //                 if cmd.len() > 0 {
    //                     screen.text.push(cmd.to_string());
    //                     screen.text.push(args.to_string());
    //                 }
    //                     
    //                 //{
    //                 //        "command": "new-session -s <string>",
    //                 //        "description": "Create a new session.",
    //                 //        "tag": "session"
    //                 //    },
    //             },
    //             _ => {}
    //         }
    //     }
    // }

    Ok(())
}
