// --- File: src/backend/dataloader.rs ---
// --- Purpose: Acts as a thread-safe Queue/Buffer for incoming data ---

use super::csi_data::CsiData;
use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;

pub struct Dataloader {
    // Changed from random-access Vec to a Queue
    pub queue: VecDeque<CsiData>,
    pub history: Vec<CsiData>,
}

impl Dataloader {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            history: Vec::new(),
        }
    }

    /// Called by the backend thread to add fresh data
    pub fn push_data_packet(&mut self, packet: CsiData) {
        self.history.push(packet.clone());
        self.queue.push_back(packet);
    }

    /// REPLACEMENT: Called by App::on_tick to consume ALL pending data for averaging
    /// This replaces get_data_packet
    pub fn drain_buffer(&mut self) -> Vec<CsiData> {
        self.queue.drain(..).collect()
    }

    /// Exports the entire history of CsiData to a CSV file.
    pub fn export_history_to_csv(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(filename)?;
        let mut wtr = csv::Writer::from_writer(file);

        for data in &self.history {
            wtr.serialize(data)?;
        }

        wtr.flush()?;
        Ok(())
    }
}