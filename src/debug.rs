use std::fs::File;
use std::io::{Write, Result};


pub fn log(val: &str) -> Result<()>{
    let mut file = File::options().append(true).open("log.txt")?;
    writeln!(&mut file, "{}", val)?;
    Ok(())
}
