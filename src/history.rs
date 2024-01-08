use anyhow::anyhow;
use std::path::PathBuf;
use std::{env, fs};

type HistoryCommand = String;
type HistoryCommands = Vec<HistoryCommand>;

pub enum MoveDirection {
    Up,
    Down
}

#[derive(Default)]
pub struct History {
    search_query: String,
    pub visible_commands: HistoryCommands,
    all_commands: HistoryCommands,
    visible_limit: usize,
    pub selected_idx: usize,
    pub quit: bool
}

impl History {
    pub fn new() -> anyhow::Result<Self> {
        let shell_path = env::var("SHELL")?;
        let shell_name = shell_path.rsplit('/').next().unwrap_or("");
        let home_dir = dirs::home_dir().expect("Could not determine home dir");

        let shell_path = match shell_name {
            "bash" => Ok(home_dir.join(".bash_history")),
            "zsh" => Ok(home_dir.join(".zsh_history")),
            _ => Err(anyhow!("We could not find your shell.")),
        }?;
        
        let limit = 10;
        let data: Vec<String> = fs::read_to_string(shell_path)? 
            .lines()
            .rev()
            .map(|x| x.to_string())
            .collect();

        Ok(History {
            search_query: String::from(""),
            visible_commands: data.iter().take(limit).cloned().collect(),
            all_commands: data,
            visible_limit: limit,
            selected_idx: 0,
            quit: false
        })
    }

    pub fn move_selected_index(&mut self, dir: MoveDirection) {
        if self.visible_commands.len() == 0 {
            return;
        }

        match dir {
            MoveDirection::Up => {
                self.selected_idx = self.selected_idx.saturating_sub(1);
            },
            MoveDirection::Down => {
                self.selected_idx = (self.selected_idx + 1).min(self.visible_commands.len() - 1);
            }
        }
    }
}
