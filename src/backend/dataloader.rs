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

        // Define a helper struct for CSV serialization to handle Vec<i32>
        #[derive(serde::Serialize)]
        struct CsiDataCsv<'a> {
            mac: &'a str,
            rssi: i32,
            rate: u32,
            noise_floor: i32,
            channel: u32,
            timestamp: u64,
            sig_len: u32,
            rx_state: u32,
            secondary_channel: u32,
            sgi: u32,
            ant: u32,
            ampdu_cnt: u32,
            sig_mode: u32,
            mcs: u32,
            cwb: u32,
            smoothing: u32,
            not_sounding: u32,
            aggregation: u32,
            stbc: u32,
            fec_coding: u32,
            sig_len_extra: u32,
            data_length: u32,
            csi_raw_data: String,
        }

        for data in &self.history {
            let csv_row = CsiDataCsv {
                mac: &data.mac,
                rssi: data.rssi,
                rate: data.rate,
                noise_floor: data.noise_floor,
                channel: data.channel,
                timestamp: data.timestamp,
                sig_len: data.sig_len,
                rx_state: data.rx_state,
                secondary_channel: data.secondary_channel,
                sgi: data.sgi,
                ant: data.ant,
                ampdu_cnt: data.ampdu_cnt,
                sig_mode: data.sig_mode,
                mcs: data.mcs,
                cwb: data.cwb,
                smoothing: data.smoothing,
                not_sounding: data.not_sounding,
                aggregation: data.aggregation,
                stbc: data.stbc,
                fec_coding: data.fec_coding,
                sig_len_extra: data.sig_len_extra,
                data_length: data.data_length,
                csi_raw_data: format!("{:?}", data.csi_raw_data),
            };
            wtr.serialize(csv_row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn import_history_from_csv(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filename)?;
        let mut rdr = csv::Reader::from_reader(file);

        #[derive(serde::Deserialize)]
        struct CsiDataCsv {
            mac: String,
            rssi: i32,
            rate: u32,
            noise_floor: i32,
            channel: u32,
            timestamp: u64,
            sig_len: u32,
            rx_state: u32,
            secondary_channel: u32,
            sgi: u32,
            ant: u32,
            ampdu_cnt: u32,
            sig_mode: u32,
            mcs: u32,
            cwb: u32,
            smoothing: u32,
            not_sounding: u32,
            aggregation: u32,
            stbc: u32,
            fec_coding: u32,
            sig_len_extra: u32,
            data_length: u32,
            csi_raw_data: String,
        }

        for result in rdr.deserialize() {
            let record: CsiDataCsv = result?;

            // Parse the "[1, 2, 3]" string
            let raw_str = record.csi_raw_data.trim().trim_start_matches('[').trim_end_matches(']');
            let csi_vec: Vec<i32> = if raw_str.is_empty() {
                Vec::new()
            } else {
                raw_str.split(',')
                    .map(|s| s.trim().parse::<i32>())
                    .collect::<Result<Vec<_>, _>>()?
            };

            let data = CsiData {
                mac: record.mac,
                rssi: record.rssi,
                rate: record.rate,
                noise_floor: record.noise_floor,
                channel: record.channel,
                timestamp: record.timestamp,
                sig_len: record.sig_len,
                rx_state: record.rx_state,
                secondary_channel: record.secondary_channel,
                sgi: record.sgi,
                ant: record.ant,
                ampdu_cnt: record.ampdu_cnt,
                sig_mode: record.sig_mode,
                mcs: record.mcs,
                cwb: record.cwb,
                smoothing: record.smoothing,
                not_sounding: record.not_sounding,
                aggregation: record.aggregation,
                stbc: record.stbc,
                fec_coding: record.fec_coding,
                sig_len_extra: record.sig_len_extra,
                data_length: record.data_length,
                csi_raw_data: csi_vec,
            };

            self.history.push(data);
        }
        Ok(())
    }
}
