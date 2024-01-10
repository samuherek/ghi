use anyhow::anyhow;
use std::collections::HashSet;
use std::{env, fs};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

type HistoryCommand = String;
type HistoryCommands = Vec<HistoryCommand>;

pub enum MoveDirection {
    Up,
    Down
}

#[derive(Default)]
pub struct History {
    pub query: String,
    prev_query: String,
    pub results: HistoryCommands,
    history: HistoryCommands,
    pub selected_idx: usize,
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
        
        let mut cache = HashSet::new();
        let mut data = Vec::new();

        for line in fs::read_to_string(shell_path)?.lines().rev() {
            if cache.insert(line) {
                data.push(line.to_string());
            }
        }

        Ok(History {
            query: String::from(""),
            prev_query: String::from(""),
            results: Vec::new(), 
            history: data,
            selected_idx: 0,
        })
    }

    pub fn init_search(&mut self, limit: usize) -> &Vec<String> {
        self.results = self.history.iter().take(limit).cloned().collect();
        return &self.results; 
    }

    pub fn search(&mut self, limit: usize) -> &Vec<String> {
        if self.query == self.prev_query {
            return &self.results; 
        } else {
            if self.query == "" {
                self.results = self.history.iter().take(limit).cloned().collect();
            } else {
                let matcher = SkimMatcherV2::default();

                self.results = self.history.iter().filter_map(|x| {
                    matcher.fuzzy_match(x, &self.query).map(|score| (x, score))
                })
                .take(limit)
                .map(|(x, _)| x.clone())
                .collect();
            }
            return &self.results; 
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

    pub fn get_selection(&self) -> Option<&String> {
        self.results.get(self.selected_idx)
    }
}
