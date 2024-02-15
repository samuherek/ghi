use diesel::SqliteConnection;
use crossterm::{execute, terminal, QueueableCommand, cursor};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Color;
use crate::screen;
use std::io::Write;
use crate::db::models;

fn query_lessons(conn: &mut SqliteConnection) -> Vec<models::Lesson> {
    use diesel::prelude::*;
    use crate::db::schema::lessons::dsl::*;

    let res = lessons.get_results(conn).expect("Getting lessons faild");
    res
}

enum View {
    Lessons,
    Quests
}

struct State {
    lessons_idx: usize,
    quests_idx: usize,
    lessons: Vec<models::Lesson>,
    quests: Vec<models::Quest>,
    view: View
}

impl State {
    fn new(conn: &mut SqliteConnection) -> Self {
        let lessons = query_lessons(conn);

        Self {
            lessons_idx: 0,
            quests_idx: 0,
            lessons,
            quests: Vec::new(),
            view: View::Lessons
        }
    }

    fn select_up(&mut self) {
        match self.view {
            View::Lessons => self.lessons_idx = self.lessons_idx.saturating_sub(1),
            View::Quests => self.quests_idx = self.quests_idx.saturating_sub(1),
        }
    }

    fn select_down(&mut self) {
        match self.view {
            View::Lessons => {
                let max = self.lessons.len() - 1;
                let max_len = if self.lessons_idx >= max {
                    max
                } else {
                    self.lessons_idx + 1
                };
                self.lessons_idx = max_len;
            },
            View::Quests => {
                let max = self.quests.len();
                let max_len = if self.quests_idx >= max {
                    max
                } else {
                    self.quests_idx + 1
                };
                self.quests_idx = max_len;
            }
        }
    }

    fn select(&mut self) {
        match self.view {
            View::Lessons => self.view = View::Quests,
            View::Quests => self.view = View::Lessons,
        }
    }

    fn render_header(&self, page: &mut Screen, point: screen::Point) {
        let text = "Your available courses".chars().map(|ch| screen::Cell::new(ch, Color::White)).collect();
        page.next_buf.put_cells(point, text);
    }

    fn render_divider(&self, page: &mut Screen, point: screen::Point) {
        let cells = vec![screen::Cell::new('-', Color::White); page.rect.width().into()];
        page.next_buf.put_cells(point, cells);
    }

    fn render_back(&self, page: &mut Screen, point: screen::Point) {
        let cells = "..".chars().map(|ch| screen::Cell::new(ch, Color::White)).collect();
        page.next_buf.put_cells(point.add(3, 0), cells);
    }

    fn render_selection_caret(&self, page: &mut Screen, point: screen::Point) {
        let idx = match self.view {
            View::Lessons => self.lessons_idx,
            View::Quests => self.quests_idx,
        };
        let pnt = point.add(0, idx as u16);
        page.next_buf.put_cell(pnt, screen::Cell::new('>', Color::White));
    }

    fn render_lessons(&self, page: &mut Screen, point: screen::Point) {
        for (offset, lesson) in self.lessons.iter().enumerate() {
            let point = point.add(3, offset as u16);
            let cells = lesson.cmd.chars().map(|ch| screen::Cell::new(ch, Color::White)).collect();
            page.next_buf.put_cells(point, cells);
        }
    }

    fn render_quests(&self, page: &mut Screen, point: screen::Point) {
        for (offset, quest) in self.quests.iter().enumerate() {
            let point = point.add(3, offset as u16);
            let cells = quest.cmd_name.chars().map(|ch| screen::Cell::new(ch, Color::White)).collect();
            page.next_buf.put_cells(point, cells);
        }
    }

    fn render(&self, page: &mut Screen) {
        match self.view {
            View::Lessons => {
                let pnt = page.rect.top_left_padded();
                self.render_header(page, pnt.add(0, 0));
                self.render_divider(page, pnt.add(0, 3));
                self.render_lessons(page, pnt.add(0, 4));
                self.render_selection_caret(page, pnt.add(0, 4)); 
                let _ = page.stdout.queue(cursor::Hide);
            },
            View::Quests => {
                let pnt = page.rect.top_left_padded();
                self.render_header(page, pnt.add(0, 0));
                self.render_divider(page, pnt.add(0, 3));
                self.render_back(page, pnt.add(0, 4));
                self.render_quests(page, pnt.add(0, 5));
                self.render_selection_caret(page, pnt.add(0, 4)); 
                let _ = page.stdout.queue(cursor::Hide);
            }
        }
    }
}

struct Screen {
    stdout: std::io::Stdout,
    quit: bool,
    curr_buf: screen::ScreenBuf, 
    next_buf: screen::ScreenBuf,
    width: u16,
    height: u16,
    rect: screen::Rect,
}

impl Screen {
    pub fn start() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        let (width, height) = terminal::size()?;
        let mut rect = screen::Rect::default();
        rect.set_dimensions(width, height);

        Ok(Self {
            stdout,
            quit: false,
            curr_buf: screen::ScreenBuf::new(width, height),
            next_buf: screen::ScreenBuf::new(width, height),
            width,
            height,
            rect
        })
    }

   pub fn resize(&mut self, width: u16, height: u16) -> std::io::Result<()> {
        self.curr_buf.resize(width, height);
        self.next_buf.resize(width, height);
        self.width = width;
        self.height = height;
        self.curr_buf.flush(&mut self.stdout)?;
        Ok(())
   } 

   pub fn apply_patches(&mut self) -> std::io::Result<()> {
       use crossterm::cursor::MoveTo;
       use crossterm::style::{SetBackgroundColor, SetForegroundColor, Print, ResetColor};
       let diff = self.curr_buf.diff(&self.next_buf);

       for screen::Patch{ cell: screen::Cell{ ch, fg, bg }, x, y } in diff {
           self.stdout.queue(MoveTo(x as u16, y as u16))?;
           if let Some(bg) = bg {
               self.stdout.queue(SetBackgroundColor(bg))?;
           } else {
               self.stdout.queue(ResetColor)?;
           }
           self.stdout.queue(SetForegroundColor(fg))?;
           self.stdout.queue(Print(ch))?;
       }

       Ok(())
   }

   pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.curr_buf, &mut self.next_buf);
   }

   pub fn quit(&mut self) {
       self.quit = true; 
   }
}


impl Drop for Screen {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode: {err}")
        });

        let _ = execute!(self.stdout, terminal::LeaveAlternateScreen).map_err(|err| {
            eprintln!("ERROR: leave alternate screen: {err}")
        });
    }
}


pub fn run(conn: &mut SqliteConnection) -> std::io::Result<()> {
    let mut state = State::new(conn);
    let mut page = Screen::start()?;

    while !page.quit {
        while poll(std::time::Duration::ZERO)? {
            match read()? {
                Event::Resize(next_width, next_height) => {
                    page.resize(next_width, next_height)?;
                },
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    match event.code {
                        KeyCode::Char(ch) => {
                            if event.modifiers.contains(KeyModifiers::CONTROL) && ch == 'c' {
                                page.quit()
                            } else {
                                match ch {
                                    'j' => state.select_down(),
                                    'k' => state.select_up(),
                                    _ =>  {}
                                }
                            }
                        },
                        KeyCode::Up => state.select_up(),
                        KeyCode::Down => state.select_down(),
                        KeyCode::Enter => state.select(),
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        page.next_buf.clear();
        state.render(&mut page);
        page.apply_patches()?;
        page.stdout.flush()?;
        page.swap_buffers();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}