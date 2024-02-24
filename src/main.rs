mod parser;
mod commands;
mod window;
mod db;

use clap::{Parser, Subcommand};
use simplelog::{WriteLogger, LevelFilter, Config};
use log::info;
use dirs;
use std::path::PathBuf;
use serde::Deserialize;

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
    /// Add a value to a TODO to revisit later
    Bucket { value: Option<String>},
    /// Start an explorer to see all the commands 
    Explore,
    /// Start the flashcard game
    Run
}

struct GhiConfig {
    // - $HOME/.ghi
    config_dir: PathBuf,
    // - $HOME/.ghirc
    // - $HOME/.ghi/config
    // - $HOME/.config/ghi/config
    database_path: Option<String>,
}

#[derive(Deserialize)]
struct GhiUserConfig {
   database_path: Option<String>,
}

const CONFIG_PATHS: [&str; 2] = [".ghi/config.toml", ".config/ghi/config.toml"];

fn get_config() -> GhiConfig {
    let home = dirs::home_dir().expect("Could not resolve home directory");
    let first_config_path = CONFIG_PATHS.iter()
        .map(|path| home.join(path))
        .find(|path| path.exists());

    let config = if let Some(first_config_path) = first_config_path {
        let data = std::fs::read_to_string(first_config_path).expect("The config path to exist");
        let user_config: GhiUserConfig = toml::from_str(&data).expect("Toml config could not parse");
        GhiConfig {
            config_dir: home.join(".ghi"),
            database_path: user_config.database_path
        }
    } else {
        GhiConfig {
            config_dir: home.join(".ghi"),
            database_path: Some(home.join(".ghi/database.sql").display().to_string())
        }
    };

    config 
}

fn main() -> anyhow::Result<()>{
    let config = get_config();
    let log_file = std::fs::File::create(config.config_dir.join("logs.txt")).expect("Could not create the log file");
    WriteLogger::init(LevelFilter::max(), Config::default(), log_file).unwrap();
    

    info!("Application started");

    let cli = Cli::parse();
    let mut conn = db::establish_connection();
    db::ensure_tables(&mut conn);

    match &cli.command {
        Some(Commands::Add{value}) => commands::add::run(&mut conn, value)?,
        Some(Commands::Bucket{value}) => commands::bucket::run(&mut conn, value)?,
        Some(Commands::Explore) => commands::explore::run(&mut conn)?,
        Some(Commands::Run) => commands::run::run()?,
        None => {
            unimplemented!();
        }
    }

    return Ok(());
}
