use std::{fs, io};
use std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
        let name =  file_name(value);
        let path = self.path.join(name);
        
        if !path.exists() {
            let data = format!("{}\n\n", value);
            fs::write(path, data)?;
        }

        Ok(())
    }
}

fn file_name(value: &str) -> String {
    let command = value.split_whitespace().next().expect("The value is not empty");
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let hash = hasher.finish();

    return format!("{}-{:x}.md", command, hash);
}
