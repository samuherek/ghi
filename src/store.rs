use std::{fs, io};
use std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;

const DOTFOLDER_PATH: &str = ".ghi";

#[derive(Debug)]
pub struct Store {
    path: PathBuf
}

impl Store {
    pub fn init() -> io::Result<Self> {
        let home_dir = dirs::home_dir().expect("Could not find home dir");     
        let config_path = home_dir.join(DOTFOLDER_PATH);

        if !config_path.exists() {
            fs::create_dir(&config_path)?;
        }

        Ok(Self {
            path: config_path
        })
    }

    pub fn create(&self, value: &String) -> io::Result<()> {
        let path = self.path.join("index.md");
        
        if !path.exists() {
            fs::File::create(&path)?;
        }

        let mut file = fs::File::options().append(true).open(path)?;
        file.write_all(format!("{}\n", value).as_bytes())?;

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
