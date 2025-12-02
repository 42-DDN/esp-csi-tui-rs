// --- File: src/app.rs ---
// --- Purpose: Holds the central Application State and Logic ---

use std::time::Instant;
use std::cell::RefCell;
use std::collections::HashMap;
use ratatui::layout::Rect;

use crate::config_manager;
use crate::frontend::layout_tree::{TilingManager, SplitDirection};
use crate::frontend::theme::{Theme, ThemeType};
use crate::frontend::view_state::ViewState;
use crate::backend::csi_data::CsiData;
use crate::backend::dataloader::Dataloader;

pub const MAX_HISTORY_SIZE: usize = 10000;

#[derive(Clone, Debug)]
pub struct NetworkStats {
    pub packet_count: u64,
    pub rssi: i32,
    pub pps: u64,
    pub snr: i32,
    pub timestamp: u64,
    pub csi: Option<CsiData>,
}

// State for resizing operation
pub struct DragState {
    pub split_path: Vec<usize>,
    pub start_ratio: u16,
    pub start_mouse_pos: (u16, u16),
    pub direction: SplitDirection,
    pub container_size: u16,
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

    pub current_stats: NetworkStats,
    pub history: Vec<NetworkStats>,

    // Sampling State
    pub last_history_timestamp: u64,
    pub start_time: Instant,
    pub packet_buffer: Vec<CsiData>,
    pub pps_counter: u64, // <--- Added: Real PPS Counter

    // Interaction Caches
    pub pane_regions: RefCell<Vec<(usize, Rect)>>,
    pub dataloader: Dataloader,
    pub splitter_regions: RefCell<Vec<(Vec<usize>, Rect, SplitDirection)>>,
    pub drag_state: Option<DragState>,
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

            current_stats: NetworkStats { packet_count: 0, rssi: -90, pps: 0, snr: 0, timestamp: 0, csi: None },
            history: Vec::new(),
            last_history_timestamp: 0,
            start_time: Instant::now(),
            packet_buffer: Vec::new(),
            pps_counter: 0, // <--- Init

            pane_regions: RefCell::new(Vec::new()),
            splitter_regions: RefCell::new(Vec::new()),
            drag_state: None,
        }
    }

    pub fn get_pane_state_mut(&mut self, id: usize) -> &mut ViewState {
        self.pane_states.entry(id).or_insert_with(ViewState::new)
    }

    pub fn on_tick(&mut self) {
        let idx = self.current_stats.packet_count as usize + self.packet_buffer.len();

        if let Some(csi_packet) = self.dataloader.get_data_packet(idx) {
            // Check for valid data (non-zero payload)
            // Real hardware sometimes sends empty CSI frames; we want to exclude those from PPS if needed,
            // or simply count valid frames.
            let has_data = !csi_packet.csi_raw_data.is_empty() && csi_packet.csi_raw_data.iter().any(|&x| x != 0);

            if has_data {
                self.pps_counter += 1;
            }

            // Buffer for averaging
            self.packet_buffer.push(csi_packet);
        }

        // Check if 1 second has passed to process the average
        let elapsed = self.start_time.elapsed().as_millis() as u64;

        if elapsed >= self.last_history_timestamp + 1000 {
            if let Some(avg_packet) = self.calculate_average_packet() {

                // Use the REAL counter we tracked over the last second
                let real_pps = self.pps_counter;

                // Reset for next second
                self.pps_counter = 0;

                let noise_floor = if avg_packet.noise_floor > 127 {
                    avg_packet.noise_floor - 256
                } else {
                    avg_packet.noise_floor
                };

                let snr = avg_packet.rssi - noise_floor;

                // Update UI state with Averaged Data + Real PPS
                self.current_stats = NetworkStats {
                    packet_count: idx as u64,
                    rssi: avg_packet.rssi,
                    pps: real_pps,
                    snr,
                    timestamp: elapsed,
                    csi: Some(avg_packet),
                };

                if self.history.len() >= MAX_HISTORY_SIZE {
                    self.history.remove(0);
                }
                self.history.push(self.current_stats.clone());
            }

            self.packet_buffer.clear();
            self.last_history_timestamp = elapsed;
        }
    }

    // Helper to average the buffered packets
    fn calculate_average_packet(&self) -> Option<CsiData> {
        if self.packet_buffer.is_empty() {
            return None;
        }

        let count = self.packet_buffer.len() as i32;
        let mut sum_rssi = 0;
        let mut sum_noise = 0;

        let len = self.packet_buffer[0].csi_raw_data.len();
        let mut sum_csi = vec![0i64; len];

        for p in &self.packet_buffer {
            sum_rssi += p.rssi;
            sum_noise += p.noise_floor;

            for (i, val) in p.csi_raw_data.iter().enumerate() {
                if i < len {
                    sum_csi[i] += *val as i64;
                }
            }
        }

        let mut avg = self.packet_buffer.last().cloned().unwrap();
        avg.rssi = sum_rssi / count;
        avg.noise_floor = sum_noise / count;
        avg.csi_raw_data = sum_csi.into_iter().map(|x| (x / count as i64) as i32).collect();

        Some(avg)
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