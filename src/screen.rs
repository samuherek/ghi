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
    x: usize,
    y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y
        }
    }

    fn buf_addr(&self, screen_width: usize) -> usize {
        self.y * screen_width + self.x 
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
    width: usize,
    height: usize
}

impl ScreenBuf {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell::default(); width * height];
        Self {
            cells,
            width,
            height
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.cells.resize(width*height, Cell::default());
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
                let x = i % self.width;
                let y = i / self.width;
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
        let mut curr_bg = style::Color::Black;
        out.queue(Clear(ClearType::All))?;
        out.queue(style::SetForegroundColor(curr_fg))?;
        out.queue(style::SetBackgroundColor(curr_bg))?;
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


