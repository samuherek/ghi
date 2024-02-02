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

fn render_question(buf: &mut ScreenBuf, question: &str) {
    for (i, c) in question.chars().enumerate() {
        buf.put_cell(Point::new(i, 0), Cell::new(c, style::Color::White));
    }
}

fn render_input(buf: &mut ScreenBuf, input: &String) {
    for (i, c) in input.chars().enumerate() {
        buf.put_cell(Point::new(i, 2), Cell::new(c, style::Color::White));
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
    let (mut description, mut cmd) = cmds.get(state.cmd_idx).unwrap();

    while !screen.quit {
        while poll(Duration::ZERO)? {
            match read()? {
                Event::Resize(next_width, next_height) => {
                    w = next_width;
                    h = next_height;
                    curr_buf.resize(w.into(), h.into());
                    next_buf.resize(w.into(), h.into());
                    curr_buf.flush(&mut stdout)?;
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
                                } else {
                                    let (desc, cm) = cmds.get(state.cmd_idx).unwrap();
                                    cmd = cm;
                                    description = desc;
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
        render_question(&mut next_buf, &description);

        match state.view {
            View::Prompt => {
                render_input(&mut next_buf, &state.input);
            },
            _ => {}
        }
        
        apply_patches(&mut stdout, &curr_buf.diff(&next_buf))?;
        // sync_cursor();

        stdout.flush()?;

        mem::swap(&mut curr_buf, &mut next_buf);
        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
