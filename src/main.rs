mod app;
mod event;
mod theme;
mod ui;

use std::io::stdout;

use color_eyre::eyre::Result;
use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::AppState;
use event::handle_events;
use ui::ui;

fn main() -> Result<()> {
    color_eyre::install()?;
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = AppState::default();
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame| ui(frame, &mut app))?;
        should_quit = handle_events(&mut app)?;
    }

    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
