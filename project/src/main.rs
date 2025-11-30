// --- File: src/main.rs ---
// --- Purpose: Application Entry Point, State Management, and Main Run Loop ---

use std::{io, time::Duration};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, DisableMouseCapture, EnableMouseCapture},
};
use ratatui::prelude::*;

// --- Modules ---
pub mod layout_tree;
pub mod theme;
pub mod view_router;
pub mod view_expo;
pub mod overlay_expo;
pub mod input_handler;

// Views
pub mod stats;
pub mod help;
pub mod options;

use layout_tree::{TilingManager, ViewType};
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
        // This is where we will eventually poll the Backend Channel
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
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Initialize App
    let mut app = App::new();

    // 3. Run Loop
    loop {
        // A. Draw UI
        terminal.draw(|f| view_router::ui(f, &app))?;

        // B. Handle Input (Poll for 16ms to target ~60fps)
        if event::poll(Duration::from_millis(16))? {
            input_handler::handle_event(&mut app)?;
        }

        // C. Update Logic / Process Data
        app.on_tick();

        // D. Exit Condition
        if app.should_quit {
            break;
        }
    }

    // 4. Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}