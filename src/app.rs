// --- File: src/app.rs ---
// --- Purpose: Holds the central Application State and Logic ---

use std::time::{Duration, Instant};
use std::cell::RefCell;
use std::collections::HashMap;
use ratatui::layout::Rect;

use crate::dataloader::Dataloader;
use crate::config_manager;
use crate::frontend::layout_tree::TilingManager;
use crate::frontend::theme::{Theme, ThemeType};
use crate::frontend::view_state::ViewState;
use crate::backend::csi_data::CsiData;

// We store fewer packets because we are storing averages now.
// 10,000 averages @ 10Hz = 1000 seconds (~16 minutes) of history.
pub const MAX_HISTORY_SIZE: usize = 10000;

// Configurable update rate.
// 0.5s = 500ms (Very slow, but good for long term stats)
// 0.1s = 100ms (Recommended for "Real-time" feel)
pub const UPDATE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Debug)]
pub struct NetworkStats {
    pub id: u64, // Unique sequence ID for the UI
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
    pub show_theme_selector: bool,
    pub theme_selector_index: usize,
    pub show_save_input: bool,
    pub input_buffer: String,
    pub show_load_selector: bool,
    pub load_selector_index: usize,
    pub available_templates: Vec<(String, bool)>,

    pub fullscreen_pane_id: Option<usize>,
    pub pane_states: HashMap<usize, ViewState>,
    pub should_quit: bool,

    // Data State
    pub current_stats: NetworkStats,
    pub history: Vec<NetworkStats>,

    // Timing State
    pub start_time: Instant,
    pub last_update_time: Instant,
    pub pps_window: Vec<usize>,

    // Interaction Caches & Backend
    pub pane_regions: RefCell<Vec<(usize, Rect)>>,
    pub dataloader: Dataloader,
    pub splitter_regions: RefCell<Vec<(Vec<usize>, Rect, crate::frontend::layout_tree::SplitDirection, u16, u16)>>,
    pub drag_state: Option<crate::app::DragState>, // Re-using DragState struct definition or define here if moved
}

// State for resizing operation
pub struct DragState {
    pub split_path: Vec<usize>,
    pub start_ratio: u16,
    pub start_mouse_pos: (u16, u16),
    pub direction: crate::frontend::layout_tree::SplitDirection,
    pub container_size: u16,
}

impl App {
    pub fn new() -> Self {
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
            show_theme_selector: false,
            theme_selector_index: 0,
            show_save_input: false,
            input_buffer: String::new(),
            show_load_selector: false,
            load_selector_index: 0,
            available_templates: Vec::new(),
            fullscreen_pane_id: None,
            pane_states: HashMap::new(),
            should_quit: false,

            dataloader: Dataloader::new(),
            current_stats: NetworkStats { id: 0, rssi: -90, pps: 0, snr: 0, timestamp: 0, csi: None },
            history: Vec::with_capacity(MAX_HISTORY_SIZE),

            start_time: Instant::now(),
            last_update_time: Instant::now(),
            pps_window: Vec::new(),

            pane_regions: RefCell::new(Vec::new()),
            splitter_regions: RefCell::new(Vec::new()),
            drag_state: None,
        }
    }

    pub fn get_pane_state_mut(&mut self, id: usize) -> &mut ViewState {
        self.pane_states.entry(id).or_insert_with(ViewState::new)
    }

    pub fn on_tick(&mut self) {
        // 1. Drain the Queue from the background thread
        // We do this every tick to prevent the queue from exploding in memory,
        // even if we don't update the UI yet.
        // (Alternatively, let it build up and drain only when updating,
        // but draining periodically is safer for memory spikes).
        // For accurate averaging over the interval, we need to accumulate them.

        // HOWEVER, since Dataloader is now a Queue, we can simply wait until the
        // timer fires to drain it.

        if self.last_update_time.elapsed() >= UPDATE_INTERVAL {
            // TIME TO UPDATE!

            let raw_packets = self.dataloader.drain_buffer();
            let count = raw_packets.len();

            // Update PPS Window
            self.pps_window.push(count);
            // Keep last 1 second of history (10 * 100ms)
            if self.pps_window.len() > 10 {
                self.pps_window.remove(0);
            }

            let total_packets: usize = self.pps_window.iter().sum();
            let window_secs = self.pps_window.len() as f64 * UPDATE_INTERVAL.as_secs_f64();
            let calculated_pps = if window_secs > 0.0 {
                (total_packets as f64 / window_secs) as u64
            } else {
                0
            };

            if count > 0 {
                // Calculate Average
                let averaged_csi = CsiData::average(&raw_packets);
                let elapsed_ms = self.start_time.elapsed().as_millis() as u64;

                let noise = averaged_csi.noise_floor;
                let snr = averaged_csi.rssi - noise;

                // Create new Stat Snapshot
                let new_stat = NetworkStats {
                    id: self.current_stats.id + 1,
                    rssi: averaged_csi.rssi,
                    pps: calculated_pps,
                    snr,
                    timestamp: elapsed_ms,
                    csi: Some(averaged_csi),
                };

                self.current_stats = new_stat.clone();

                // History Management
                if self.history.len() >= MAX_HISTORY_SIZE {
                    self.history.remove(0);
                }
                self.history.push(new_stat);
            } else {
                // No data received in this interval
                // We can either hold the last value or show "0 PPS"
                 self.current_stats.pps = calculated_pps;
            }

            self.last_update_time = Instant::now();
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