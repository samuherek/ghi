use std::{fs, thread, mem};
use std::path::PathBuf;
use std::io::{self, stdout, Write};
use crate::parser::{CmdParser, CmdWord};
use crate::input_lexer::InputCmdLexer;
use crate::compare::match_schema;
use crossterm::{execute, cursor, style,QueueableCommand};
use crossterm::event::{self, KeyCode, KeyModifiers, Event, poll, read, KeyEventKind};
use crossterm::terminal::{self, LeaveAlternateScreen, Clear, ClearType, EnterAlternateScreen};
use anyhow;
use std::time::Duration;
use crate::screen::{Screen, ScreenBuf, Point, Cell, Rect, apply_patches};


#[derive(PartialEq)]
enum View {
    Prompt, 
    Correct,
    Wrong,
}

struct Prompt {
    title: Vec<char>,
    input: Vec<char>,
    input_cursor: u16,
    ast: Vec<CmdWord>,
    feedback: Option<Vec<char>>,
    view: View,
}

impl Prompt {
    fn new(title: &str, cmd: &str) -> Self {
        let ast = CmdParser::compile(&cmd);

        Self {
            title: title.chars().collect(),
            input: Vec::new(),
            input_cursor: 0,
            ast,
            feedback: None,
            view: View::Prompt,
        }
    }

    fn set_question(&mut self, title: &str, cmd: &str) {
        let ast = CmdParser::compile(&cmd);
        self.title = title.chars().collect();
        self.reset_input();
        self.ast = ast;
        self.feedback = None;
        self.view = View::Prompt;
    }

    fn submit_answer(&mut self) {
        let in_lex = InputCmdLexer::compile(&self.get_input_string());
        let matcher = match_schema(&self.ast, &in_lex, 0, 0);

        if matcher.iter().all(|x| x.1) {
            self.view = View::Correct;
        } else {
            let mut underline = Vec::new(); 

            for (value, is_match) in matcher {
                let symbol = if is_match { ' ' } else { '^' };
                for _ in [..=value.len()] {
                    underline.push(symbol);
                }
                underline.push(' ');
            }
            self.feedback = Some(underline);
            self.view = View::Wrong;
        }
    }

    fn append_input(&mut self, ch: char) {
        self.input.push(ch);
        self.input_cursor += 1;
    }

    fn backspace_input(&mut self) {
        if self.input.len() > 0 {
            self.input.pop();
            self.input_cursor -= 1;
        }
    }

    fn get_input_string(&self) -> String {
       self.input.iter().collect() 
    }

    fn reset_input(&mut self) {
        self.input.clear();
        self.input_cursor = 0;
    }

    fn render_question(&self, buf: &mut ScreenBuf, rect: &Rect) {
        let text = self.title.iter().map(|ch| Cell::new(*ch, style::Color::White)).collect();
        buf.put_cells(rect.top_left(), text);
    }

    fn render_input(&self, buf: &mut ScreenBuf, rect: &Rect) {
        let text = self.input.iter().map(|ch| Cell::new(*ch, style::Color::White)).collect();
        buf.put_cells(rect.top_left(), text);
    }

    fn render_wrong_feedback(&self, buf: &mut ScreenBuf, rect: &Rect) {
        let text = "You missed it:".chars().map(|ch| Cell::new(ch, style::Color::White)).collect();
        buf.put_cells(rect.top_left(), text);

        let mut offset = 0;

        for (_, word) in self.ast.iter().enumerate() {
            let word = word.to_string();
            for (_, ch) in word.chars().enumerate() {
                buf.put_cell(rect.point(offset, 1), Cell::new(ch, style::Color::White));
                offset += 1;
            }
            buf.put_cell(rect.point(offset, 1), Cell::new(' ', style::Color::White));
            offset += 1;
        }

        if let Some(feedback) = &self.feedback {
            for (i, ch) in feedback.iter().enumerate() {
                buf.put_cell(rect.point(i as u16, 2), Cell::new(*ch, style::Color::White));
            }
        } else {
            panic!("Expected to have feedback, but it was missing");
        }
    }

    fn render_correct_feedback(&self, buf: &mut ScreenBuf, rect: &Rect) {
        let text = "You got it!".chars().map(|ch| Cell::new(ch, style::Color::White)).collect();
        buf.put_cells(rect.top_left(), text);
    }

    fn render_debug(&self, buf: &mut ScreenBuf, rect: &Rect) {
        let text = "Debug".chars().map(|ch| Cell::new(ch, style::Color::White)).collect();
        buf.put_cells(rect.top_left(), text);
    }

    fn sync_cursor(&self, out: &mut impl Write) -> io::Result<()>{
        match self.view {
            View::Prompt => {
                out.queue(cursor::Show)?;
                out.queue(cursor::MoveTo(self.input_cursor, 2))?;
            },
            _ => {
                out.queue(cursor::Hide)?;
            }
        }
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

    let debug = true;
    let mut stdout = stdout();
    let mut screen = Screen::start()?;
    let (mut term_w, mut term_h) = terminal::size()?;
    let mut curr_buf = ScreenBuf::new(term_w.into(), term_h.into());
    let mut next_buf = ScreenBuf::new(term_w.into(), term_h.into());
    let mut cmd_idx = 0;
    let (title, cmd) = cmds.get(cmd_idx).unwrap();
    let mut prompt = Prompt::new(title, cmd);

    while !screen.quit {
        while poll(Duration::ZERO)? {
            match read()? {
                Event::Resize(next_width, next_height) => {
                    term_w = next_width;
                    term_h = next_height;
                    curr_buf.resize(term_w.into(), term_h.into());
                    next_buf.resize(term_w.into(), term_h.into());
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
                            if prompt.view == View::Prompt {
                                prompt.submit_answer();
                            } else {
                                cmd_idx += 1;
                                if cmd_idx > cmds.len() {
                                    screen.quit = true;
                                } else {
                                    let (t, c) = cmds.get(cmd_idx).unwrap();
                                    prompt.set_question(t, c);
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
        prompt.render_question(&mut next_buf, &Rect::new(0, 0, term_w, term_h));

        match prompt.view {
            View::Prompt => {
                prompt.render_input(&mut next_buf, &Rect::new(0, 2, term_w, term_h));
            },
            View::Correct => {
                prompt.render_correct_feedback(&mut next_buf, &Rect::new(0, 2, term_w, term_h));
            },
            View::Wrong => {
                prompt.render_wrong_feedback(&mut next_buf, &Rect::new(0, 2, term_w, term_h));
            },
        }

        if debug {
            let x_point = ((term_w / 3) * 2).into();
            let rect = Rect::new(x_point, 0, term_w, term_h);
            prompt.render_debug(&mut next_buf, &rect);
        }

        apply_patches(&mut stdout, &curr_buf.diff(&next_buf))?;
        prompt.sync_cursor(&mut stdout)?;
        stdout.flush()?;

        mem::swap(&mut curr_buf, &mut next_buf);
        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
