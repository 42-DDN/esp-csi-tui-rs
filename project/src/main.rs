// --- File: src/main.rs ---
// --- Purpose: Entry Point. Configures the module tree to match the file structure. ---

use std::{io, time::Duration, cell::RefCell};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, DisableMouseCapture, EnableMouseCapture},
};
use ratatui::prelude::*;

// 1. Declare modules
pub mod input_handler;
pub mod frontend;
pub mod config_manager;

pub use frontend::layout_tree;
pub use frontend::theme;
pub use frontend::view_router;
pub use frontend::views::stats;
pub use frontend::overlays::{help, options, quit, view_selector, main_menu, save_template, load_template};

use layout_tree::TilingManager;
use theme::{Theme, ThemeType};

pub struct App {
    pub tiling: TilingManager,
    pub theme: Theme,

    // UI State
    pub show_help: bool,
    pub show_quit_popup: bool,
    pub show_view_selector: bool,
    pub view_selector_index: usize,
    pub show_main_menu: bool,
    pub main_menu_index: usize,

    // Template System State
    pub show_save_input: bool,
    pub input_buffer: String,

    pub show_load_selector: bool,
    pub load_selector_index: usize,
    pub available_templates: Vec<(String, bool)>,

    pub should_quit: bool,
    pub packet_count: u64,
    pub last_rssi: i32,
    pub pane_regions: RefCell<Vec<(usize, Rect)>>,
}

impl App {
    fn new() -> Self {
        // Load default template and theme if available
        let (tiling, theme) = if let Some(tm) = config_manager::load_startup_template() {
            let loaded_theme = if let Some(variant) = tm.theme_variant {
                Theme::new(variant)
            } else {
                Theme::new(ThemeType::Dark)
            };
            (tm, loaded_theme)
        } else {
            (TilingManager::new(), Theme::new(ThemeType::Dark))
        };

        Self {
            tiling,
            theme,

            show_help: false,
            show_quit_popup: false,
            show_view_selector: false,
            view_selector_index: 0,
            show_main_menu: false,
            main_menu_index: 0,

            show_save_input: false,
            input_buffer: String::new(),

            show_load_selector: false,
            load_selector_index: 0,
            available_templates: Vec::new(),

            should_quit: false,
            packet_count: 0,
            last_rssi: -85,
            pane_regions: RefCell::new(Vec::new()),
        }
    }

    //MOCK:
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
            input_handler::handle_event(&mut app)?;
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