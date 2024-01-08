mod renderer;
mod history;
mod config;

use crate::history::History;
use clap::{Parser, Subcommand};
use crossterm::{execute, style, cursor};
use crossterm::terminal::{self, EnterAlternateScreen, Clear, ClearType, LeaveAlternateScreen};
use std::io::{self, stdout, Write};
use std::thread;
use std::time::Duration;
use crossterm::QueueableCommand;
use crossterm::event::{self, KeyCode, KeyEvent, Event};

#[derive(Parser)]
#[command(author = "Sam Uherek <samuherekbiz@gmail.com>")]
#[command(about = "Quick reference to commands", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Add,
    List
}

enum View {
    List
}

struct ScreenState {
    view: View,
}

impl ScreenState {
    fn enable() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(Self {
            view: View::List
        })
    }

    fn render_help(&self, qc: &mut impl Write) -> io::Result<u16> {
        let (cols, _) = terminal::size()?;
        let mut line_num = 0;
        let col_num = 32;
        qc.queue(cursor::MoveTo(0, line_num))?;
        qc.queue(style::Print("Help"))?;
        line_num += 1; 

        qc.queue(cursor::MoveTo(0, line_num))?;
        qc.queue(style::Print("To quite the program:"))?;
        qc.queue(cursor::MoveTo(col_num, line_num))?;
        qc.queue(style::Print("q"))?;
        line_num += 1; 

        qc.queue(cursor::MoveTo(0, line_num))?;
        qc.queue(style::Print("To search the history:"))?;
        qc.queue(cursor::MoveTo(col_num, line_num))?;
        qc.queue(style::Print("/"))?;
        line_num += 1; 

        qc.queue(cursor::MoveTo(0, line_num))?;
        qc.queue(style::Print("Up:"))?;
        qc.queue(cursor::MoveTo(col_num, line_num))?;
        qc.queue(style::Print("k"))?;
        line_num += 1; 

        qc.queue(cursor::MoveTo(0, line_num))?;
        qc.queue(style::Print("Down:"))?;
        qc.queue(cursor::MoveTo(col_num, line_num))?;
        qc.queue(style::Print("j"))?;
        line_num += 1; 

        for col in 0..cols {
            qc.queue(cursor::MoveTo(col, line_num))?;
            qc.queue(style::Print("-"))?;
        }
        line_num += 1; 
        qc.flush()?;

        return Ok(line_num);
    }

    fn reset(&self, qc: &mut impl Write) -> io::Result<()> {
        qc.queue(Clear(ClearType::All))?;
        qc.queue(cursor::MoveTo(0, 0))?;
        qc.queue(cursor::Hide)?;

        qc.flush()?;

        Ok(())
    }
}

impl Drop for ScreenState {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode: {err}")
        });
        let _ = execute!(stdout(), LeaveAlternateScreen).map_err(|err| {
            eprintln!("ERROR: leave alternate screen: {err}")
        });
    }
}

fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add) => {
            todo!();
        },
        Some(Commands::List) => {
            todo!();
        },
        None => {
            let history = History::default();
            let screen = ScreenState::enable()?;
            let mut stdout = stdout();

            while !history.quit {
                let _ = screen.reset(&mut stdout)?;
                let line_num = screen.render_help(&mut stdout)?;

                match screen.view {
                    View::List => {
                        stdout.queue(cursor::MoveTo(0, line_num))?;
                        stdout.queue(style::Print("working"))?;
                        stdout.flush()?;
                    }
                }

                if let Event::Key(KeyEvent { code,  .. }) = event::read()? {
                    match code {
                        KeyCode::Char('q') => {
                            break;
                        },
                        _ => {}
                    }
                }

                stdout.flush()?;
                thread::sleep(Duration::from_millis(16));
            }
        }
    }

    return Ok(());
}
