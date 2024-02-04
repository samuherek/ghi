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
use crate::screen::{Screen, ScreenBuf, Point, Cell, Rect, apply_patches, GhiDebug};



#[derive(PartialEq)]
enum View {
    Prompt, 
    Correct,
    Wrong,
}

struct Prompt {
    cmd: Vec<char>,
    title: Vec<char>,
    input: Vec<char>,
    input_cursor: u16,
    ast: Vec<CmdWord>,
    feedback: Option<Vec<char>>,
    view: View,
    rect: Rect,
}

impl Prompt {
    fn new(title: &str, cmd: &str) -> Self {
        let cmd_name = cmd.split_whitespace().next().expect("Command must start with a command");
        let ast = CmdParser::compile(&cmd);
        let mut rect = Rect::default();
        rect.set_padding(Some(2), Some(4), Some(1), Some(4));

        Self {
            cmd: cmd_name.chars().collect(),
            title: title.chars().collect(),
            input: Vec::new(),
            input_cursor: 0,
            ast,
            feedback: None,
            view: View::Prompt,
            rect
        }
    }

    fn resize(&mut self, w: u16, h: u16) {
        let padd_x = w / 8;
        let padd_y = h / 10;
        let b_w = w - (2 * padd_x);
        let b_x = padd_x;
        let b_h = h - (2 * padd_y);
        let b_y = padd_y;
        self.rect.set_anchor(b_x, b_y);
        self.rect.set_dimensions(b_w, b_h);
    }

    fn set_question(&mut self, title: &str, cmd: &str) {
        let cmd_name = cmd.split_whitespace().next().expect("Command must start with a command");
        let ast = CmdParser::compile(&cmd);
        self.cmd =cmd_name.chars().collect();
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
                for _ in 0..value.len() {
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

    fn render_cmd_name(&self, buf: &mut ScreenBuf) {
        let tl = self.rect.top_left_padded(); 
        let mut text: Vec<Cell> = "Cmd: ".chars().map(|ch| Cell::new(ch,style::Color::White )).collect();
        for ch in self.cmd.iter() {
            text.push(Cell::new(*ch, style::Color::White));
        }
        buf.put_cells(tl, text);
    }

    fn render_question(&self, buf: &mut ScreenBuf) {
        let tl = self.rect.top_left_padded().add(0, 1); 
        let text = self.title.iter().map(|ch| Cell::new(*ch, style::Color::White)).collect();
        buf.put_cells(tl, text);
    }

    fn render_input(&self, buf: &mut ScreenBuf) {
        let tl = self.rect.bottom_left_padded(); 
        let text = self.input.iter().map(|ch| Cell::new(*ch, style::Color::White)).collect();
        buf.put_cells(tl, text);
    }

    fn render_wrong_feedback(&self, buf: &mut ScreenBuf) {
        let tl = self.rect.top_left_padded().add(0,4);
        let text = "You missed it:".chars().map(|ch| Cell::new(ch, style::Color::White)).collect();
        buf.put_cells(tl.clone(), text);

        let mut offset = 0;

        for (_, cmd_word) in self.ast.iter().enumerate() {
            let word = cmd_word.to_string();
            for (_, ch) in word.chars().enumerate() {
                buf.put_cell(tl.add(offset, 1), Cell::new(ch, style::Color::White));
                offset += 1;
            }
            buf.put_cell(tl.add(offset, 1), Cell::new(' ', style::Color::White));
            offset += 1;
        }

        if let Some(feedback) = &self.feedback {
            for (i, ch) in feedback.iter().enumerate() {
                buf.put_cell(tl.add(i as u16, 2), Cell::new(*ch, style::Color::White));
            }
        } else {
            panic!("Expected to have feedback, but it was missing");
        }
    }

    fn render_correct_feedback(&self, buf: &mut ScreenBuf) {
        let tl = self.rect.top_left_padded().add(0, 4); 
        let text = "You got it!".chars().map(|ch| Cell::new(ch, style::Color::White)).collect();
        buf.put_cells(tl, text);
    }

    fn render_debug(&self, buf: &mut ScreenBuf, rect: &Rect) {
        let top_left = rect.top_left();
        let text = "Debug".chars().map(|ch| Cell::new(ch, style::Color::White)).collect();
        buf.put_cells(top_left.clone(), text);

        

        // for (i, coord) in self.rect.debug().iter().enumerate() {
        //     let text = coord.chars().map(|ch| Cell::new(ch, style::Color::White)).collect();
        //     buf.put_cells(top_left.add(0, (1 + i) as u16), text);
        // }
    }

    fn render_boundary(&self, buf: &mut ScreenBuf) {
        let top_left = self.rect.top_left();
        let bottom_left = self.rect.bottom_left();
        let top_right = self.rect.top_right();

        let cells: Vec<Cell> = (0..self.rect.width())
            .enumerate()
            .map(|(i, _)| {
                let ch = if i % 2 == 0 {
                    '-'
                } else {
                    ' '
                };
                Cell::new(ch, style::Color::White)
            }).collect();
        buf.put_cells(top_left.clone(), cells.clone());
        buf.put_cells(bottom_left, cells);

        for i in 0..=self.rect.height() {
            buf.put_cell(top_left.add(0, i), Cell::new('-', style::Color::White));
            buf.put_cell(top_right.add(0, i), Cell::new('-', style::Color::White));
        }
    }

    fn sync_cursor(&self, out: &mut impl Write) -> io::Result<()>{
        match self.view {
            View::Prompt => {
                let tl = self.rect.bottom_left_padded().add((self.input.len()) as u16, 0);
                out.queue(cursor::Show)?;
                out.queue(cursor::MoveTo(tl.x, tl.y))?;
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
            let (_, description) = line.split_once(' ').expect("Title needs to have a value");
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
    prompt.resize(term_w, term_h);

    while !screen.quit {
        while poll(Duration::ZERO)? {
            match read()? {
                Event::Resize(next_width, next_height) => {
                    term_w = next_width;
                    term_h = next_height;
                    curr_buf.resize(term_w.into(), term_h.into());
                    next_buf.resize(term_w.into(), term_h.into());
                    prompt.resize(term_w, term_h);
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
        prompt.render_cmd_name(&mut next_buf);
        prompt.render_boundary(&mut next_buf);
        prompt.render_question(&mut next_buf);

        match prompt.view {
            View::Prompt => {
                prompt.render_input(&mut next_buf);
            },
            View::Correct => {
                prompt.render_correct_feedback(&mut next_buf);
            },
            View::Wrong => {
                prompt.render_wrong_feedback(&mut next_buf);
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
