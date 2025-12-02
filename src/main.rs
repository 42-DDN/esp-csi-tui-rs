// --- File: src/main.rs ---
// --- Purpose: Entry Point. Configures the module tree and runs the main loop. ---

use std::{io, thread, time::Duration};
use std::sync::{Arc, Mutex}; // Import Arc and Mutex

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
pub mod backend;
pub mod config_manager;
pub mod esp_com;

// 2. Re-exports to maintain "crate::App" access throughout the project
pub use app::{App, NetworkStats};

pub use frontend::layout_tree;
pub use frontend::theme;
pub use frontend::view_router;
pub use frontend::view_traits;
pub use frontend::view_state;
pub use frontend::views::stats;
pub use frontend::overlays::{help, options, quit, view_selector, main_menu, save_template, load_template};
pub use backend::{dataloader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = config_manager::init();

    // 1. Wrap App in Arc<Mutex<>> to allow sharing across threads
    let app = Arc::new(Mutex::new(App::new()));

    // 2. Clone the reference for the background thread
    let app_access = Arc::clone(&app);

    thread::spawn(move || {
        esp_com::esp_com(app_access);
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        // 3. Lock the app briefly to draw the UI
        terminal.draw(|f| {
            let app = app.lock().unwrap();
            view_router::ui(f, &app)
        })?;

        if event::poll(Duration::from_millis(16))? {
            // 4. Lock the app to handle input
            let mut app = app.lock().unwrap();
            let _ = input_handler::handle_event(&mut app)?;
        }

        // 5. Lock the app to tick and check quit condition
        let should_quit = {
            let mut app = app.lock().unwrap();
            app.on_tick();
            app.should_quit
        };

        if should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}