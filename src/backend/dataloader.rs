// --- File: src/backend/dataloader.rs ---
use super::csi_data::CsiData;

pub struct Dataloader {
    pub data: Vec<Option<CsiData>>,
}

impl Dataloader {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn get_data_packet(&mut self, idx: usize) -> Option<CsiData> {
        if idx >= self.data.len() {
            self.data.push(None);
        }
        self.data.get(idx).cloned().flatten()
    }

    pub fn push_data_packet(&mut self, packet: CsiData) {
        self.data.push(Some(packet));
    }
}
