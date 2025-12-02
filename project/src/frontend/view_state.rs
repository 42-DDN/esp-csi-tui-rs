// --- File: src/frontend/view_state.rs ---
// --- Purpose: Stores persistent state for each pane (Camera, Playback, Pause) ---

#[derive(Clone, Debug)]
pub struct ViewState {
    // Temporal State
    pub history_offset: usize, // 0 = Live, >0 = Looking back N frames
    pub is_paused: bool,

    // Spatial State (3D / Camera)
    pub camera_x: f64,
    pub camera_y: f64,
    pub zoom: f64,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            history_offset: 0,
            is_paused: false,
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
        }
    }

    // --- Temporal Logic ---
    // Guard against underflow by checking max available history
    pub fn step_back(&mut self, max_history: usize) {
        if self.history_offset < max_history {
            self.history_offset += 1;
            self.is_paused = true;
        }
    }

    pub fn step_forward(&mut self) {
        if self.history_offset > 0 {
            self.history_offset -= 1;
        }
        // If we catch up to 0, we remain paused until user hits 'r'
    }

    pub fn reset_live(&mut self) {
        self.history_offset = 0;
        self.is_paused = false;
    }

    // --- Spatial Logic ---
    pub fn move_camera(&mut self, dx: f64, dy: f64) {
        self.camera_x += dx;
        self.camera_y += dy;
        self.is_paused = true;
    }
}