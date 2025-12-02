// --- File: src/main.rs ---
// --- Purpose: Entry Point. Configures the module tree and runs the main loop. ---

use std::{io, time::Duration};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, DisableMouseCapture, EnableMouseCapture},
};
use ratatui::prelude::*;

// 1. Declare modules
pub mod app; // <--- The App struct lives here now
pub mod input_handler;
pub mod frontend;
pub mod config_manager;

// 2. Re-exports to maintain "crate::App" access throughout the project
pub use app::{App, NetworkStats};

pub use frontend::layout_tree;
pub use frontend::theme;
pub use frontend::view_router;
pub use frontend::view_traits;
pub use frontend::view_state;
pub use frontend::views::stats;
pub use frontend::overlays::{help, options, quit, view_selector, main_menu, save_template, load_template};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = config_manager::init();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| view_router::ui(f, &app))?;

        if event::poll(Duration::from_millis(16))? {
            let _ = input_handler::handle_event(&mut app)?;
        }

        app.on_tick();

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}