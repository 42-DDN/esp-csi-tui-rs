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

    pub fn step_back(&mut self, current_live_id: u64) {
        // If live, start anchoring at current - 1
        // If already anchored, decrement anchor
        let target = self.anchor_packet_id.unwrap_or(current_live_id);
        if target > 0 {
            self.anchor_packet_id = Some(target - 1);
        }
    }

    pub fn step_forward(&mut self, current_live_id: u64) {
        if let Some(target) = self.anchor_packet_id {
            if target < current_live_id {
                self.anchor_packet_id = Some(target + 1);
            } else {
                // If we catch up to live, we can optionally detach
                // For now, let's keep it anchored at the latest ID so it feels like "Paused at end"
                self.anchor_packet_id = Some(current_live_id);
            }
        }
    }

    pub fn reset_live(&mut self) {
        self.anchor_packet_id = None;
    }

    // --- Spatial Logic ---
    pub fn move_camera(&mut self, dx: f64, dy: f64) {
        self.camera_x += dx;
        self.camera_y += dy;
    }
}