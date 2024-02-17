use crossterm::{execute, terminal};
use std::io::{self, stdout};

pub struct Screen {
    with_alternate_screen: bool,
    quit: bool,
}

impl Screen {
    pub fn start() -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        Ok(Self {
            quit: false,
            with_alternate_screen: false
        })
    }
    
    pub fn with_altenrate(&mut self) -> std::io::Result<&Self> {
        self.with_alternate_screen = true; 
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        Ok(self)
    } 

    pub fn set_quit(&mut self) {
        self.quit = true; 
    }

    pub fn get_quit(&self) -> bool {
        self.quit 
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode().map_err(|err| {
            eprintln!("ERROR: disable raw mode: {err}")
        });

        if self.with_alternate_screen {
            let _ = execute!(stdout(), terminal::LeaveAlternateScreen).map_err(|err| {
                eprintln!("ERROR: leave alternate screen: {err}")
            });
        }
    }
}
 
