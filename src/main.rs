mod parser;
mod commands;
mod screen;
mod db;

use clap::{Parser, Subcommand};

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
    /// Add a new command to the default list
    Add { value: Option<String> },
    /// Start an explorer to see all the commands 
    Explore,
    /// Start the flashcard game
    Run
}

fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();
    let mut conn = db::establish_connection();
    db::ensure_tables(&mut conn);

    match &cli.command {
        Some(Commands::Add{value}) => commands::add::run(&mut conn, value)?,
        Some(Commands::Explore) => commands::explore::run(&mut conn)?,
        Some(Commands::Run) => commands::run::run()?,
        None => {
            unimplemented!();
        }
    }

    return Ok(());
}
