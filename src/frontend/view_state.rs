// --- File: src/frontend/view_state.rs ---
// --- Purpose: Stores persistent state for each pane (Camera, Playback, Pause) ---

#[derive(Clone, Debug)]
pub struct ViewState {
    // Temporal State
    // If Some(id), we are locked to that specific packet ID (Paused/Replay).
    // If None, we are following the Live head.
    pub anchor_packet_id: Option<u64>,

    // Spatial State (3D / Camera)
    pub camera_x: f64,
    pub camera_y: f64,
    pub zoom: f64,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            anchor_packet_id: None,
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
        }
    }

    // --- Temporal Logic ---

    /// Freezes the view at the specified packet ID
    pub fn pause_at(&mut self, current_live_id: u64) {
        if self.anchor_packet_id.is_none() {
            self.anchor_packet_id = Some(current_live_id);
        }
    }

    pub fn step_back(&mut self, current_live_id: u64, min_id: u64) {
        // If live, start anchoring at current
        let target = self.anchor_packet_id.unwrap_or(current_live_id);

        if target > min_id {
            self.anchor_packet_id = Some(target - 1);
        } else {
            // Loop around to the newest packet (Live)
            // We can either set it to Some(current) or None (Live mode)
            // Setting to None returns to "Live Mode" which is effectively the newest.
            self.anchor_packet_id = None;
        }
    }

    pub fn step_forward(&mut self, current_live_id: u64, min_id: u64) {
        if let Some(target) = self.anchor_packet_id {
            if target < current_live_id {
                self.anchor_packet_id = Some(target + 1);
            } else {
                // We are at the newest packet. Loop around to the oldest.
                self.anchor_packet_id = Some(min_id);
            }
        } else {
            // We are currently Live (Newest). Loop to oldest.
            self.anchor_packet_id = Some(min_id);
        }
    }

    pub fn reset_live(&mut self) {
        self.anchor_packet_id = None;
    }

    // --- Spatial Logic ---
    pub fn move_camera(&mut self, dx: f64, dy: f64) {
        self.camera_x += dx;
        self.camera_y += dy;

        // Clamp Tilt (Y) to match visual limits in raw_scatter.rs
        // Visual: elevation = PI/4 - (y * 0.05) clamped to [0.1, PI/2 - 0.1]
        // Limits: y approx +/- 13.7
        self.camera_y = self.camera_y.clamp(-14.0, 14.0);

        // Wrap Rotation (X) to keep values sane
        // Visual: azimuth = PI/4 + x * 0.1
        // Period: 2*PI / 0.1 = 20*PI approx 62.83
        let period = 20.0 * std::f64::consts::PI;
        if self.camera_x > period {
            self.camera_x -= period;
        } else if self.camera_x < -period {
            self.camera_x += period;
        }
    }
}