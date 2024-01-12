use std::{fs, io, env};
use anyhow::anyhow;
use std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// The directory name of the dotfolder where we store
/// the databse by default.
const DOTFOLDER_PATH: &str = ".ghi";
/// The name of the database file where we store
/// the data. Currently in markdown.
const INDEX_FILE: &str = "index.md";

type DbItem = String;

#[derive(Debug)]
pub struct DbIndex(Vec<DbItem>);

impl DbIndex {
    fn new() -> Self {
        DbIndex(Vec::new())
    }

    fn add(&mut self, val: &str) {
        self.0.push(val.to_string())
    }

    fn get(&self, idx: usize) -> Option<&DbItem> {
        self.0.get(idx)
    }

    pub fn has(&self, val: &str) -> bool {
        self.0.iter().any(|x| x == val)
    }

    fn all(&self) -> &Vec<DbItem> {
        &self.0       
    }

    fn filter(&self, search: &str, limit: usize) -> Vec<usize> {
        let matcher = SkimMatcherV2::default();
        self.0.iter().enumerate().filter_map(|(idx, x)| {
            matcher.fuzzy_match(&x, search).map(|score| (idx, score))
        })
        .take(limit)
        .map(|(idx, _)| idx)
        .collect()
    }
}

impl FromStr for DbIndex {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s.lines()
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect();

        Ok(DbIndex(values))
    }
}

impl ToString for DbIndex {
    fn to_string(&self) -> String {
        self.0.iter().map(|x| format!("{}\n", {x})).collect()
    }
}

#[derive(Clone, Debug)]
pub struct HistoryItem {
    pub value: String,
    pub selected: bool,
}

impl HistoryItem {
    pub fn new(val: &str, selected: bool) -> Self {
        Self {
            value: val.to_string(),
            selected
        }
    }

}

#[derive(Debug)]
struct HistoryItems(Vec<HistoryItem>);

impl HistoryItems {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn get(&self, idx: usize) -> Option<&HistoryItem> {
        self.0.get(idx)
    }

    fn push(&mut self, item: HistoryItem) {
        self.0.push(item)
    }

    fn filter(&self, search: &str, limit: usize) -> Vec<usize> {
        let matcher = SkimMatcherV2::default();
        self.0.iter().enumerate().filter_map(|(idx, x)| {
            matcher.fuzzy_match(&x.value, search).map(|score| (idx, score))
        })
        .take(limit)
        .map(|(idx, _)| idx)
        .collect()
    }
}

#[derive(Debug)]
struct Cache {
    history: HistoryItems,
    db: DbIndex,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            history: HistoryItems::new(),
            db: DbIndex::new()
        }
    }

    fn load_history_from_file(&mut self, path: &PathBuf) -> io::Result<()> {
        let data = fs::read_to_string(path)?;
        self.db = DbIndex::from_str(&data)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Store {
    dir: PathBuf,
    file: PathBuf,
    cache: Cache
}

impl Store {
    pub fn new() -> Self {
        let cache = Cache::new();
        let home_dir = dirs::home_dir().expect("Could not find home dir");     
        let store_dir = home_dir.join(DOTFOLDER_PATH);
        let store_file = store_dir.join(INDEX_FILE);

        Self {
            dir: store_dir,
            file: store_file,
            cache
        }
    }

    fn init_shell(&mut self) -> anyhow::Result<()> {
        let shell_path = env::var("SHELL")?;
        let shell_name = shell_path.rsplit('/').next().unwrap_or("");
        let home_dir = dirs::home_dir().expect("Could not determine home dir");

        let shell_path = match shell_name {
            "bash" => Ok(home_dir.join(".bash_history")),
            "zsh" => Ok(home_dir.join(".zsh_history")),
            _ => Err(anyhow!("We could not find your shell.")),
        }?;

        let mut cache = HashSet::new();

        for line in fs::read_to_string(shell_path)?.lines().rev() {
            if cache.insert(line) {
                let item = HistoryItem::new(line, self.cache.db.has(line));
                self.cache.history.push(item);
            }
        }

        Ok(())
    }

    fn init_database(&mut self) -> io::Result<()> {
        let store_file = self.dir.join(&self.file);

        if !self.dir.exists() {
            fs::create_dir(&self.dir)?;
        }

        if !store_file.exists() {
            fs::File::create(&store_file)?;
        }
        
        self.cache.load_history_from_file(&store_file)?;

        Ok(())
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        self.init_database()?;
        self.init_shell()?; 
        Ok(())
    }

    pub fn get_history_refs(&self, search: &str, limit: usize) -> Vec<usize> {
        self.cache.history.filter(search, limit)
    }

    pub fn get_history_item(&self, idx: usize) -> Option<&HistoryItem> {
        self.cache.history.get(idx)
    }

    pub fn get_item(&self, idx: usize) -> Option<&DbItem>{
        self.cache.db.get(idx)
    }

    pub fn get_refs(&self, search: &str, limit: usize) -> Vec<usize> {
        self.cache.db.filter(search, limit)
    }

    pub fn create(&mut self, value: &str) -> io::Result<()> {
        self.cache.db.add(value);
        self.commit()?;

        Ok(())
    }

    fn commit(&self) -> io::Result<()> {
        fs::write(&self.file, self.cache.db.to_string())?;
        Ok(())
    }

    pub fn all(&self) -> &Vec<DbItem> {
       self.cache.db.all()
    }
}

fn _file_name(value: &str) -> String {
    let command = value.split_whitespace().next().expect("The value is not empty");
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let hash = hasher.finish();

    return format!("{}-{:x}.md", command, hash);
}
