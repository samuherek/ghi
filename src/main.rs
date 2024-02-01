mod store;
mod debug;
mod lexer;
mod parser;
mod input_lexer;
mod compare;
mod commands;

use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;
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
    Add { value: Option<String> },
    List, 
    Flash,
    Tmux,
    Test
}

fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add{value}) => commands::add::run(value)?,
        Some(Commands::List) => commands::list::run()?,
        Some(Commands::Test) => commands::test::run(),
        Some(Commands::Tmux) => commands::tmux::run()?,
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
        None => commands::interaction::run()?,
    }

    return Ok(());
}
