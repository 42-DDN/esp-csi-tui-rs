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
use crate::rerun_stream::SharedRerunStreamer;

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
    
    // Rerun Integration
    pub rerun_streamer: Option<SharedRerunStreamer>,
}

impl App {
    pub fn new(rerun_addr: Option<String>) -> Self {
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

        let app = Self {
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
            rerun_streamer: Some(crate::rerun_stream::create_shared_streamer()),
        };

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
                csi: Some(csi_packet.clone()),
            };

            if self.history.len() >= MAX_HISTORY_SIZE {
                self.history.remove(0);
            }
            self.history.push(self.current_stats.clone());
            
            // Log to Rerun if enabled
            if let Some(ref streamer) = self.rerun_streamer {
                if let Ok(mut s) = streamer.lock() {
                    #[cfg(feature = "rerun")]
                    {
                        let frame = crate::rerun_stream::CsiFrame::from(&csi_packet);
                        s.push_csi(&frame);
                    }
                }
            }
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