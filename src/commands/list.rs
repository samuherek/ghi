use std::io;
use crate::store::Store;

pub fn run() -> io::Result<()> {
    let mut store = Store::new();
    store.init_database()?;
    for item in store.db_take(None) {
        println!("{}", item);
    };

    Ok(())
}
