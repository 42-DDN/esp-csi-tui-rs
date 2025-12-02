// --- File: src/app.rs ---
// --- Purpose: Holds the central Application State and Logic ---

use std::time::Instant;
use std::cell::RefCell;
use std::collections::HashMap;
use ratatui::layout::Rect;

use crate::dataloader::Dataloader;
use crate::{config_manager};
use crate::frontend::layout_tree::{TilingManager, SplitDirection};
use crate::frontend::theme::{Theme, ThemeType};
use crate::frontend::view_state::ViewState;
use crate::backend::csi_data::CsiData;

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
    pub start_time: Instant,

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
            history: Vec::with_capacity(MAX_HISTORY_SIZE),
            start_time: Instant::now(),

            pane_regions: RefCell::new(Vec::new()),
            splitter_regions: RefCell::new(Vec::new()),
            drag_state: None,
        }
    }

    pub fn get_pane_state_mut(&mut self, id: usize) -> &mut ViewState {
        self.pane_states.entry(id).or_insert_with(ViewState::new)
    }

    pub fn on_tick(&mut self) {
        // FIX: Cast u64 packet_count to usize for the backend call
        let idx = self.current_stats.packet_count as usize;

        if let Some(csi_packet) = Dataloader::get_data_packet(&mut self.dataloader, idx) {
            let elapsed = self.start_time.elapsed().as_millis() as u64;
            let mock_pps = (idx as u64 % 50) * 12 + 100;

            let noise_floor = if csi_packet.noise_floor > 127 {
                csi_packet.noise_floor - 256
            } else {
                csi_packet.noise_floor
            };

            let snr = csi_packet.rssi - noise_floor;

            self.current_stats = NetworkStats {
                packet_count: (idx + 1) as u64,
                rssi: csi_packet.rssi,
                pps: mock_pps,
                snr,
                timestamp: elapsed,
                csi: Some(csi_packet),
            };

            if self.history.len() >= MAX_HISTORY_SIZE {
                self.history.remove(0);
            }
            self.history.push(self.current_stats.clone());
        } else {
            // No real packet available: insert mock data so UI can still render
            let elapsed = self.start_time.elapsed().as_millis() as u64;
            let mock_pps = (idx as u64 % 50) * 12 + 20;

            // Create a reasonable mock CSI packet. Values chosen to resemble
            // parsed data so downstream consumers behave the same as with real data.
            let mock_csi = CsiData {
                mac: "00:11:22:33:44:55".to_string(),
                rssi: -42,
                rate: 300,
                noise_floor: 200, // will be adjusted below (200 -> -56)
                channel: 36,
                timestamp: elapsed,
                sig_len: 128,
                rx_state: 0,
                secondary_channel: 0,
                sgi: 0,
                ant: 3,
                ampdu_cnt: 0,
                sig_mode: 0,
                mcs: 9,
                cwb: 1,
                smoothing: 0,
                not_sounding: 0,
                aggregation: 0,
                stbc: 0,
                fec_coding: 0,
                sig_len_extra: 0,
                data_length: 1024,
                csi_raw_data: vec![0i32, 12, -8, 7, 3, -2, 1, 0, 4, -1],
            };

            let noise_floor = if mock_csi.noise_floor > 127 {
                mock_csi.noise_floor - 256
            } else {
                mock_csi.noise_floor
            };

            let snr = mock_csi.rssi - noise_floor;

            self.current_stats = NetworkStats {
                packet_count: (idx + 1) as u64,
                rssi: mock_csi.rssi,
                pps: mock_pps,
                snr,
                timestamp: elapsed,
                csi: Some(mock_csi),
            };

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