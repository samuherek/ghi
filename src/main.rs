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
    database_dir: PathBuf,
}

#[derive(Deserialize)]
struct GhiUserConfig {
   database_dir: Option<String>,
}

const CONFIG_PATHS: [&str; 2] = [".ghi/config.toml", ".config/ghi/config.toml"];

fn parse_database_dir(home: &PathBuf, value: &Option<String>) -> Option<PathBuf> {
    if let Some(val) = value {
        if val.starts_with("~") {
            let relative_path = &val[2..];
            return Some(home.join(relative_path));
        } else {
            return Some(PathBuf::from(val.to_string()));
        }
    }
    return None;
}

fn get_config() -> GhiConfig {
    let home = dirs::home_dir().expect("Could not resolve home directory");
    let first_config_path = CONFIG_PATHS.iter()
        .map(|path| home.join(path))
        .find(|path| path.exists());

    let config = if let Some(first_config_path) = first_config_path {
        info!("Parsing config from {:?}", first_config_path);
        let data = std::fs::read_to_string(first_config_path).expect("The config path to exist");
        let user_config: GhiUserConfig = toml::from_str(&data).expect("Toml config could not parse");
        let database_dir = if let Some(dir) = parse_database_dir(&home, &user_config.database_dir) {
            info!("Resolved custom databse directory.");
            dir
        } else {
            info!("Could not resolve database dir. Using default.");
            home.join(".ghi")
        };

        GhiConfig {
            config_dir: home.join(".ghi"),
            database_dir,
        }
    } else {
        info!("Using default config");
        GhiConfig {
            config_dir: home.join(".ghi"),
            database_dir: home.join(".ghi")
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
    let mut conn = db::establish_connection(&config);

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
