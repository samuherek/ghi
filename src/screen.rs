use crossterm::{execute, style, cursor};
use crossterm::terminal::{self, Clear, ClearType};
use std::io::{self, stdout, Write};
use crossterm::QueueableCommand;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cell {
    ch: char,
    fg: style::Color,
    bg: style::Color
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: style::Color::White,
            bg: style::Color::Black,
        }
    }
}

struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn buf_addr(&self, screen_width: usize) -> usize {
        self.y * screen_width + self.x 
    }
}

struct Patch {
    cell: Cell,
    x: usize,
    y: usize
}

#[derive(Debug)]
pub struct ScreenBuf {
    cells: Vec<Cell>,
    width: usize,
    height: usize
}

impl ScreenBuf {
    fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell::default(); width * height];
        Self {
            cells,
            width,
            height
        }
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.cells.resize(width*height, Cell::default());
        self.cells.fill(Cell::default());
        self.width = width;
        self.height = height;
    }

    fn diff(&self, next: &Self) -> Vec<Patch> {
        assert!(self.width == next.width && self.height == next.height);
        self.cells.iter()
            .zip(next.cells.iter())
            .enumerate()
            .filter(|(_, (a, b))| *a != *b)
            .map(|(i, (_, cell))| {
                let x = i % self.width;
                let y = i / self.width;
                Patch{cell: *cell, x, y}
            })
            .collect()
    }

    fn put_cell(&mut self, point: Point, cell: Cell) {
        if let Some(buf_cell) = self.cells.get_mut(point.buf_addr(self.width)) {
            *buf_cell = cell;
        }
    }

    fn put_cells(&mut self, point: Point, cells: Vec<Cell>) {
        let start = point.buf_addr(self.width);
        for (offest, &cell) in cells.iter().enumerate() {
            if let Some(buf_cell) = self.cells.get_mut(start + offest) {
                *buf_cell = cell;
            } else {
                break;
            }
        } 
    }

    fn flush(&self, out: &mut impl Write) -> io::Result<()> {
        let mut curr_fg = style::Color::White;
        let mut curr_bg = style::Color::Black;
        out.queue(Clear(ClearType::All))?;
        out.queue(style::SetForegroundColor(curr_fg))?;
        out.queue(style::SetBackgroundColor(curr_bg))?;
        out.queue(cursor::MoveTo(0, 0))?;
        for Cell{ch, fg, bg} in self.cells.iter() {
            if curr_fg != *fg {
                curr_fg = *fg;
                out.queue(style::SetForegroundColor(curr_fg))?;
            }
            if curr_bg != *bg {
                curr_bg = *bg;
                out.queue(style::SetForegroundColor(curr_bg))?;
            }
            out.queue(style::Print(ch))?;
        }
        out.flush()?;
        Ok(())
    }
}

pub struct Screen {
    pub quit: bool
}

impl Screen {
    pub fn start() -> io::Result<Self> {
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        Ok(Self {
            quit: false,
        })
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode: {err}")
        });
        let _ = execute!(stdout(), terminal::LeaveAlternateScreen).map_err(|err| {
            eprintln!("ERROR: leave alternate screen: {err}")
        });
    }
}


