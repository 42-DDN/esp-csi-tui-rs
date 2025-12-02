// --- File: src/app.rs ---
// --- Purpose: Holds the central Application State and Logic ---

use std::time::Instant;
use std::cell::RefCell;
use std::collections::HashMap;
use ratatui::layout::Rect;

// Internal Imports
use crate::config_manager;
use crate::frontend::layout_tree::TilingManager;
use crate::frontend::theme::{Theme, ThemeType};
use crate::frontend::view_state::ViewState;
use crate::backend::csi_data::CsiData;
use crate::backend::dataloader;

// Global Constant for History Limit
pub const MAX_HISTORY_SIZE: usize = 10000;

// Data Point for History
#[derive(Clone, Debug)]
pub struct NetworkStats {
    pub packet_count: u64,
    pub rssi: i32,
    pub pps: u64,
    pub snr: i32,
    pub timestamp: u64,
    pub csi: Option<CsiData>,
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
    pub fn new() -> Self {
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
            fullscreen_pane_id: None,
            pane_states: HashMap::new(),
            should_quit: false,

            // Init Stats
            current_stats: NetworkStats { packet_count: 0, rssi: -90, pps: 0, snr: 0, timestamp: 0, csi: None },
            history: Vec::with_capacity(MAX_HISTORY_SIZE),
            start_time: Instant::now(),

            pane_regions: RefCell::new(Vec::new()),
        }
    }

    pub fn get_pane_state_mut(&mut self, id: usize) -> &mut ViewState {
        self.pane_states.entry(id).or_insert_with(ViewState::new)
    }

    // MAIN DATA LOOP
    pub fn on_tick(&mut self) {
        // Fetch next packet based on packet_count
        let idx = self.current_stats.packet_count;

        if let Some(csi_packet) = dataloader::get_data_packet(idx) {
            let elapsed = self.start_time.elapsed().as_millis() as u64;

            // Calculate derived metrics
            let mock_pps = (idx % 50) * 12 + 100; // Still mocked until we have real timing

            // FIX: Convert unsigned byte noise floor (e.g. 161) to signed dBm (e.g. -95)
            // If value > 127, it's likely a signed int8 representation
            let noise_floor = if csi_packet.noise_floor > 127 {
                csi_packet.noise_floor - 256
            } else {
                csi_packet.noise_floor
            };

            let snr = csi_packet.rssi - noise_floor;

            self.current_stats = NetworkStats {
                packet_count: idx + 1,
                rssi: csi_packet.rssi,
                pps: mock_pps,
                snr,
                timestamp: elapsed,
                csi: Some(csi_packet),
            };

            // Push to History (Ring Buffer)
            if self.history.len() >= MAX_HISTORY_SIZE {
                self.history.remove(0);
            }
            self.history.push(self.current_stats.clone());
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