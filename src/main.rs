mod store;

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

#[derive(PartialEq)]
enum View {
    History,
    List
}

enum MoveDirection {
    Up,
    Down
}

struct ScreenState {
    view: View,
    help_line_count: u16,
    search_rows_count: u16,
    query: String,
    results: Vec<usize>,
    selected_idx: usize,
    pub quit: bool
}

impl ScreenState {
    fn enable() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(Self {
            view: View::History,
            query: String::from(""),
            results: Vec::new(),
            selected_idx: 0,
            help_line_count: 0,
            search_rows_count: 2,
            quit: false
        })
    }

    pub fn move_selected_index(&mut self, dir: MoveDirection) {
        if self.results.len() == 0 {
            return;
        }

        match dir {
            MoveDirection::Up => {
                self.selected_idx = self.selected_idx.saturating_sub(1);
            },
            MoveDirection::Down => {
                self.selected_idx = (self.selected_idx + 1).min(self.results.len() - 1);
            }
        }
    }

    pub fn set_view(&mut self, view: View) {
        if self.view != view {
            self.query.clear();
            self.results.clear();
            self.selected_idx = 0;
            self.view = view;
        }
    }
   
    pub fn append_query(&mut self, x: char) {
        self.selected_idx = 0;
        self.query.push(x);
    }

    pub fn backspace_query(&mut self) {
        self.selected_idx = 0;
        self.query.pop();
    }

    pub fn get_selected(&self) -> Option<&usize> {
        self.results.get(self.selected_idx) 
    }

    fn search(&mut self, store: &Store, limit: usize) {
        self.results = store.get_history_refs(&self.query, limit);
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
            let mut store = Store::new();
            let _ = store.init();
            let mut screen = ScreenState::enable()?;
            let mut stdout = stdout();
            let (screen_cols, screen_rows) = terminal::size()?;

            screen.search(&store, screen_rows.saturating_sub(15).into());

            while !screen.quit {
                let _ = screen.reset(&mut stdout)?;
                let _ = screen.render_help(&mut stdout)?;

                stdout.queue(cursor::MoveTo(0, screen.help_line_count))?;
                let visible_rows: usize = (screen_rows - screen.help_line_count - screen.search_rows_count).into();

                match screen.view {
                    View::History => {
                        for (idx, item) in screen.results.iter().enumerate() {
                            if let Some(item) = store.get_history_item(*item) {
                                let next_row = screen.help_line_count + idx as u16 + 1;
                                let is_curr = idx == screen.selected_idx;
                                let arrow = if is_curr {
                                    "> "
                                } else {
                                    "  "
                                };

                                let selected_dot = if item.selected {
                                    "o "
                                } else {
                                    "  " 
                                };

                                if is_curr {
                                    stdout.queue(style::SetForegroundColor(style::Color::Green))?;
                                } else if item.selected {
                                    stdout.queue(style::SetForegroundColor(style::Color::DarkGreen))?;
                                };
                                stdout.queue(style::Print(format!("{}{}{}", selected_dot, arrow, item.value)))?;
                                stdout.queue(style::SetForegroundColor(style::Color::Reset))?;
                                stdout.queue(cursor::MoveTo(0, next_row))?;
                            }
                        }

                        for col in 0..screen_cols {
                            stdout.queue(cursor::MoveTo(col, screen_rows - screen.search_rows_count))?;
                            stdout.queue(style::Print("-"))?;
                        }

                        stdout.queue(cursor::MoveTo(0, screen_rows - 1))?;
                        stdout.queue(style::Print(format!("{}", screen.query)))?;

                        stdout.flush()?;

                        if let Event::Key(event) = event::read()? {
                            match event.code {
                                KeyCode::Char(x) => {
                                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                                        match x {
                                            'c' => screen.quit = true,
                                            'n' => screen.move_selected_index(MoveDirection::Down),
                                            'p' => screen.move_selected_index(MoveDirection::Up),
                                            'l' => screen.set_view(View::List),
                                            _ => {}
                                        }
                                    } else {
                                        screen.append_query(x);
                                        screen.search(&store, visible_rows);
                                    }
                                },
                                KeyCode::Backspace => {
                                    screen.backspace_query();
                                    screen.search(&store, visible_rows);
                                },
                                KeyCode::Enter => {
                                    let item = screen.get_selected().and_then(|x| store.get_history_item(*x));
                                    if let Some(command) = item {
                                        if !command.selected {
                                            //store.create(&command.value)?;
                                            //history.add();
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    },
                    View::List => {
                        for (idx, item) in store.all().iter().enumerate() {
                            let next_row = screen.help_line_count + idx as u16 + 1;

                            stdout.queue(style::Print(format!("{}", item)))?;
                            stdout.queue(cursor::MoveTo(0, next_row))?;
                        }

                        stdout.flush()?;

                        if let Event::Key(event) = event::read()? {
                            match event.code {
                                KeyCode::Char(x) => {
                                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                                        match x {
                                            'c' => screen.quit = true,
                                            //'n' => history.move_selected_index(MoveDirection::Down),
                                            //'p' => history.move_selected_index(MoveDirection::Up),
                                            's' => {
                                                screen.set_view(View::History);
                                            },
                                            _ => {}
                                        }
                                    } 
                                },
                                _ => {}
                            }
                        }
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
