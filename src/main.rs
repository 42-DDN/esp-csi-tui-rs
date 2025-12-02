// --- File: src/main.rs ---
// --- Purpose: Entry Point. Configures the module tree to match the file structure. ---

use std::{io, time::{Duration, Instant}, cell::RefCell, collections::HashMap};
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
pub use frontend::view_traits;
pub use frontend::view_state;
pub use frontend::views::stats;
pub use frontend::overlays::{help, options, quit, view_selector, main_menu, save_template, load_template};

use layout_tree::TilingManager;
use theme::{Theme, ThemeType};
use view_state::ViewState;

// Global Constant for History Limit
pub const MAX_HISTORY_SIZE: usize = 10000;

// Data Point for History
#[derive(Clone, Copy, Debug)]
pub struct NetworkStats {
    pub packet_count: u64,
    pub rssi: i32,
    pub pps: u64,
    pub snr: i32,
    pub timestamp: u64,
}

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

    // Fullscreen & Playback State
    pub fullscreen_pane_id: Option<usize>,
    pub pane_states: HashMap<usize, ViewState>,

    pub should_quit: bool,

    // Data & History
    pub current_stats: NetworkStats,
    pub history: Vec<NetworkStats>,
    pub start_time: Instant,

    pub pane_regions: RefCell<Vec<(usize, Rect)>>,
}

impl App {
    fn new() -> Self {
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
            fullscreen_pane_id: None,
            pane_states: HashMap::new(),
            should_quit: false,

            // Init Stats
            current_stats: NetworkStats { packet_count: 0, rssi: -90, pps: 0, snr: 0, timestamp: 0 },
            history: Vec::with_capacity(MAX_HISTORY_SIZE),
            start_time: Instant::now(),

            pane_regions: RefCell::new(Vec::new()),
        }
    }

    pub fn get_pane_state_mut(&mut self, id: usize) -> &mut ViewState {
        self.pane_states.entry(id).or_insert_with(ViewState::new)
    }

    // MOCK DATA GENERATOR
    // TODO: Remove this mock logic when integrating real hardware
    fn on_tick(&mut self) {
        // 1. Generate Mock Data
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        let mock_packet_count = self.current_stats.packet_count + 1;
        let mock_rssi = -30 - (rand::random::<i32>().abs() % 60);
        let mock_pps = (mock_packet_count % 50) * 12 + 100;
        let mock_snr = mock_rssi - (-95);

        // 2. Update Current
        self.current_stats = NetworkStats {
            packet_count: mock_packet_count,
            rssi: mock_rssi,
            pps: mock_pps,
            snr: mock_snr,
            timestamp: elapsed,
        };

        // 3. Push to History (Ring Buffer Logic)
        if self.history.len() >= MAX_HISTORY_SIZE {
            self.history.remove(0); // Remove oldest
        }
        self.history.push(self.current_stats);
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