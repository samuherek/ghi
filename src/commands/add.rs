use std::io::{self, Read};
use crate::store::Store;
use anyhow::Result;
use tempfile::NamedTempFile;
use std::process::Command;
use std::env;
use std::fs;

pub fn run(value: &Option<String>) -> Result<()> {
    let mut store = Store::new();
    store.init_database()?;
    let mut input = String::new();

    if let Some(value) = value {
        store.create_from_string(value)?;
        input = value.clone();
    } else {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Ok(_) => {
                store.create_from_string(&buf)?;
                input = buf;
            }, 
            Err(err) => eprintln!("Error reading stdion: {}", err)
        };
    };

    let file = NamedTempFile::new()?;
    let path = file.path();

    let path2 = file.path().to_str().unwrap().to_string();
    println!("path, {:?}", path);
    println!("path2, {:?}", path2);

    let editor = env::var("EDITOR")?;

    Command::new(editor)
        .args(path)
        .status()?;

    let i = fs::read_to_string(path)?;
    println!("User input: {}", i);

    println!("added: {:?}", input);

    Ok(())
}
