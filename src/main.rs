mod renderer;
mod history;
mod config;
mod store;

use crate::history::{MoveDirection, History};
use clap::{Parser, Subcommand};
use crossterm::{execute, style, cursor};
use crossterm::terminal::{self, EnterAlternateScreen, Clear, ClearType, LeaveAlternateScreen};
use std::io::{self, stdout, Write};
use crossterm::QueueableCommand;
use crossterm::event::{self, KeyCode, KeyModifiers, Event};
use store::Store;

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
    Search
}

struct ScreenState {
    view: View,
    help_line_count: u16,
    pub quit: bool
}

impl ScreenState {
    fn enable() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(Self {
            view: View::Search,
            help_line_count: 0,
            quit: false
        })
    }

    fn render_help(&mut self, qc: &mut impl Write) -> io::Result<()> {
        let (cols, _) = terminal::size()?;

        self.help_line_count = 0;

        let col_num = 32;
        qc.queue(cursor::MoveTo(0, self.help_line_count))?;
        qc.queue(style::Print("Help"))?;
        
        self.help_line_count += 1; 

        qc.queue(cursor::MoveTo(0, self.help_line_count))?;
        qc.queue(style::Print("To quite the program:"))?;
        qc.queue(cursor::MoveTo(col_num, self.help_line_count))?;
        qc.queue(style::Print("CTRL + c"))?;
        self.help_line_count += 1; 

        qc.queue(cursor::MoveTo(0, self.help_line_count))?;
        qc.queue(style::Print("Up:"))?;
        qc.queue(cursor::MoveTo(col_num, self.help_line_count))?;
        qc.queue(style::Print("CTRL + p"))?;
        self.help_line_count += 1; 

        qc.queue(cursor::MoveTo(0, self.help_line_count))?;
        qc.queue(style::Print("Down:"))?;
        qc.queue(cursor::MoveTo(col_num, self.help_line_count))?;
        qc.queue(style::Print("CTRL + n"))?;
        self.help_line_count += 1; 

        qc.queue(cursor::MoveTo(0, self.help_line_count))?;
        qc.queue(style::Print("Select line:"))?;
        qc.queue(cursor::MoveTo(col_num, self.help_line_count))?;
        qc.queue(style::Print("enter"))?;
        self.help_line_count += 1; 

        for col in 0..cols {
            qc.queue(cursor::MoveTo(col, self.help_line_count))?;
            qc.queue(style::Print("-"))?;
        }
        self.help_line_count += 1; 
        qc.flush()?;

        Ok(())
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
            let store = Store::init()?;
            let mut history = History::new()?;
            let mut screen = ScreenState::enable()?;
            let mut stdout = stdout();

            history.init_search((terminal::size()?.1 - 15).into());

            while !screen.quit {
                let _ = screen.reset(&mut stdout)?;
                let _ = screen.render_help(&mut stdout)?;
                let (screen_cols, screen_rows) = terminal::size()?;
                let search_rows = 2;

                stdout.queue(cursor::MoveTo(0, screen.help_line_count))?;

                let visible_rows = (screen_rows - screen.help_line_count - search_rows).into();
                let selected_idx = history.selected_idx;

                for (idx, item) in history.search(visible_rows).iter().enumerate() {
                    let next_row = screen.help_line_count + idx as u16 + 1;
                    let arrow = if idx == selected_idx {
                        ">  "
                    } else {
                        "   "
                    };
                    stdout.queue(style::Print(format!("{}{}", arrow, item)))?;
                    stdout.queue(cursor::MoveTo(0, next_row))?;
                }

                for col in 0..screen_cols {
                    stdout.queue(cursor::MoveTo(col, screen_rows - search_rows))?;
                    stdout.queue(style::Print("-"))?;
                }

                stdout.queue(cursor::MoveTo(0, screen_rows - 1))?;
                stdout.queue(style::Print(format!("{}", history.query)))?;

                stdout.flush()?;

                if let Event::Key(event) = event::read()? {
                    match event.code {
                        KeyCode::Char(x) => {
                            if event.modifiers.contains(KeyModifiers::CONTROL) {
                                match x {
                                    'c' => screen.quit = true,
                                    'n' => history.move_selected_index(MoveDirection::Down),
                                    'p' => history.move_selected_index(MoveDirection::Up),
                                    _ => {}
                                }
                            } else {
                                history.append_query(x);
                            }
                        },
                        KeyCode::Backspace => {
                            history.backspace_query();
                        },
                        KeyCode::Enter => {
                            if let Some(command) = history.get_selection() {
                                store.create(command)?;
                                screen.quit = true;
                            }
                        },
                        _ => {}
                    }
                }

                // TODO: is this necessary? It makes sense but at the same time
                // when I navigate through the queries, it flickers. 
                //thread::sleep(Duration::from_millis(16));
            }
        }
    }

    return Ok(());
}
