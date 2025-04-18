mod app;
mod ui;
mod api;
mod models;
mod utils;

use crate::app::{ App };
use crate::api::fetch_stackoverflow_questions;
use crate::ui::run_app;

use crossterm::{
    execute,
    terminal::{ enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use std::{ env, error::Error, io, time::Duration };
use tui::{ backend::CrosstermBackend, Terminal };

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <search-term>", args[0]);
        std::process::exit(1);
    }

    let query = &args[1];
    let items = fetch_stackoverflow_questions(query)?;

    if items.is_empty() {
        eprintln!("No results found for query: {}", query);
        std::process::exit(1);
    }

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(200);
    let app = App::new(items, query.clone());
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}
