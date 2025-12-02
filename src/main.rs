// --- File: src/main.rs ---
// --- Purpose: Entry Point. Configures the module tree and runs the main loop. ---

use std::{io, thread, time::{Duration, Instant}};
use std::sync::{Arc, Mutex};

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
pub mod backend;
pub mod config_manager;
pub mod esp_com;
pub mod rerun_stream;

// 2. Re-exports
pub use app::{App, NetworkStats};

pub use frontend::layout_tree;
pub use frontend::theme;
pub use frontend::view_router;
pub use frontend::view_traits;
pub use frontend::view_state;
pub use frontend::views::stats;
pub use frontend::overlays::{help, options, quit, view_selector, main_menu, save_template, load_template, theme_selector};
pub use backend::dataloader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI args for --rerun <addr>
    let args: Vec<String> = std::env::args().collect();
    let mut rerun_addr = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--rerun" && i + 1 < args.len() {
            rerun_addr = Some(args[i+1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let _ = config_manager::init();

    // 1. Wrap App in Arc<Mutex<>> to allow sharing across threads
    let app = Arc::new(Mutex::new(App::new(rerun_addr)));

    // 2. Clone the reference for the background thread
    let app_access = Arc::clone(&app);

    // TODO: Create src/esp_com.rs if you haven't already, or comment this block out
    thread::spawn(move || {
        esp_com::esp_com(app_access);
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Loop Timing Control
    let tick_rate = Duration::from_millis(100); // 10Hz Data Updates
    let mut last_tick = Instant::now();

    loop {
        // 1. Render Layer
        // Lock the app briefly to draw the UI
        terminal.draw(|f| {
            let app = app.lock().unwrap();
            view_router::ui(f, &app)
        })?;

        // 2. Input Layer
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            // Processing LOOP: Drain the event queue
            let start = Instant::now();

            // Loop while events are available AND we haven't spent too long (20ms) processing them.
            while event::poll(Duration::from_millis(0))? && start.elapsed() < Duration::from_millis(20) {
                // Lock the app to handle input
                let mut app_guard = app.lock().unwrap();
                let _ = input_handler::handle_event(&mut app_guard)?;

                if app_guard.should_quit {
                    // We need to release the lock before breaking,
                    // but since we are breaking the loop immediately, it's fine.
                    drop(app_guard); // Explicit drop for clarity
                    break;
                }
            }
        }

        // Check quit condition from input loop (requires re-locking or checking flags)
        {
            let app_guard = app.lock().unwrap();
            if app_guard.should_quit {
                break;
            }
        }

        // 3. Data Update Layer
        if last_tick.elapsed() >= tick_rate {
            let should_quit = {
                let mut app_guard = app.lock().unwrap();
                app_guard.on_tick();
                last_tick = Instant::now();
                app_guard.should_quit
            };

            if should_quit {
                break;
            }
        }
    } // <--- This closing brace was missing!

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}