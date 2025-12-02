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
pub use frontend::overlays::{help, options, quit, view_selector, main_menu, save_template, load_template, theme_selector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = config_manager::init();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Loop Timing Control
    let tick_rate = Duration::from_millis(100); // 10Hz Data Updates
    let mut last_tick = Instant::now();

    loop {
        // 1. Render Layer
        // Always draw. This ensures UI responsiveness and simplifies the state logic.
        // Ratatui's double-buffering makes this efficient enough.
        terminal.draw(|f| view_router::ui(f, &app))?;

        // 2. Input Layer
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            // Processing LOOP: Drain the event queue
            // This prevents "lag" when holding down a key (OS repeats keys faster than draw loop)
            let start = Instant::now();

            // Loop while events are available AND we haven't spent too long (20ms) processing them.
            // Using poll(0) ensures we only process events that are ALREADY in the queue.
            while event::poll(Duration::from_millis(0))? && start.elapsed() < Duration::from_millis(20) {
                // We don't care if it returns true/false here, just consume it.
                // If it was a valid key, state updates. If invalid, we just consume the event.
                let _ = input_handler::handle_event(&mut app)?;

                if app.should_quit {
                    break;
                }
            }
        }

        // 3. Data Update Layer
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