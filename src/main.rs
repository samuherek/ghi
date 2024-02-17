mod parser;
mod commands;
mod window;
mod db;

use clap::{Parser, Subcommand};
use simplelog::{WriteLogger, LevelFilter, Config};
use log::info;

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
    let log_file = std::fs::File::create("log.txt").expect("Could not create the log file");
    WriteLogger::init(LevelFilter::max(), Config::default(), log_file).unwrap();

    info!("Application started");

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
