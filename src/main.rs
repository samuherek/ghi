mod store;
mod debug;
mod lexer;
mod parser;
mod input_lexer;
mod compare;

use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;
use crossterm::{execute, style, cursor};
use crossterm::terminal::{self, EnterAlternateScreen, Clear, ClearType, LeaveAlternateScreen};
use std::io::{self, stdout, Write, Read};
use std::path::PathBuf;
use std::{fs, env};
use crossterm::QueueableCommand;
use crossterm::event::{self, KeyCode, KeyModifiers, Event};
use store::Store;
use tempfile::NamedTempFile;
use std::process::Command;
use serde::Deserialize;
use serde_json;
use parser::CmdParser;

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
    Add { value: Option<String> },
    List, 
    Flash,
    Tmux,
    Test
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
        match self.view {
            View::History => {
                self.results = store.get_history_refs(&self.query, limit);
            },
            View::List => {
                self.results = store.get_refs(&self.query, limit);
            }
        }
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


#[derive(Deserialize)]
struct Cmd {
    cmd: String, 
    description: String,
    tag: String
}

struct ScreenTmux {
    input: String,
    quest: String,
    answer: String,
    text: Vec<String>,
    quit: bool,
}

impl ScreenTmux {
    fn enable() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(Self {
            input: String::new(),
            quest: String::new(),
            answer: String::new(),
            text: Vec::new(),
            quit: false,
        })
    }

    pub fn append_input(&mut self, x: char) {
        self.input.push(x);
    }

    pub fn backspace_input(&mut self) {
        self.input.pop();
    }

    fn reset(&self, qc: &mut impl Write) -> io::Result<()> {
        qc.queue(Clear(ClearType::All))?;
        qc.queue(cursor::MoveTo(0, 0))?;
        qc.queue(cursor::Hide)?;

        qc.flush()?;

        Ok(())
    }
}

impl Drop for ScreenTmux {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode: {err}")
        });
        let _ = execute!(stdout(), LeaveAlternateScreen).map_err(|err| {
            eprintln!("ERROR: leave alternate screen: {err}")
        });
    }
}

#[derive(Debug)]
enum Arg {
    String(String),
    Int(usize)
}

#[derive(Debug)]
enum Token { 
    Command(String),
    Flag(String),
    Argument(Arg),
}

// [] optional
// <> required
// ... previous element can repeat

fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let parts = input.split_whitespace().collect::<Vec<_>>();

    for part in parts {
        if part.starts_with('[') {
            

        } else if part.starts_with('-') {
            tokens.push(Token::Flag(part.to_string()));
        } else if part.starts_with('<') && part.ends_with('>') {
            let val = &part[1..part.len().saturating_sub(1)];
            let num = val.parse::<usize>().ok();

            if let Some(num) = num {
                tokens.push(Token::Argument(Arg::Int(num)));
            } else {
                tokens.push(Token::Argument(Arg::String(val.to_string())));
            }
        } else {
            tokens.push(Token::Command(part.to_string()));
        }
    }

    return tokens;
}

fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add{value}) => {
            let mut store = Store::new();
            store.init_database()?;
            let mut input = String::new();

            if let Some(value) = value {
                store.create_from_string(value)?;
                input = value.clone();
            } else {
                let mut buf = String::new();
                match io::stdin().read_to_string(&mut buf) {
                    Ok(_) => {
                        store.create_from_string(&buf)?;
                        input = buf;
                    }, 
                    Err(err) => eprintln!("Error reading stdion: {}", err)
                };
            };

            let file = NamedTempFile::new()?;
            let path = file.path();

            let path2 = file.path().to_str().unwrap().to_string();
            println!("path, {:?}", path);
            println!("path2, {:?}", path2);

            let editor = env::var("EDITOR")?;

            Command::new(editor)
                .args(path)
                .status()?;

            let i = fs::read_to_string(path)?;
            println!("User input: {}", i);

            println!("added: {:?}", input);
        },
        Some(Commands::List) => {
            let mut store = Store::new();
            store.init_database()?;
            for item in store.db_take(None) {
                println!("{}", item);
            };
        },
        Some(Commands::Test) => {
            for item in vec![
                "some cmd --depth -f",
                "some cmd -la",
                "some <path> <path>",
            ] {
                println!("{}", item);
                let res = CmdParser::compile(item).iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ");
                println!("{res}");
            }
        },
        Some(Commands::Tmux) => {
            let data = fs::read_to_string(PathBuf::from("tmux.json")).unwrap();
            let course: Vec<Cmd> = serde_json::from_str(&data)?;
            let quest = course.into_iter().nth(0).unwrap();

            println!("{}", quest.cmd);
            let l = lex(&quest.cmd);
            println!("tokens:: {:?}", l);

            // let mut screen = ScreenTmux::enable()?;
            // let mut stdout = stdout();
            // let (screen_cols, screen_rows) = terminal::size()?;
            //
            // screen.quest = quest.description;

            

            // while !screen.quit {
            //     let _ = screen.reset(&mut stdout)?;
            //
            //     stdout.queue(cursor::MoveTo(0, 0))?;
            //     stdout.queue(style::Print(&screen.quest))?;
            //
            //     for (i, item) in screen.text.iter().enumerate() {
            //         stdout.queue(cursor::MoveTo(0, i as u16 + 2))?;
            //         stdout.queue(style::Print(item))?;
            //     }
            //
            //     for col in 0..screen_cols {
            //         stdout.queue(cursor::MoveTo(col, screen_rows - 2))?;
            //         stdout.queue(style::Print("-"))?;
            //     }
            //
            //     stdout.queue(cursor::MoveTo(0, screen_rows - 1))?;
            //     stdout.queue(style::Print(format!("{}", screen.input)))?;
            //
            //     stdout.flush()?;
            //
            //     if let Event::Key(event) = event::read()? {
            //         match event.code {
            //             KeyCode::Char(x) => {
            //                 if event.modifiers.contains(KeyModifiers::CONTROL) {
            //                     match x {
            //                         'c' => screen.quit = true,
            //                         _ => {}
            //                     }
            //                 } else {
            //                     screen.append_input(x);
            //                 }
            //             },
            //             KeyCode::Backspace => {
            //                 screen.backspace_input();
            //             },
            //             KeyCode::Enter => {
            //                 screen.text.push("submitted".to_string());
            //                 
            //                 let (cmd, args) = screen.input.split_once(" ").unwrap_or(("", ""));
            //                 if cmd.len() > 0 {
            //                     screen.text.push(cmd.to_string());
            //                     screen.text.push(args.to_string());
            //                 }
            //                     
            //                 //{
            //                 //        "command": "new-session -s <string>",
            //                 //        "description": "Create a new session.",
            //                 //        "tag": "session"
            //                 //    },
            //             },
            //             _ => {}
            //         }
            //     }
            // }
        },
        Some(Commands::Flash) => {
            let mut store = Store::new();
            store.init_database()?;

            let list = store.db_take(None);
            if let Some(val) = list.choose(&mut rand::thread_rng()) {
                println!("{val}");
            }  else {
                eprintln!("The list is empty");
            }

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
                                            'l' => {
                                                screen.set_view(View::List);
                                                screen.search(&store, visible_rows);
                                            }
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
                                    if let Some(id) = screen.get_selected() {
                                        store.create(*id)?;
                                    }
                                },
                                _ => {}
                            }
                        }
                    },
                    View::List => {
                        for (idx, item) in screen.results.iter().enumerate() {
                            if let Some(item) = store.get_item(*item) {
                                let next_row = screen.help_line_count + idx as u16 + 1;
                                let is_curr = idx == screen.selected_idx;
                                let arrow = if is_curr {
                                    "> "
                                } else {
                                    "  "
                                };

                                if is_curr {
                                    stdout.queue(style::SetForegroundColor(style::Color::Green))?;
                                } 
                                stdout.queue(style::Print(format!("{}{}", arrow, item)))?;
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
                                            's' => {
                                                screen.set_view(View::History);
                                                screen.search(&store, visible_rows);
                                            },
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
                                    if let Some(id) = screen.get_selected() {
                                        store.delete(*id)?;
                                        screen.selected_idx = screen.selected_idx.saturating_sub(1);
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
