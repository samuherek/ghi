use anyhow::anyhow;
use std::path::PathBuf;
use std::{env, fs};

type HistoryCommand = String;
type HistoryCommands = Vec<HistoryCommand>;

#[derive(Default)]
pub struct History {
    search_query: String,
    visible_commands: HistoryCommands,
    all_commands: HistoryCommands,
    visible_limit: usize,
    selected_idx: usize,
    pub quit: bool
}

impl History {
    pub fn new() -> Self {
        History {
            search_query: String::from(""),
            visible_commands: HistoryCommands::new(),
            all_commands: HistoryCommands::new(),
            visible_limit: 10,
            selected_idx: 0,
            quit: false
        }
    }
}

fn get_history_path() -> anyhow::Result<PathBuf> {
    let shell_path = env::var("SHELL")?;
    let shell_name = shell_path.rsplit('/').next().unwrap_or("");
    let home_dir = dirs::home_dir().expect("Could not determine home dir");

    return match shell_name {
        "bash" => Ok(home_dir.join(".bash_history")),
        "zsh" => Ok(home_dir.join(".zsh_history")),
        _ => Err(anyhow!("We could not find your shell.")),
    }
}


/// Read the shell history.
pub fn get_history() -> anyhow::Result<Vec<String>> {
    let history_path = get_history_path()?;

    let data: Vec<String> = String::from_utf8_lossy(&fs::read(history_path)?)
        .lines()
        .rev()
        .map(|l| l.to_string())
        .collect();

    return Ok(data);
}
