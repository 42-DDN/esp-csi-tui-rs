// --- File: src/backend/dataloader.rs ---
// --- Purpose: Acts as a thread-safe Queue/Buffer for incoming data ---

use super::csi_data::CsiData;
use std::collections::VecDeque;

pub struct Dataloader {
    // Changed from random-access Vec to a Queue
    pub queue: VecDeque<CsiData>,
}

impl Dataloader {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Called by the backend thread to add fresh data
    pub fn push_data_packet(&mut self, packet: CsiData) {
        self.queue.push_back(packet);
    }

    /// REPLACEMENT: Called by App::on_tick to consume ALL pending data for averaging
    /// This replaces get_data_packet
    pub fn drain_buffer(&mut self) -> Vec<CsiData> {
        self.queue.drain(..).collect()
    }
}