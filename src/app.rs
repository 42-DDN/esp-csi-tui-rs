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
use crate::rerun_stream::SharedRerunStreamer;

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
    // Cumulative I/Q Distribution Grid (24x24)
    // Stores the frequency count of (I, Q) pairs accumulated over time.
    pub distribution_grid: [[f32; 24]; 24],
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
    pub show_export_input: bool,
    pub export_input_buffer: String,
    pub show_load_selector: bool,
    pub load_selector_index: usize,
    pub available_templates: Vec<(String, bool)>,

    pub fullscreen_pane_id: Option<usize>,
    pub pane_states: HashMap<usize, ViewState>,
    pub should_quit: bool,
    pub should_reset_esp: bool,

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

    // Rerun Integration
    pub rerun_streamer: Option<SharedRerunStreamer>,
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
    pub fn new(rerun_addr: Option<String>, csv_file: Option<String>) -> Self {
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

        let mut app = Self {
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
            show_export_input: false,
            export_input_buffer: String::new(),
            show_load_selector: false,
            load_selector_index: 0,
            available_templates: Vec::new(),
            fullscreen_pane_id: None,
            pane_states: HashMap::new(),
            should_quit: false,
            should_reset_esp: false,

            dataloader: Dataloader::new(),
            current_stats: NetworkStats {
                id: 0,
                rssi: -90,
                pps: 0,
                snr: 0,
                timestamp: 0,
                csi: None,
                distribution_grid: [[0.0; 24]; 24],
            },
            history: Vec::with_capacity(MAX_HISTORY_SIZE),

            start_time: Instant::now(),
            last_update_time: Instant::now(),
            pps_window: Vec::new(),

            pane_regions: RefCell::new(Vec::new()),
            splitter_regions: RefCell::new(Vec::new()),
            drag_state: None,
            rerun_streamer: Some(crate::rerun_stream::create_shared_streamer()),
        };

        // Load CSV if provided
        if let Some(path) = csv_file {
            if let Err(e) = app.dataloader.import_history_from_csv(&path) {
                eprintln!("Failed to load CSV: {}", e);
            } else {
                // Populate App::history from dataloader.history
                let mut previous_grid = [[0.0; 24]; 24];
                let mut id_counter = 0;

                for csi in &app.dataloader.history {
                    id_counter += 1;
                    let snr = csi.rssi - csi.noise_floor;

                    // Calculate Grid
                    let mut grid = previous_grid;
                    const GRID_SIZE: usize = 24;
                    const MIN_VAL: f64 = -128.0;
                    const MAX_VAL: f64 = 128.0;
                    const BIN_WIDTH: f64 = (MAX_VAL - MIN_VAL) / GRID_SIZE as f64;

                    let sc_count = csi.csi_raw_data.len() / 2;
                    for s in 0..sc_count {
                        let i_val = csi.csi_raw_data.get(s * 2).copied().unwrap_or(0) as f64;
                        let q_val = csi.csi_raw_data.get(s * 2 + 1).copied().unwrap_or(0) as f64;

                        let bx = ((i_val - MIN_VAL) / BIN_WIDTH).floor() as usize;
                        let by = ((q_val - MIN_VAL) / BIN_WIDTH).floor() as usize;

                        if bx < GRID_SIZE && by < GRID_SIZE {
                            grid[bx][by] += 1.0;
                        }
                    }
                    previous_grid = grid;

                    let stat = NetworkStats {
                        id: id_counter,
                        rssi: csi.rssi,
                        pps: 0, // Static file
                        snr,
                        timestamp: csi.timestamp,
                        csi: Some(csi.clone()),
                        distribution_grid: grid,
                    };
                    app.history.push(stat);
                }

                // Set current stats to last one
                if let Some(last) = app.history.last() {
                    app.current_stats = last.clone();
                }
            }
        }

        if let Some(addr) = rerun_addr {
            if let Some(ref streamer) = app.rerun_streamer {
                if let Ok(mut s) = streamer.lock() {
                    s.connect(&addr);
                }
            }
        }

        app
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

                // --- Calculate Distribution Grid (Cumulative) ---
                let mut grid = self.current_stats.distribution_grid; // Copy previous state (Cumulative)
                const GRID_SIZE: usize = 24;
                const MIN_VAL: f64 = -128.0;
                const MAX_VAL: f64 = 128.0;
                const BIN_WIDTH: f64 = (MAX_VAL - MIN_VAL) / GRID_SIZE as f64;

                let sc_count = averaged_csi.csi_raw_data.len() / 2;
                for s in 0..sc_count {
                    let i_val = averaged_csi.csi_raw_data.get(s * 2).copied().unwrap_or(0) as f64;
                    let q_val = averaged_csi.csi_raw_data.get(s * 2 + 1).copied().unwrap_or(0) as f64;

                    let bx = ((i_val - MIN_VAL) / BIN_WIDTH).floor() as usize;
                    let by = ((q_val - MIN_VAL) / BIN_WIDTH).floor() as usize;

                    if bx < GRID_SIZE && by < GRID_SIZE {
                        grid[bx][by] += 1.0;
                    }
                }

                // Create new Stat Snapshot
                let new_stat = NetworkStats {
                    id: self.current_stats.id + 1,
                    rssi: averaged_csi.rssi,
                    pps: calculated_pps,
                    snr,
                    timestamp: elapsed_ms,
                    csi: Some(averaged_csi.clone()),
                    distribution_grid: grid,
                };

                self.current_stats = new_stat.clone();

                // History Management
                if self.history.len() >= MAX_HISTORY_SIZE {
                    self.history.remove(0);
                }
                self.history.push(new_stat);

                // Log to Rerun if enabled
                if let Some(ref streamer) = self.rerun_streamer {
                    if let Ok(mut s) = streamer.lock() {
                        #[cfg(feature = "rerun")]
                        {
                            let frame = crate::rerun_stream::CsiFrame::from(&averaged_csi);
                            s.push_csi(&frame);
                        }
                    }
                }
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