// --- File: src/frontend/view_state.rs ---
// --- Purpose: Stores persistent state for each pane (Camera, Playback, Pause) ---

#[derive(Clone, Debug)]
pub struct ViewState {

    pub history_offset: usize,
    pub is_paused: bool,


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


    pub fn step_back(&mut self) {
        self.history_offset += 1;
        self.is_paused = true;
    }

    pub fn step_forward(&mut self) {
        if self.history_offset > 0 {
            self.history_offset -= 1;
        }
    
    
    }

    pub fn reset_live(&mut self) {
        self.history_offset = 0;
        self.is_paused = false;
    
    }


    pub fn move_camera(&mut self, dx: f64, dy: f64) {
        self.camera_x += dx;
        self.camera_y += dy;
        self.is_paused = true;
    }
}