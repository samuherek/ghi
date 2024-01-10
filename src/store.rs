use std::{fs, io};
use std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

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
    fn add(&mut self, val: &str) {
        self.0.push(val.to_string())
    }

    pub fn has(&self, val: &str) -> bool {
        self.0.iter().any(|x| x == val)
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

#[derive(Debug)]
pub struct Store {
    dir: PathBuf,
    file: PathBuf,
    pub db: DbIndex,
}

impl Store {
    pub fn init() -> io::Result<Self> {
        let home_dir = dirs::home_dir().expect("Could not find home dir");     
        let store_dir = home_dir.join(DOTFOLDER_PATH);
        let store_file = store_dir.join(INDEX_FILE);

        if !store_dir.exists() {
            fs::create_dir(&store_dir)?;
        }

        if !store_file.exists() {
            fs::File::create(&store_file)?;
        }

        let db = DbIndex::from_str(&fs::read_to_string(&store_file)?)?;
    
        Ok(Self {
            dir: store_dir,
            file: store_file,
            db
        })
    }

    pub fn create(&mut self, value: &str) -> io::Result<()> {
        self.db.add(value);
        self.commit()?;

        Ok(())
    }

    fn commit(&self) -> io::Result<()> {
        fs::write(&self.file, self.db.to_string())?;
        Ok(())
    }
}

fn _file_name(value: &str) -> String {
    let command = value.split_whitespace().next().expect("The value is not empty");
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let hash = hasher.finish();

    return format!("{}-{:x}.md", command, hash);
}
