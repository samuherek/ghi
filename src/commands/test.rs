use std::{fs, thread, mem};
use std::path::PathBuf;
use std::io::{self, stdout, Write};
use crate::parser::CmdParser;
use crate::input_lexer::InputCmdLexer;
use crate::compare::match_schema;
use crossterm::{execute, cursor, style,QueueableCommand};
use crossterm::event::{self, KeyCode, KeyModifiers, Event, poll, read, KeyEventKind};
use crossterm::terminal::{self, LeaveAlternateScreen, Clear, ClearType, EnterAlternateScreen};
use anyhow;
use std::time::Duration;
use crate::screen::{Screen, ScreenBuf, Point, Cell, apply_patches};

#[derive(PartialEq)]
enum View {
    Prompt, 
    Correct,
    Wrong(String, String),
}

struct State {
    cmd_idx: usize,
    input: String, 
    view: View,
}

impl State {
    fn new() -> Self {
        Self {
            cmd_idx: 0,
            input: String::new(), 
            view: View::Prompt,
        }
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

    let mut stdout = stdout();
    let mut screen = Screen::start()?;
    let (mut w, mut h) = terminal::size()?;
    let mut curr_buf = ScreenBuf::new(w.into(), h.into());
    let mut next_buf = ScreenBuf::new(w.into(), h.into());
    let mut state = State::new();
    let (description, cmd) = cmds.get(state.cmd_idx).unwrap();

    while !screen.quit {
        while poll(Duration::ZERO)? {
            match read()? {
                Event::Resize(next_width, next_height) => {
                    w = next_width;
                    h = next_height;
                    curr_buf.resize(w.into(), h.into());
                    next_buf.resize(w.into(), h.into());
                    curr_buf.flush(&mut stdout);
                },
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    match event.code {
                        KeyCode::Char(x) => {
                            if event.modifiers.contains(KeyModifiers::CONTROL) && x == 'c' {
                                screen.quit = true;
                            } else {
                                state.input.push(x);
                            }
                        },
                        KeyCode::Backspace => {
                            state.input.pop();
                        },
                        KeyCode::Enter => {
                            if state.view == View::Prompt {
                                let ast = CmdParser::compile(cmd);
                                let in_lex = InputCmdLexer::compile(state.input.trim());
                                let matcher = match_schema(&ast, &in_lex, 0, 0);
                                let is_full_match = matcher.iter().all(|x| x.1);

                                let mut line = String::new();
                                let mut underline = String::new();

                                for (value, is_match) in matcher {
                                    let symbol = if is_match { " " } else { "^" };
                                    line.push_str(value.as_str());
                                    underline.push_str(&symbol.repeat(value.len()));
                                    line.push_str(" ");
                                    underline.push_str(" ");
                                }
                                state.view = if is_full_match {
                                    View::Correct
                                } else {
                                    View::Wrong(line, underline)
                                }
                            } else {
                                state.cmd_idx += 1;
                                state.view = View::Prompt;

                                if state.cmd_idx > cmds.len() {
                                    screen.quit = true;
                                }
                            }

                            state.input.clear();
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        next_buf.clear();
        next_buf.put_cell(Point::new(0, 0), Cell::new('c', style::Color::White));

        apply_patches(&mut stdout, &curr_buf.diff(&next_buf))?;

        stdout.flush()?;

        mem::swap(&mut curr_buf, &mut next_buf);
        thread::sleep(Duration::from_millis(16));
    }



    // let (description, cmd) = cmds.get(screen.cmd_idx).unwrap();
    //     stdout.queue(cursor::MoveTo(0, 0))?;
    //     stdout.queue(style::Print(description))?;
    //
    //     match &screen.view {
    //         View::Prompt => {
    //             stdout.queue(cursor::MoveTo(0, 3))?;
    //             stdout.queue(Clear(ClearType::CurrentLine))?;
    //             stdout.queue(cursor::MoveTo(0, 2))?;
    //             stdout.queue(Clear(ClearType::CurrentLine))?;
    //             stdout.queue(style::Print(&screen.input))?;
    //         },
    //         View::Wrong(line, feedback) => {
    //             stdout.queue(cursor::MoveTo(0, 2))?;
    //             stdout.queue(Clear(ClearType::CurrentLine))?;
    //             stdout.queue(style::Print(line))?;
    //             stdout.queue(cursor::MoveTo(0, 3))?;
    //             stdout.queue(Clear(ClearType::CurrentLine))?;
    //             stdout.queue(style::Print(feedback))?;
    //             stdout.queue(cursor::MoveTo(0, 2))?;
    //         },
    //         View::Correct => {
    //             stdout.queue(cursor::MoveTo(0, 3))?;
    //             stdout.queue(Clear(ClearType::CurrentLine))?;
    //             stdout.queue(cursor::MoveTo(0, 2))?;
    //             stdout.queue(Clear(ClearType::CurrentLine))?;
    //             stdout.queue(style::Print(&"You are correct!"))?;
    //         }
    //     }
    //     stdout.flush()?;
    //
    //     let _ = terminal::disable_raw_mode().map_err(|err| {
    //         eprintln!("ERROR: disable raw mode: {err}")
    //     });
    //     let _ = execute!(stdout, LeaveAlternateScreen).map_err(|err| {
    //     eprintln!("ERROR: leave alternate screen: {err}")
    // });

    Ok(())
}
