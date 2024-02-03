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
    Wrong(Vec<char>, Vec<char>),
}

struct State {
    cmd_idx: usize,
    view: View,
}

impl State {
    fn new() -> Self {
        Self {
            cmd_idx: 0,
            view: View::Prompt,
        }
    }
}

fn render_question(buf: &mut ScreenBuf, question: &str) {
    for (i, c) in question.chars().enumerate() {
        buf.put_cell(Point::new(i, 0), Cell::new(c, style::Color::White));
    }
}

struct Prompt {
    input: Vec<char>,
    cursor: u16,
    
}

impl Prompt {
    fn new() -> Self {
        Self {
            input: Vec::new(),
            cursor: 0
        }
    }

    fn append_input(&mut self, ch: char) {
        self.input.push(ch);
        self.cursor += 1;
    }

    fn backspace_input(&mut self) {
        if self.input.len() > 0 {
            self.input.pop();
            self.cursor -= 1;
        }
    }

    fn get_input_string(&self) -> String {
       self.input.iter().collect() 
    }

    fn reset_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }

    fn render(&self, buf: &mut ScreenBuf) {
        for (i, c) in self.input.iter().enumerate() {
            buf.put_cell(Point::new(i, 2), Cell::new(*c, style::Color::White));
        }
    }

    fn sync_cursor(&self, out: &mut impl Write) -> io::Result<()>{
        out.queue(cursor::MoveTo(self.cursor, 2))?;
        Ok(())
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
    let mut prompt = Prompt::new();

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
                                prompt.append_input(x);
                            }
                        },
                        KeyCode::Backspace => {
                            prompt.backspace_input();
                        },
                        KeyCode::Enter => {
                            if state.view == View::Prompt {
                                let ast = CmdParser::compile(cmd);
                                let in_lex = InputCmdLexer::compile(&prompt.get_input_string());
                                let matcher = match_schema(&ast, &in_lex, 0, 0);
                                let is_full_match = matcher.iter().all(|x| x.1);

                                let mut line = Vec::new();
                                let mut underline = Vec::new(); 

                                for (value, is_match) in matcher {
                                    let symbol = if is_match { ' ' } else { '^' };
                                    for ch in value.chars() {
                                        line.push(ch);
                                        underline.push(symbol);
                                    }

                                    line.push(' ');
                                    underline.push(' ');
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

                            prompt.reset_input();
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
                prompt.render(&mut next_buf);
                prompt.sync_cursor(&mut stdout)?;
            },
            _ => {}
        }
        
        apply_patches(&mut stdout, &curr_buf.diff(&next_buf))?;
        stdout.flush()?;

        mem::swap(&mut curr_buf, &mut next_buf);
        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
