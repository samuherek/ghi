use diesel::SqliteConnection;
use crossterm::{execute, terminal, QueueableCommand, cursor};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Color;
use crate::window::{ScreenBuf, Cell, Point, Patch, Rect};
use std::io::Write;
use crate::db::models;
use log::{info, error};
use crate::db::lessons::query_all_lessons;
use crate::db::quests::{query_quests, query_quest};

enum View {
    Lessons,
    Quests,
    Quest
}

struct State {
    lessons_idx: usize,
    quests_idx: usize,
    lessons: Vec<models::Lesson>,
    quests: Vec<models::Quest>,
    quest: Option<models::Quest>,
    view: View
}

impl State {
    fn new(conn: &mut SqliteConnection) -> Self {
        let lessons = query_all_lessons(conn);

        Self {
            lessons_idx: 0,
            quests_idx: 0,
            lessons,
            quests: Vec::new(),
            quest: None,
            view: View::Lessons
        }
    }

    fn select_up(&mut self) {
        match self.view {
            View::Lessons => self.lessons_idx = self.lessons_idx.saturating_sub(1),
            View::Quests => self.quests_idx = self.quests_idx.saturating_sub(1),
            _ => {}
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
            },
            _ => {}
        }
    }

    fn select(&mut self, conn: &mut SqliteConnection) {
        match self.view {
            View::Lessons => {
                self.view = View::Quests;
                let id = self.lessons[self.lessons_idx].id;
                info!("Switch to quests with lesson_id: {}", id);
                self.quests = query_quests(conn, id);
            },
            View::Quests => {
                if self.quests_idx == 0 {
                    info!("Switch to lessons");
                    self.view = View::Lessons;
                } else {
                    self.view = View::Quest;
                    info!("We are about to get the quest id, {}", self.quests_idx);
                    if let Some(quest) = self.quests.get(self.quests_idx - 1) {
                        info!("We are going to render quest with id {}", quest.id);
                        self.quest = query_quest(conn, quest.id);
                    } else {
                        error!("Could not find the quest with index {}", self.quests_idx - 1);
                    }
                }
            },
            View::Quest => {
                info!("Switch to Quests");
                self.view = View::Quests;
                self.quest = None;
            }
        }
    }

    fn get_selection_idx(&self) -> u16 {
        match self.view {
            View::Lessons => self.lessons_idx as u16,
            View::Quests => self.quests_idx as u16,
            View::Quest => 0 
        }
    }

    fn render(&self, page: &mut Screen) {
        let pnt = page.rect.top_left_padded();
        render_header(&mut page.next_buf, pnt.add(0, 0));
        render_divider(&mut page.next_buf, pnt.add(0, 3), page.rect.width().into());

        match self.view {
            View::Lessons => {
                render_lessons(&mut page.next_buf, pnt.add(0, 4), &self.lessons);
            },
            View::Quests => {
                render_quests(&mut page.next_buf, pnt.add(0, 4), &self.quests);
            },
            View::Quest => {
                render_quest(&mut page.next_buf, pnt.add(0, 4), &self.quest);
            }
        }

        render_selection_caret(&mut page.next_buf, pnt.add(0, 4), self.get_selection_idx()); 
        let _ = page.stdout.queue(cursor::Hide);
    }
}



// TODO: for the help stuff
//
// let col_num = 32;
// qc.queue(cursor::MoveTo(0, self.help_line_count))?;
// qc.queue(style::Print("Help"))?;
//
// self.help_line_count += 1; 
//
// qc.queue(cursor::MoveTo(0, self.help_line_count))?;
// qc.queue(style::Print("To quite the program:"))?;
// qc.queue(cursor::MoveTo(col_num, self.help_line_count))?;
// qc.queue(style::Print("CTRL + c"))?;
// self.help_line_count += 1; 
//
// qc.queue(cursor::MoveTo(0, self.help_line_count))?;
// qc.queue(style::Print("Up:"))?;
// qc.queue(cursor::MoveTo(col_num, self.help_line_count))?;
// qc.queue(style::Print("CTRL + p"))?;
// self.help_line_count += 1; 
//
// qc.queue(cursor::MoveTo(0, self.help_line_count))?;
// qc.queue(style::Print("Down:"))?;
// qc.queue(cursor::MoveTo(col_num, self.help_line_count))?;
// qc.queue(style::Print("CTRL + n"))?;
// self.help_line_count += 1; 
//
// qc.queue(cursor::MoveTo(0, self.help_line_count))?;
// qc.queue(style::Print("Select line:"))?;
// qc.queue(cursor::MoveTo(col_num, self.help_line_count))?;
// qc.queue(style::Print("enter"))?;
// self.help_line_count += 1; 
//
// for col in 0..cols {
//     qc.queue(cursor::MoveTo(col, self.help_line_count))?;
//     qc.queue(style::Print("-"))?;
// }
// self.help_line_count += 1; 
// qc.flush()?;


/// Header 
/// Your available courses:
///
///
///
fn render_header(buf: &mut ScreenBuf, point: Point) {
    let text = "Your available courses"
        .chars()
        .map(|ch| Cell::new(ch, Color::White))
        .collect();

    buf.put_cells(point, text);
}

/// Divider
/// ----------------------------------------------------
fn render_divider(buf: &mut ScreenBuf, point: Point, screen_width: usize) {
    let cells = vec![Cell::new('-', Color::White); screen_width];
    buf.put_cells(point, cells);
}

/// Selection caret
/// > 
fn render_selection_caret(buf: &mut ScreenBuf, point: Point, idx: u16) {
    let pnt = point.add(0, idx);
    buf.put_cell(pnt, Cell::new('>', Color::White));
}

/// Lessons
///   name
///   name
///   name
fn render_lessons(buf: &mut ScreenBuf, point: Point, lessons: &Vec<models::Lesson>) {
    for (offset, lesson) in lessons.iter().enumerate() {
        let point = point.add(3, offset as u16);
        let cells = lesson.cmd.chars().map(|ch| Cell::new(ch, Color::White)).collect();
        buf.put_cells(point, cells);
    }
}

/// quests
///   ..
///   name
///   name
fn render_quests(buf: &mut ScreenBuf, point: Point, quests: &Vec<models::Quest>) {
    let cells = "..".chars().map(|ch| Cell::new(ch, Color::White)).collect();
    buf.put_cells(point.add(3, 0), cells);

    for (offset, quest) in quests.iter().enumerate() {
        let point = point.add(3, (offset + 1) as u16);
        let cells = quest.pattern.chars().map(|ch| Cell::new(ch, Color::White)).collect();
        buf.put_cells(point, cells);
    }
}

fn render_quest(buf: &mut ScreenBuf, point: Point, quest: &Option<models::Quest>) {
    let cells = "..".chars().map(|ch| Cell::new(ch, Color::White)).collect();
    buf.put_cells(point.add(3, 0), cells);

    if let Some(quest) = quest {
        let cmd = quest.cmd.chars().map(|ch| Cell::new(ch, Color::White)).collect();
        buf.put_cells(point.add(0, 2), cmd);
    } else {
        let text = "Not found id"
            .chars()
            .map(|ch| Cell::new(ch, Color::White))
            .collect();

        buf.put_cells(point.add(0, 2), text);
    }
}

struct Screen {
    stdout: std::io::Stdout,
    quit: bool,
    curr_buf: ScreenBuf, 
    next_buf: ScreenBuf,
    width: u16,
    height: u16,
    rect: Rect,
}

impl Screen {
    pub fn start() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        let (width, height) = terminal::size()?;
        let mut rect = Rect::default();
        rect.set_dimensions(width, height);

        Ok(Self {
            stdout,
            quit: false,
            curr_buf: ScreenBuf::new(width, height),
            next_buf: ScreenBuf::new(width, height),
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

       for Patch{ cell: Cell{ ch, fg, bg }, x, y } in diff {
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

    info!("Start explore command");

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
                        KeyCode::Enter => state.select(conn),
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
