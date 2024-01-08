mod renderer;
mod history;
mod views;
mod config;

use crate::renderer::Renderer;
use crate::views::history_view::HistoryView;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author = "Sam Uherek <samuherekbiz@gmail.com>")]
#[command(about = "Quick reference to commands", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Add,
    List
}

fn main() -> anyhow::Result<()>{
    let cli = Cli::parse();

    let mut stdout = std::io::stdout();
    let mut renderer = Renderer::new(&mut stdout);

    match &cli.command {
        Some(Commands::Add) => {
            todo!();
        },
        Some(Commands::List) => {
            todo!();
        },
        None => {
            let mut history = HistoryView::new();
            history.load_history()?;

            if history.is_empty() {
                println!("Your command history is empty.");
                return Ok(());
            }

            // It renders the alternative screen to keep the old terminal view
            renderer.enter_screen()?;
            // Interactive render that will be in a loop until it's exited
            history.render(&mut renderer)?;
            // Cleanup and restoring the original termianl view
            renderer.leave_screen()?;

        }
    }

    return Ok(());
}
