use crossterm::{execute, cursor};
use crossterm::style::{self, SetForegroundColor, SetBackgroundColor};
use crossterm::terminal::{self, Clear, ClearType};
use std::io::{self, stdout, Write};
use crossterm::QueueableCommand;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    ch: char,
    fg: style::Color,
}

impl Cell {
    pub fn new(ch: char, fg: style::Color) -> Self {
        Self { ch, fg }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: style::Color::White,
        }
    }
}

pub struct Point {
    x: u16,
    y: u16,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x: x.try_into().expect("X point can not be more than u16 value"),
            y: y.try_into().expect("Y point can not be more than u16 value")
        }
    }

    pub fn buf_addr(&self, screen_width: u16) -> usize {
        (self.y * screen_width + self.x).into()
    }
}

pub struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16, 
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height } 
    }

    pub fn top_left(&self) -> Point {
        Point::new(self.x.into(), self.y.into())
    }

    pub fn point(&self, x: u16, y: u16) -> Point {
        let x = self.x + x;
        let y = self.y + y;
        Point::new(x.into(), y.into())
    }
}

#[derive(Debug)]
pub struct Patch {
    cell: Cell,
    x: usize,
    y: usize
}

#[derive(Debug)]
pub struct ScreenBuf {
    cells: Vec<Cell>,
    width: u16,
    height: u16
}

impl ScreenBuf {
    pub fn new(width: u16, height: u16) -> Self {
        let len = (width * height).into();
        let cells = vec![Cell::default(); len];
        Self {
            cells,
            width,
            height
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let len = (width*height).into();
        self.cells.resize(len, Cell::default());
        self.cells.fill(Cell::default());
        self.width = width;
        self.height = height;
    }

    pub fn diff(&self, next: &Self) -> Vec<Patch> {
        assert!(self.width == next.width && self.height == next.height);
        self.cells.iter()
            .zip(next.cells.iter())
            .enumerate()
            .filter(|(_, (a, b))| *a != *b)
            .map(|(i, (_, cell))| {
                let w: usize = self.width.into();
                let x = i % w;
                let y = i / w;
                Patch{cell: *cell, x, y}
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn put_cell(&mut self, point: Point, cell: Cell) {
        if let Some(buf_cell) = self.cells.get_mut(point.buf_addr(self.width)) {
            *buf_cell = cell;
        }
    }

    pub fn put_cells(&mut self, point: Point, cells: Vec<Cell>) {
        let start = point.buf_addr(self.width);
        for (offest, &cell) in cells.iter().enumerate() {
            if let Some(buf_cell) = self.cells.get_mut(start + offest) {
                *buf_cell = cell;
            } else {
                break;
            }
        } 
    }

    pub fn flush(&self, out: &mut impl Write) -> io::Result<()> {
        let mut curr_fg = style::Color::White;
        out.queue(Clear(ClearType::All))?;
        out.queue(style::SetForegroundColor(curr_fg))?;
        out.queue(cursor::MoveTo(0, 0))?;
        for Cell{ch, fg} in self.cells.iter() {
            if curr_fg != *fg {
                curr_fg = *fg;
                out.queue(style::SetForegroundColor(curr_fg))?;
            }
            out.queue(style::Print(ch))?;
        }
        out.flush()?;
        Ok(())
    }

    fn reset(&self, out: &mut impl Write) -> io::Result<()> {
        out.queue(Clear(ClearType::All))?;
        out.queue(cursor::MoveTo(0, 0))?;
        out.queue(cursor::Hide)?;
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
 

pub fn apply_patches(out: &mut impl QueueableCommand, diff: &Vec<Patch>) -> io::Result<()> {
    out.queue(SetForegroundColor(style::Color::White))?;

    for Patch{ cell: Cell{ ch, fg }, x, y } in diff {
        out.queue(cursor::MoveTo(*x as u16, *y as u16))?;
        out.queue(style::Print(ch))?;
    }
    Ok(())
}


