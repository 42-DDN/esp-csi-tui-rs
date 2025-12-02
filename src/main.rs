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
    let mut should_draw = true;

    loop {
        // 1. Render only if needed
        if should_draw {
            terminal.draw(|f| view_router::ui(f, &app))?;
            should_draw = false;
        }

        // 2. Poll for input
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            // Processing LOOP: Drain the event queue
            let start = Instant::now();
            let mut event_processed = false;

            // We loop while events are available AND we haven't spent too long (20ms) processing them
            // This prevents the loop from hanging if input floods in faster than we can process.
            while event::poll(Duration::from_secs(0))? && start.elapsed() < Duration::from_millis(20) {
                if input_handler::handle_event(&mut app)? {
                    event_processed = true;
                }
                if app.should_quit { break; }
            }

            // If we broke out of the loop or poll returned false, ensure we processed at least one if pending
            if !event_processed && !app.should_quit {
                 // Try processing one last event if the loop condition failed but poll was true initially
                 if input_handler::handle_event(&mut app)? {
                     event_processed = true;
                 }
            }

            if event_processed {
                should_draw = true;
            }
        }

        // 3. Data Update
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            should_draw = true; // Always draw on new data
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