use crossterm::cursor;
use crossterm::style;
use crossterm::terminal::{Clear, ClearType};
use std::io::{self, Write};
use crossterm::QueueableCommand;
use super::rect::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: style::Color,
    pub bg: Option<style::Color>
}

impl Cell {
    pub fn new(ch: char, fg: style::Color) -> Self {
        Self { ch, fg, bg: None }
    }

    pub fn set_bg(&mut self, bg: style::Color) -> Self {
        self.bg = Some(bg);
        *self
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: style::Color::White,
            bg: None
        }
    }
}

#[derive(Debug)]
pub struct Patch {
    pub cell: Cell,
    pub x: usize,
    pub y: usize
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
        let mut curr_bg: Option<style::Color> = None;
        out.queue(Clear(ClearType::All))?;
        out.queue(style::SetForegroundColor(curr_fg))?;
        if let Some(curr_bg) = curr_bg {
            out.queue(style::SetBackgroundColor(curr_bg))?;
        } else {
            out.queue(style::ResetColor)?;
        }
        out.queue(cursor::MoveTo(0, 0))?;
        for Cell{ch, fg, bg} in self.cells.iter() {
            if curr_bg != *bg {
                curr_bg = *bg;
                if let Some(curr_bg) = curr_bg {
                    out.queue(style::SetBackgroundColor(curr_bg))?;
                } else {
                    out.queue(style::ResetColor)?;
                }
            }
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

pub fn apply_patches(out: &mut impl QueueableCommand, diff: &Vec<Patch>) -> io::Result<()> {
    for Patch{ cell: Cell{ ch, fg, bg }, x, y } in diff {
        out.queue(cursor::MoveTo(*x as u16, *y as u16))?;
        if let Some(bg) = *bg {
            out.queue(style::SetBackgroundColor(bg))?;
        } else {
            out.queue(style::ResetColor)?;
        }
        out.queue(style::SetForegroundColor(*fg))?;
        out.queue(style::Print(ch))?;
    }
    Ok(())
}


