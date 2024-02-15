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
    Add { value: Option<String> },
    Manage,
    Play
}

fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();
    let mut conn = db::establish_connection();
    db::ensure_tables(&mut conn);

    match &cli.command {
        Some(Commands::Add{value}) => commands::add::run(&mut conn, value)?,
        Some(Commands::Manage) => commands::manage::run(&mut conn)?,
        Some(Commands::Play) => commands::play::run()?,
        None => {
            unimplemented!();
        }
    }

    return Ok(());
}
