// --- File: src/main.rs ---
// --- Purpose: Entry Point. Configures the module tree to match the file structure. ---

use std::{io, time::Duration};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, DisableMouseCapture, EnableMouseCapture},
};
use ratatui::prelude::*;

// 1. Declare the top-level modules
pub mod input_handler;
pub mod frontend;

// 2. Re-export frontend modules for convenience
pub use frontend::layout_tree;
pub use frontend::theme;
pub use frontend::view_router;

// 3. Re-export Views (directly from the views module now)
pub use frontend::views::stats;
// When you implement the other files, uncomment these:
pub use frontend::views::polar;
pub use frontend::views::isometric;
pub use frontend::views::spectrogram;
pub use frontend::views::phase;
pub use frontend::views::camera;

// 4. Re-export Overlays
pub use frontend::overlays::help;
pub use frontend::overlays::options;

// --- Imports ---
use layout_tree::{TilingManager};
use theme::{Theme, ThemeType};

// --- App State ---
pub struct App {
    pub tiling: TilingManager,
    pub theme: Theme,
    pub sidebar_active: bool,
    pub sidebar_index: usize,
    pub show_help: bool,
    pub show_options: bool,
    pub should_quit: bool,

    // Data State (Mock)
    pub packet_count: u64,
    pub last_rssi: i32,
}

impl App {
    fn new() -> Self {
        Self {
            tiling: TilingManager::new(),
            theme: Theme::new(ThemeType::Dark),
            sidebar_active: false,
            sidebar_index: 0,
            show_help: false,
            show_options: false,
            should_quit: false,
            packet_count: 0,
            last_rssi: -85,
        }
    }

    fn on_tick(&mut self) {
        self.packet_count += 1;
        if self.packet_count % 10 == 0 {
            self.last_rssi = -30 - (rand::random::<i32>().abs() % 60);
        }
    }

    pub fn next_theme(&mut self) {
        let next = match self.theme.variant {
            ThemeType::Dark => ThemeType::Light,
            ThemeType::Light => ThemeType::Nordic,
            ThemeType::Nordic => ThemeType::Gruvbox,
            ThemeType::Gruvbox => ThemeType::Catppuccin,
            ThemeType::Catppuccin => ThemeType::Dark,
        };
        self.theme = Theme::new(next);
    }
}

// --- Main Loop ---
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize App
    let mut app = App::new();

    // Run Loop
    loop {
        terminal.draw(|f| view_router::ui(f, &app))?;

        if event::poll(Duration::from_millis(16))? {
            input_handler::handle_event(&mut app)?;
        }

        app.on_tick();

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}