use std::fs;
use std::path::PathBuf;
use std::io::{self, stdout, Write};
use crate::parser::CmdParser;
use crate::input_lexer::InputCmdLexer;
use crate::compare::match_schema;
use crossterm::{execute, cursor, style};
use crossterm::event::{self, KeyCode, KeyModifiers, Event};
use crossterm::terminal::{self, LeaveAlternateScreen, Clear, ClearType, EnterAlternateScreen};
use crossterm::QueueableCommand;
use anyhow;

enum View {
    Prompt, 
    Correct,
    Wrong(String, String),
}

struct Screen {
    cmd_idx: usize,
    input: String, 
    quit: bool,
    view: View,
}

impl Screen {
    fn enable() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(Self {
            cmd_idx: 0,
            input: String::new() ,
            quit: false,
            view: View::Prompt,
        })
    }


    fn reset(&self, qc: &mut impl Write) -> io::Result<()> {
        qc.queue(Clear(ClearType::All))?;
        qc.queue(cursor::MoveTo(0, 0))?;
        qc.queue(cursor::Hide)?;

        qc.flush()?;

        Ok(())
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode: {err}")
        });
        let _ = execute!(stdout(), LeaveAlternateScreen).map_err(|err| {
            eprintln!("ERROR: leave alternate screen: {err}")
        });
    }
}

pub fn run() -> anyhow::Result<()>{
    let data = fs::read_to_string(PathBuf::from("test.txt")).unwrap();
    let lines: Vec<_> = data.lines().filter(|x| !x.is_empty()).collect();
    let mut cmds = Vec::new();
    let mut line_iter = lines.iter();

    while let Some(line) = line_iter.next() {
        if line.starts_with('#') {
            let description = *line;
            if let Some(line) = line_iter.next() {
                cmds.push((description, *line));
            } else {
                panic!("didn't find command for a description in the field");
            }
        }
    }

    let mut screen = Screen::enable()?;
    let mut stdout = stdout();
    let (screen_cols, screen_rows) = terminal::size()?;

    while !screen.quit {
        let (description, cmd) = cmds.get(screen.cmd_idx).unwrap();
        stdout.queue(cursor::MoveTo(0, 0))?;
        stdout.queue(style::Print(description))?;

        match &screen.view {
            View::Prompt => {
                stdout.queue(cursor::MoveTo(0, 3))?;
                stdout.queue(Clear(ClearType::CurrentLine))?;
                stdout.queue(cursor::MoveTo(0, 2))?;
                stdout.queue(Clear(ClearType::CurrentLine))?;
                stdout.queue(style::Print(&screen.input))?;
            },
            View::Wrong(line, feedback) => {
                stdout.queue(cursor::MoveTo(0, 2))?;
                stdout.queue(Clear(ClearType::CurrentLine))?;
                stdout.queue(style::Print(line))?;
                stdout.queue(cursor::MoveTo(0, 3))?;
                stdout.queue(Clear(ClearType::CurrentLine))?;
                stdout.queue(style::Print(feedback))?;
                stdout.queue(cursor::MoveTo(0, 2))?;
            },
            View::Correct => {
                stdout.queue(cursor::MoveTo(0, 3))?;
                stdout.queue(Clear(ClearType::CurrentLine))?;
                stdout.queue(cursor::MoveTo(0, 2))?;
                stdout.queue(Clear(ClearType::CurrentLine))?;
                stdout.queue(style::Print(&"You are correct!"))?;
            }
        }
        stdout.flush()?;

        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Char(x) => {
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        match x {
                            'c' => screen.quit = true,
                            _ => {}
                        }
                    } else {
                        screen.input.push(x);
                    }
                },
                KeyCode::Backspace => {
                    screen.input.pop();
                },
                KeyCode::Enter => {
                    match screen.view {
                        View::Prompt => {
                            let ast = CmdParser::compile(cmd);
                            let in_lex = InputCmdLexer::compile(screen.input.trim());
                            let matcher = match_schema(&ast, &in_lex, 0, 0);
                            let is_full_match = matcher.iter().all(|x| x.1);

                            let mut line = String::new();
                            let mut underline = String::new();

                            for (value, is_match) in matcher {
                                line.push_str(value.as_str());
                                if is_match {
                                    underline.push_str(&" ".repeat(value.len()));
                                } else {
                                    underline.push_str(&"^".repeat(value.len()));
                                }
                                line.push_str(" ");
                                underline.push_str(" ");
                            }
                            if is_full_match {
                                screen.view = View::Correct;
                            } else {
                                screen.view = View::Wrong(line, underline);
                            }
                        },
                        _ => {
                            screen.cmd_idx += 1;
                            screen.view = View::Prompt;

                            if screen.cmd_idx > cmds.len() {
                                screen.quit = true;
                            }
                        }
                    };
                    screen.input.clear();
                },
                _ => {}
            }
        }

            // stdout.flush()?;
        }

        let _ = terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode: {err}")
        });
        let _ = execute!(stdout, LeaveAlternateScreen).map_err(|err| {
        eprintln!("ERROR: leave alternate screen: {err}")
    });

    Ok(())
}
