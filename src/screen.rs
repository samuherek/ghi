use crossterm::{execute, cursor};
use crossterm::style::{self, SetForegroundColor, SetBackgroundColor};
use crossterm::terminal::{self, Clear, ClearType};
use std::io::{self, stdout, Write};
use crossterm::QueueableCommand;

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

#[derive(Clone)]
pub struct Point {
    pub x: u16,
    pub y: u16,
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

    pub fn set(&mut self, x: u16, y: u16) -> &Self {
        self.x += x;
        self.y += y;
        self
    }

    pub fn add(&self, x: u16, y: u16) -> Self {
        Point::new((self.x + x).into(), (self.y + y).into())
    }
}

struct RectPadd {
    top: u16,
    right: u16,
    bottom: u16, 
    left: u16,
}

impl Default for RectPadd {
    fn default() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }
}

impl ToString for RectPadd {
    fn to_string(&self) -> String {
        format!("{}, {}, {}, {}", self.top, self.right, self.bottom, self.left)
    }
}

pub struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16, 
    padding: RectPadd,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { 
            x, 
            y, 
            width, 
            height,
            padding: RectPadd::default(),
        } 
    }

    pub fn set_padding(
        &mut self, 
        top: Option<u16>, 
        right: Option<u16>, 
        bottom: Option<u16>, 
        left: Option<u16>
    ) {
        if let Some(top) = top {
            self.padding.top = top;
        }
        if let Some(right) = right {
            self.padding.right = right;
        }
        if let Some(bottom) = bottom {
            self.padding.bottom = bottom;
        }
        if let Some(left) = left {
            self.padding.left = left;
        }
    }

    pub fn anchor(&self) -> Point {
        Point::new(self.x.into(), self.y.into())
    }
    
    pub fn set_anchor(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub fn set_dimensions(&mut self, w: u16, h: u16) {
        self.width = w;
        self.height = h;
    }

    pub fn top_left(&self) -> Point {
        Point::new(self.x.into(), self.y.into())
    }

    pub fn top_left_padded(&self) -> Point {
        let x = (self.x + self.padding.left).into();
        let y = (self.y + self.padding.top).into();
        Point::new(x, y)
    }

    pub fn top_right(&self) -> Point {
        let x = self.x + self.width;
        Point::new(x.into(), self.y.into())
    }

    pub fn top_right_padded(&self) -> Point {
        let x = (self.width - self.padding.right).into();
        let y = (self.y + self.padding.top).into();
        Point::new(x, y)
    }

    pub fn bottom_left(&self) -> Point {
        let y = self.y + self.height;
        Point::new(self.x.into(), y.into())
    }

    pub fn bottom_left_padded(&self) -> Point {
        let x = (self.x + self.padding.left).into();
        let y = (self.y + self.height - self.padding.bottom).into();
        Point::new(x, y)
    }


    pub fn bottom_right(&self) -> Point {
        let x = self.x + self.width;
        let y = self.y + self.height;
        Point::new(x.into(), y.into())
    }
     
    pub fn width(&self) -> u16 {
        self.width 
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn point(&self, x: u16, y: u16) -> Point {
        let x = self.x + x;
        let y = self.y + y;
        Point::new(x.into(), y.into())
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            x: 0, 
            y: 0, 
            width: 0, 
            height: 0,
            padding: RectPadd::default(),
        }
    }
}

pub trait GhiDebug {
    fn debug(&self) -> Vec<String>;
}

impl GhiDebug for Rect {
    fn debug(&self) -> Vec<String> {
        vec![
            format!("x: {}", self.x),
            format!("y: {}", self.y),
            format!("width: {}", self.width),
            format!("height: {}", self.height),
            format!("padding: {}", self.padding.to_string()),
        ]
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

pub struct Screen {
    with_alternate_screen: bool,
    pub quit: bool,
}

impl Screen {
    pub fn start() -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        Ok(Self {
            quit: false,
            with_alternate_screen: false
        })
    }
    
    pub fn with_altenrate(&mut self) -> std::io::Result<&Self> {
        self.with_alternate_screen = true; 
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        Ok(self)
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

        if self.with_alternate_screen {
            let _ = execute!(stdout(), terminal::LeaveAlternateScreen).map_err(|err| {
                eprintln!("ERROR: leave alternate screen: {err}")
            });
        }
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


