// --- File: src/main.rs ---
// --- Purpose: Entry Point. Configures the module tree and runs the main loop. ---

use std::{io, time::{Duration, Instant}};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, DisableMouseCapture, EnableMouseCapture},
};
use ratatui::prelude::*;

// 1. Declare modules
pub mod app;
pub mod input_handler;
pub mod frontend;
pub mod config_manager;
pub mod backend;

// 2. Re-exports
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

    // Loop Timing Control
    let tick_rate = Duration::from_millis(100); // 10Hz Data Updates (Simulate Hardware)
    let mut last_tick = Instant::now();

    loop {
        // Always draw (target ~60fps responsive UI)
        terminal.draw(|f| view_router::ui(f, &app))?;

        // Poll for inputs with a short timeout to keep UI responsive
        // but NOT blocking data updates forever
        if event::poll(Duration::from_millis(16))? {
            let _ = input_handler::handle_event(&mut app)?;
        }

        // Decouple Data Generation from Input Loop
        // Only update data if the tick_rate duration has passed
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}