use diesel::SqliteConnection;
use crossterm::{execute, terminal, QueueableCommand};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::{Color};
use crate::screen;
use std::io::Write;

struct Screen {
    stdout: std::io::Stdout,
    quit: bool,
    curr_buf: screen::ScreenBuf, 
    next_buf: screen::ScreenBuf,
    width: u16,
    height: u16,
    rect: screen::Rect
}

impl Screen {
    pub fn start() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        let (width, height) = terminal::size()?;

        Ok(Self {
            stdout,
            quit: false,
            curr_buf: screen::ScreenBuf::new(width, height),
            next_buf: screen::ScreenBuf::new(width, height),
            width,
            height,
            rect: screen::Rect::default()
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
   let mut page = Screen::start()?;


   let text = "We are in business...".chars().map(|ch| {
       screen::Cell::new(ch, Color::Black).set_bg(Color::White)
   }).collect();
   page.next_buf.put_cells(page.rect.top_left_padded(), text);

    while !page.quit {
        while poll(std::time::Duration::ZERO)? {
            match read()? {
                Event::Resize(next_width, next_height) => {
                    page.resize(next_width, next_height)?;
                },
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    match event.code {
                        KeyCode::Char(x) => {
                            if event.modifiers.contains(KeyModifiers::CONTROL) && x == 'c' {
                                page.quit()
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        page.next_buf.clear();

        let text = "We are in business...".chars().map(|ch| {
            screen::Cell::new(ch, Color::White)
        }).collect();
        page.next_buf.put_cells(page.rect.top_left_padded().add(0, 0), text);

        page.apply_patches()?;
        page.stdout.flush()?;
        page.swap_buffers();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

   Ok(())
}
